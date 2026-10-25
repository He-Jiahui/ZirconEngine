use zircon_runtime::asset::DataAsset;
use zircon_runtime_interface::reflect::ReflectTypeInfo;

use super::{
    cell_value, raw_source, DataTableCellCodec, DataTableCellError, DataTableSourceAdapter,
    DataTableSourceError,
};

impl DataTableCellCodec {
    /// Prepares one exact source-key rename without changing row order or non-key bytes.
    ///
    /// Keys follow the existing JSON source contract: nonempty, case-sensitive strings.
    /// This does not import UE FName sanitation, reserve NAME_None or rewrite references.
    /// The caller must authorize the actual schema/source association, document revision,
    /// dependent references and one Document/history/CAS transaction before publishing.
    /// No-op still validates the whole source and requires the original row to exist.
    pub fn prepare_row_rename(
        asset: &DataAsset,
        row_type: Option<&ReflectTypeInfo>,
        key_field: &str,
        old_key: &str,
        new_key: &str,
    ) -> Result<Option<DataAsset>, DataTableCellError> {
        let source = DataTableSourceAdapter::read(asset, row_type, key_field)?;
        let row_index = source
            .rows
            .iter()
            .position(|row| row.source_key == old_key)
            .ok_or_else(|| DataTableCellError::MissingRow(old_key.into()))?;
        if new_key.is_empty() {
            return Err(DataTableSourceError::InvalidRowKey {
                source_row: row_index,
                key_field: key_field.into(),
            }
            .into());
        }
        if old_key == new_key {
            return Ok(None);
        }
        if source.rows.iter().any(|row| row.source_key == new_key) {
            return Err(DataTableSourceError::DuplicateRowKey(new_key.into()).into());
        }
        let raw_rows = raw_source::rows(&asset.text)?;
        let raw_key = raw_rows[row_index]
            .0
            .iter()
            .find(|(name, _)| name == key_field)
            .map(|(_, value)| *value)
            .ok_or(DataTableCellError::SourceInvariant)?;
        let range =
            raw_source::span(&asset.text, raw_key).ok_or(DataTableCellError::SourceInvariant)?;
        let replacement = serde_json::to_string(new_key)
            .map_err(|error| DataTableSourceError::Encoding(error.to_string()))?;
        let mut candidate = asset.clone();
        candidate.text.replace_range(range, &replacement);
        candidate.canonical_json = serde_json::from_str(&candidate.text)
            .map_err(|error| DataTableSourceError::Encoding(error.to_string()))?;
        let reopened = DataTableSourceAdapter::read(&candidate, row_type, key_field)?;
        if reopened.columns != source.columns || reopened.rows.len() != source.rows.len() {
            return Err(DataTableCellError::SourceInvariant);
        }
        for (index, (before, after)) in source.rows.iter().zip(&reopened.rows).enumerate() {
            let expected_key = if index == row_index {
                new_key
            } else {
                &before.source_key
            };
            if after.source_key != expected_key
                || before.cells.len() != after.cells.len()
                || before.cells.iter().zip(&after.cells).any(|(left, right)| {
                    left.field_id != right.field_id
                        || left.field_name != right.field_name
                        || !cell_value::equal(&left.value, &right.value)
                })
            {
                return Err(DataTableCellError::SourceInvariant);
            }
        }
        Ok(Some(candidate))
    }
}
