use std::collections::HashMap;

use zircon_runtime::asset::DataAsset;
use zircon_runtime_interface::reflect::ReflectTypeInfo;

use super::super::{DataTableCellCodec, DataTableCellError, DataTableSourceAdapter};
use super::source;

impl DataTableCellCodec {
    /// Prepares one batch deletion by exact source keys, retaining survivor order and raw bytes.
    ///
    /// The whole source and every requested key are validated before any candidate is created;
    /// missing/duplicate keys reject the batch. An empty selection is a validated no-op. This
    /// does not authorize dependent-reference deletion or publish history, selection, focus,
    /// dirty state or I/O. The original host still owns admission and one Document/history/CAS.
    pub fn prepare_rows_remove(
        asset: &DataAsset,
        row_type: Option<&ReflectTypeInfo>,
        key_field: &str,
        row_keys: &[&str],
    ) -> Result<Option<DataAsset>, DataTableCellError> {
        let original = DataTableSourceAdapter::read(asset, row_type, key_field)?;
        let indices: HashMap<_, _> = original
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| (row.source_key.as_str(), index))
            .collect();
        let mut removed = vec![false; original.rows.len()];
        for key in row_keys {
            let &index = indices
                .get(*key)
                .ok_or_else(|| DataTableCellError::MissingRow((*key).into()))?;
            if removed[index] {
                return Err(DataTableCellError::DuplicateRowSelection((*key).into()));
            }
            removed[index] = true;
        }
        if row_keys.is_empty() {
            return Ok(None);
        }
        let spans = source::spans(&asset.text)?;
        if spans.len() != original.rows.len() {
            return Err(DataTableCellError::SourceInvariant);
        }
        let mut ranges = Vec::new();
        let mut cursor = 0;
        while cursor < removed.len() {
            if !removed[cursor] {
                cursor += 1;
                continue;
            }
            let first = cursor;
            while cursor < removed.len() && removed[cursor] {
                cursor += 1;
            }
            // A non-tail run owns its following separator. A tail run instead owns the
            // separator after the last survivor. Removing all rows preserves both brackets
            // and outer whitespace. The resulting ranges are disjoint in original offsets.
            let range = if cursor < spans.len() {
                spans[first].start..spans[cursor].start
            } else if first > 0 {
                spans[first - 1].end..spans[cursor - 1].end
            } else {
                spans[first].start..spans[cursor - 1].end
            };
            ranges.push(range);
        }
        if ranges.windows(2).any(|pair| pair[0].end > pair[1].start) {
            return Err(DataTableCellError::SourceInvariant);
        }
        let mut text = asset.text.clone();
        for range in ranges.into_iter().rev() {
            text.replace_range(range, "");
        }
        let expected: Vec<_> = original
            .rows
            .iter()
            .zip(&removed)
            .filter_map(|(row, removed)| (!removed).then_some(row))
            .collect();
        source::verify(asset, text, row_type, key_field, &original, &expected).map(Some)
    }
}
