use crate::ui::retained_host::host_contract::data::HostPaneInteractionStateData;
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::super::{
    asset::dispatch_asset_pane_move, native::dispatch_native_pane_move,
    passive::clear_passive_pane_move_hover, template::dispatch_template_pane_move,
    viewport::dispatch_viewport_pane_move,
};

pub(super) fn dispatch_pane_pointer_move_target(
    ui: &UiHostWindow,
    pointer: &PanePointerRoute,
    before: &HostPaneInteractionStateData,
) {
    let pane_host = ui.global::<PaneSurfaceHostContext>();
    if !retains_asset_reference_hover(&pointer.target) {
        clear_asset_reference_hover_if_needed(&pane_host, before);
    }
    match &pointer.target {
        PanePointerTarget::Hierarchy | PanePointerTarget::Welcome => {
            dispatch_native_pane_move(&pane_host, pointer)
        }
        PanePointerTarget::AssetTree(_)
        | PanePointerTarget::AssetContent(_)
        | PanePointerTarget::AssetReference(_, _) => dispatch_asset_pane_move(&pane_host, pointer),
        PanePointerTarget::TemplateNode(hit) => dispatch_template_pane_move(ui, hit),
        PanePointerTarget::SceneViewport(_) | PanePointerTarget::GameViewport(_) => {
            dispatch_viewport_pane_move(ui, &pane_host, pointer)
        }
        PanePointerTarget::Console
        | PanePointerTarget::Inspector
        | PanePointerTarget::BrowserAssetDetails
        | PanePointerTarget::ViewportToolbar { .. }
        | PanePointerTarget::UiAsset
        | PanePointerTarget::Other => clear_passive_pane_move_hover(ui),
    }
}

fn clear_asset_reference_hover_if_needed(
    pane_host: &PaneSurfaceHostContext<'_>,
    state: &HostPaneInteractionStateData,
) {
    if state.activity_asset_references_hovered_index >= 0
        || state.activity_asset_used_by_hovered_index >= 0
    {
        pane_host.invoke_asset_reference_pointer_left("activity".into());
    }
    if state.browser_asset_references_hovered_index >= 0
        || state.browser_asset_used_by_hovered_index >= 0
    {
        pane_host.invoke_asset_reference_pointer_left("browser".into());
    }
}

fn retains_asset_reference_hover(target: &PanePointerTarget<'_>) -> bool {
    matches!(target, PanePointerTarget::AssetReference(_, _))
}

#[cfg(test)]
mod tests {
    use super::retains_asset_reference_hover;
    use crate::ui::retained_host::host_contract::native_pointer::routing::{
        PaneAssetReferenceList, PaneAssetSurface, PanePointerTarget,
    };

    #[test]
    fn only_asset_reference_targets_retain_hover() {
        assert!(!retains_asset_reference_hover(
            &PanePointerTarget::AssetContent(PaneAssetSurface::Browser)
        ));
        assert!(!retains_asset_reference_hover(
            &PanePointerTarget::BrowserAssetDetails
        ));
        assert!(retains_asset_reference_hover(
            &PanePointerTarget::AssetReference(
                PaneAssetSurface::Browser,
                PaneAssetReferenceList::References,
            )
        ));
    }
}
