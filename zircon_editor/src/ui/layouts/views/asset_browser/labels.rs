use crate::ui::workbench::snapshot::AssetItemSnapshot;

pub(super) fn asset_state_label(asset: &AssetItemSnapshot) -> &'static str {
    if asset.diagnostics.is_empty() {
        "Ready"
    } else {
        "Diagnostics"
    }
}
