use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::super::{
    asset::dispatch_asset_pane_move, native::dispatch_native_pane_move,
    passive::clear_passive_pane_move_hover, template::dispatch_template_pane_move,
    viewport::dispatch_viewport_pane_move,
};

pub(super) fn dispatch_pane_pointer_move_target(ui: &UiHostWindow, pointer: &PanePointerRoute) {
    let pane_host = ui.global::<PaneSurfaceHostContext>();
    match &pointer.target {
        PanePointerTarget::Hierarchy | PanePointerTarget::Welcome => {
            dispatch_native_pane_move(&pane_host, pointer)
        }
        PanePointerTarget::AssetTree(_)
        | PanePointerTarget::AssetContent(_)
        | PanePointerTarget::AssetReference(_, _) => dispatch_asset_pane_move(&pane_host, pointer),
        PanePointerTarget::TemplateNode(hit) => dispatch_template_pane_move(ui, hit),
        PanePointerTarget::Viewport(_) => dispatch_viewport_pane_move(ui, &pane_host, pointer),
        PanePointerTarget::Console
        | PanePointerTarget::Inspector
        | PanePointerTarget::BrowserAssetDetails
        | PanePointerTarget::ViewportToolbar { .. }
        | PanePointerTarget::UiAsset
        | PanePointerTarget::Other => clear_passive_pane_move_hover(ui),
    }
}
