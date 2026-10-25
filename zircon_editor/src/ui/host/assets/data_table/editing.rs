use std::collections::HashSet;

use zircon_runtime::asset::DataAsset;
use zircon_runtime_interface::reflect::ReflectTypeInfo;

use super::{
    cell_value, raw_source, DataTableCellEdit, DataTableCellError, DataTableSourceAdapter,
    DataTableSourceError,
};

pub struct DataTableCellCodec;

impl DataTableCellCodec {
    /// Prepares one complete in-memory candidate from one immutable source snapshot.
    ///
    /// The original document host must still authorize the actual asset/schema association,
    /// current view/publication/revision, retention budget and one history/CAS transaction.
    /// A matching cell value cannot prove those conditions or reject document ABA by itself.
    /// This function never writes a source, installs a document, performs I/O or invokes a callback.
    /// `None` is a true no-op; a candidate preserves the old URI/format and every unedited byte.
    pub fn prepare(
        asset: &DataAsset,
        row_type: Option<&ReflectTypeInfo>,
        key_field: &str,
        edits: &[DataTableCellEdit],
    ) -> Result<Option<DataAsset>, DataTableCellError> {
        let source = DataTableSourceAdapter::read(asset, row_type, key_field)?;
        let raw_rows = raw_source::rows(&asset.text)?;
        let mut seen = HashSet::new();
        let mut replacements = Vec::new();
        for edit in edits {
            if !seen.insert((edit.row_key.as_str(), edit.field_id)) {
                return Err(DataTableCellError::DuplicateEdit {
                    row: edit.row_key.clone(),
                    field_id: edit.field_id,
                });
            }
            let row_index = source
                .rows
                .iter()
                .position(|row| row.source_key == edit.row_key)
                .ok_or_else(|| DataTableCellError::MissingRow(edit.row_key.clone()))?;
            let column_index = source
                .columns
                .iter()
                .position(|field| field.id == edit.field_id)
                .ok_or(DataTableCellError::UnknownField(edit.field_id))?;
            let field = &source.columns[column_index];
            if !field.editable || !field.editor_visible {
                return Err(DataTableCellError::FieldNotEditable(edit.field_id));
            }
            let current = &source.rows[row_index].cells[column_index].value;
            if !cell_value::equal(current, &edit.expected) {
                return Err(DataTableCellError::ExpectedValueMismatch {
                    row: edit.row_key.clone(),
                    field_id: edit.field_id,
                });
            }
            let text = cell_value::encode(field, &edit.value).map_err(|reason| {
                DataTableCellError::InvalidValue {
                    row: edit.row_key.clone(),
                    field_id: edit.field_id,
                    reason,
                }
            })?;
            if cell_value::equal(current, &edit.value) {
                continue;
            }
            let raw = raw_rows[row_index]
                .0
                .iter()
                .find(|(name, _)| name == &field.name)
                .map(|(_, value)| *value)
                .ok_or(DataTableCellError::SourceInvariant)?;
            let range =
                raw_source::span(&asset.text, raw).ok_or(DataTableCellError::SourceInvariant)?;
            replacements.push((range, text));
        }
        if replacements.is_empty() {
            return Ok(None);
        }
        replacements.sort_by_key(|(range, _)| range.start);
        if replacements
            .windows(2)
            .any(|pair| pair[0].0.end > pair[1].0.start)
        {
            return Err(DataTableCellError::SourceInvariant);
        }
        let mut candidate = asset.clone();
        // Descending original spans preserve offsets after variable-length UTF-8 replacements.
        for (range, text) in replacements.into_iter().rev() {
            candidate.text.replace_range(range, &text);
        }
        candidate.canonical_json = serde_json::from_str(&candidate.text)
            .map_err(|error| DataTableSourceError::Encoding(error.to_string()))?;
        let reopened = DataTableSourceAdapter::read(&candidate, row_type, key_field)?;
        for edit in edits {
            let current = reopened
                .rows
                .iter()
                .find(|row| row.source_key == edit.row_key)
                .and_then(|row| row.cells.iter().find(|cell| cell.field_id == edit.field_id))
                .ok_or(DataTableCellError::SourceInvariant)?;
            if !cell_value::equal(&current.value, &edit.value) {
                return Err(DataTableCellError::SourceInvariant);
            }
        }
        Ok(Some(candidate))
    }
}
