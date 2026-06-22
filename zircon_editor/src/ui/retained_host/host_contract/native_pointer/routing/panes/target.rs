use crate::ui::retained_host::host_contract::data::PaneData;

use super::super::PanePointerTarget;

pub(super) fn pane_pointer_target_for_kind(
    pane: &PaneData,
    surface_key: Option<&str>,
) -> PanePointerTarget {
    match pane.kind.as_str() {
        "Hierarchy" => PanePointerTarget::Hierarchy,
        "Welcome" => PanePointerTarget::Welcome,
        "Console" => PanePointerTarget::Console,
        "Inspector" => PanePointerTarget::Inspector,
        "Assets" => PanePointerTarget::AssetTree("activity".into()),
        "AssetBrowser" => PanePointerTarget::AssetTree("browser".into()),
        "Scene" | "Game" => PanePointerTarget::Viewport(surface_key.unwrap_or("document").into()),
        "UiAssetEditor" => PanePointerTarget::UiAsset,
        _ => PanePointerTarget::Other,
    }
}
