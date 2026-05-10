use crate::ui::workbench::snapshot::AssetSelectionSnapshot;

pub(super) fn asset_details_sections_len(selection: &AssetSelectionSnapshot) -> usize {
    if selection.diagnostics.is_empty() {
        5
    } else {
        6
    }
}
