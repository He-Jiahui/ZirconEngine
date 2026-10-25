use zircon_runtime::asset::DataAsset;
use zircon_runtime_interface::reflect::ReflectTypeInfo;

use super::super::{
    raw_source, DataTableCellCodec, DataTableCellError, DataTableSourceAdapter,
    DataTableSourceError,
};
use super::source;

impl DataTableCellCodec {
    /// Appends a complete source row with a new exact key, preserving authored cell bytes.
    ///
    /// This copies actual source data, including hidden/read-only columns; it does not invent
    /// schema defaults, normalize FName keys or rewrite references. The original document host
    /// must authorize schema/source association, dependency effects, revision, budget and one
    /// Document/history/CAS transaction before publishing. No I/O or callback is performed.
    pub fn prepare_row_duplicate(
        asset: &DataAsset,
        row_type: Option<&ReflectTypeInfo>,
        key_field: &str,
        source_key: &str,
        new_key: &str,
    ) -> Result<DataAsset, DataTableCellError> {
        let original = DataTableSourceAdapter::read(asset, row_type, key_field)?;
        let row_index = original
            .rows
            .iter()
            .position(|row| row.source_key == source_key)
            .ok_or_else(|| DataTableCellError::MissingRow(source_key.into()))?;
        if new_key.is_empty() {
            return Err(DataTableSourceError::InvalidRowKey {
                source_row: original.rows.len(),
                key_field: key_field.into(),
            }
            .into());
        }
        if original.rows.iter().any(|row| row.source_key == new_key) {
            return Err(DataTableSourceError::DuplicateRowKey(new_key.into()).into());
        }
        let spans = source::spans(&asset.text)?;
        if spans.len() != original.rows.len() {
            return Err(DataTableCellError::SourceInvariant);
        }
        let raw_rows = raw_source::rows(&asset.text)?;
        let key = raw_rows[row_index]
            .0
            .iter()
            .find(|(name, _)| name == key_field)
            .map(|(_, value)| *value)
            .ok_or(DataTableCellError::SourceInvariant)?;
        let key_span =
            raw_source::span(&asset.text, key).ok_or(DataTableCellError::SourceInvariant)?;
        let row_span = &spans[row_index];
        if key_span.start < row_span.start || key_span.end > row_span.end {
            return Err(DataTableCellError::SourceInvariant);
        }
        let mut duplicate = asset.text[row_span.clone()].to_owned();
        let encoded = serde_json::to_string(new_key)
            .map_err(|error| DataTableSourceError::Encoding(error.to_string()))?;
        duplicate.replace_range(
            key_span.start - row_span.start..key_span.end - row_span.start,
            &encoded,
        );
        let last = spans.last().ok_or(DataTableCellError::SourceInvariant)?;
        // The existing separator is already valid JSON whitespace and comma. Borrow its style
        // without reserializing any original row or its trailing outer-array whitespace.
        let separator = if spans.len() > 1 {
            &asset.text[spans[spans.len() - 2].end..last.start]
        } else {
            ","
        };
        let mut text = asset.text.clone();
        text.insert_str(last.end, &format!("{separator}{duplicate}"));
        let mut expected_row = original.rows[row_index].clone();
        expected_row.source_key = new_key.into();
        let mut expected: Vec<_> = original.rows.iter().collect();
        expected.push(&expected_row);
        source::verify(asset, text, row_type, key_field, &original, &expected)
    }
}
