use zircon_runtime::asset::DataAsset;
use zircon_runtime_interface::reflect::ReflectTypeInfo;

use super::super::{DataTableCellCodec, DataTableCellError, DataTableSourceAdapter};
use super::source;

impl DataTableCellCodec {
    /// Moves an existing row to the target row's original source position.
    ///
    /// Both identities are exact source keys, not sorted or filtered view indices. Moving
    /// downward places the moved row after the target; moving upward places it before the
    /// target, matching UE MoveRow/OnRowDrop. The caller resolves neighbors or first/last
    /// keys from the same current source to request up/down or extent moves.
    ///
    /// Complete source/schema/cache validation precedes even a self-target no-op. Each row
    /// literal remains byte-exact; separators and outer whitespace retain their positional
    /// slots. No I/O, selection, history or dirty state is published here. The original host
    /// must still authorize the current document/view/source revision and commit one full
    /// candidate with its actual before/after inverse through the existing history/CAS.
    pub fn prepare_row_move(
        asset: &DataAsset,
        row_type: Option<&ReflectTypeInfo>,
        key_field: &str,
        row_key: &str,
        target_row_key: &str,
    ) -> Result<Option<DataAsset>, DataTableCellError> {
        let original = DataTableSourceAdapter::read(asset, row_type, key_field)?;
        let index = |key: &str| {
            original
                .rows
                .iter()
                .position(|row| row.source_key == key)
                .ok_or_else(|| DataTableCellError::MissingRow(key.into()))
        };
        let from = index(row_key)?;
        let to = index(target_row_key)?;
        if from == to {
            return Ok(None);
        }
        let spans = source::spans(&asset.text)?;
        if spans.len() != original.rows.len() {
            return Err(DataTableCellError::SourceInvariant);
        }
        let mut order: Vec<_> = (0..spans.len()).collect();
        let moved = order.remove(from);
        order.insert(to, moved);

        let mut text = String::with_capacity(asset.text.len());
        text.push_str(&asset.text[..spans[0].start]);
        for (slot, row) in order.iter().copied().enumerate() {
            text.push_str(&asset.text[spans[row].clone()]);
            let following = spans
                .get(slot + 1)
                .map_or(asset.text.len(), |span| span.start);
            text.push_str(&asset.text[spans[slot].end..following]);
        }
        let expected: Vec<_> = order.iter().map(|&row| &original.rows[row]).collect();
        source::verify(asset, text, row_type, key_field, &original, &expected).map(Some)
    }
}
