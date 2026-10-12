use crate::ui::retained_host::host_contract::data::PaneData;

use super::super::{PaneAssetSurface, PanePointerTarget};

pub(super) fn pane_pointer_target_for_kind<'a>(
    pane: &PaneData,
    surface_key: Option<&'a str>,
) -> PanePointerTarget<'a> {
    match pane.kind.as_str() {
        "Hierarchy" => PanePointerTarget::Hierarchy,
        "Welcome" => PanePointerTarget::Welcome,
        "Console" => PanePointerTarget::Console,
        "Inspector" => PanePointerTarget::Inspector,
        "Assets" => PanePointerTarget::AssetTree(PaneAssetSurface::Activity),
        "AssetBrowser" => PanePointerTarget::AssetTree(PaneAssetSurface::Browser),
        "Scene" => PanePointerTarget::SceneViewport(surface_key.unwrap_or("document")),
        "Game" => PanePointerTarget::GameViewport(surface_key.unwrap_or("document")),
        "UiAssetEditor" => PanePointerTarget::UiAsset,
        _ => PanePointerTarget::Other,
    }
}
