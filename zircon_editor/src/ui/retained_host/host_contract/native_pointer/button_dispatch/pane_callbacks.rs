mod asset_panes;
mod native_panes;
mod template_nodes;
mod viewport;

use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::pane_button_damage::pane_pointer_press_damage_frame;
use super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::super::{
    NativePointerButtonState, HOST_POINTER_DOWN, HOST_POINTER_UP, VIEWPORT_POINTER_DOWN,
    VIEWPORT_POINTER_UP,
};
use asset_panes::{
    dispatch_asset_content_button, dispatch_asset_reference_button, dispatch_asset_tree_button,
};
use native_panes::{dispatch_hierarchy_button, dispatch_welcome_button};
use template_nodes::dispatch_template_node_button;
use viewport::{dispatch_viewport_button, dispatch_viewport_toolbar_button};

pub(super) fn dispatch_pane_button(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    pointer: PanePointerRoute,
    state: NativePointerButtonState,
    button: UiPointerButton,
    button_id: i32,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    let kind = match state {
        NativePointerButtonState::Pressed => VIEWPORT_POINTER_DOWN,
        NativePointerButtonState::Released => VIEWPORT_POINTER_UP,
    };
    let host_kind = match state {
        NativePointerButtonState::Pressed => HOST_POINTER_DOWN,
        NativePointerButtonState::Released => HOST_POINTER_UP,
    };
    let pane_host = ui.global::<PaneSurfaceHostContext>();
    match &pointer.target {
        PanePointerTarget::Hierarchy => {
            dispatch_hierarchy_button(&pane_host, &pointer, state, button, host_kind, button_id)
        }
        PanePointerTarget::Welcome => dispatch_welcome_button(&pane_host, &pointer, state, button),
        PanePointerTarget::AssetTree(mode) => {
            dispatch_asset_tree_button(&pane_host, &pointer, mode.clone(), state, button)
        }
        PanePointerTarget::AssetContent(mode) => {
            dispatch_asset_content_button(
                &pane_host,
                &pointer,
                mode.clone(),
                state,
                button,
                host_kind,
                button_id,
            );
        }
        PanePointerTarget::AssetReference(mode, list_kind) => {
            dispatch_asset_reference_button(
                &pane_host,
                &pointer,
                mode.clone(),
                list_kind.clone(),
                state,
                button,
                host_kind,
                button_id,
            );
        }
        PanePointerTarget::ViewportToolbar(hit) => {
            return dispatch_viewport_toolbar_button(
                &pane_host,
                presentation,
                &pointer,
                hit,
                state,
                button,
                cleared_text_input_frame,
            );
        }
        PanePointerTarget::Viewport(_) => {
            return dispatch_viewport_button(
                &pane_host,
                &pointer,
                kind,
                button_id,
                cleared_text_input_frame,
            );
        }
        PanePointerTarget::TemplateNode(hit) => {
            if let Some(result) = dispatch_template_node_button(
                ui,
                &pane_host,
                hit.clone(),
                state,
                button,
                cleared_text_input_frame.clone(),
            ) {
                return result;
            }
        }
        PanePointerTarget::Console
        | PanePointerTarget::Inspector
        | PanePointerTarget::BrowserAssetDetails
        | PanePointerTarget::UiAsset
        | PanePointerTarget::Other => {
            if state == NativePointerButtonState::Pressed {
                if let Some(frame) = cleared_text_input_frame {
                    return NativePointerDispatchResult::region(frame);
                }
                return NativePointerDispatchResult::idle();
            }
        }
    }
    if state == NativePointerButtonState::Released {
        return NativePointerDispatchResult::region(pointer.frame.clone());
    }
    match pane_pointer_press_damage_frame(presentation, &pointer.frame, cleared_text_input_frame) {
        Some(damage) => NativePointerDispatchResult::region_with_frame_update(damage),
        None => NativePointerDispatchResult::full_frame(),
    }
}
