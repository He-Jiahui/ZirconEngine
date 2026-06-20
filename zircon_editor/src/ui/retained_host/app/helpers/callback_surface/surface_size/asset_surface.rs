use crate::ui::workbench::snapshot::ViewContentKind;

pub(super) fn asset_surface_kind(surface_mode: &str) -> Option<ViewContentKind> {
    match surface_mode {
        "activity" => Some(ViewContentKind::Assets),
        "browser" => Some(ViewContentKind::AssetBrowser),
        _ => None,
    }
}
