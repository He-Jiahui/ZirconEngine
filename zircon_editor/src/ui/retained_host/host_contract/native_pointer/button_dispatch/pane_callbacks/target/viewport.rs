mod body;
mod toolbar;

use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use zircon_runtime_interface::ui::dispatch::UiInputModifiers;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use self::body::dispatch_viewport_body_target_button;
use self::toolbar::dispatch_viewport_toolbar_target_button;
use super::super::super::super::NativePointerButtonState;
use super::super::super::super::routing::{PanePointerRoute, PanePointerTarget};

pub(super) fn dispatch_viewport_pane_target_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    presentation: &HostWindowPresentationData,
    pointer: &PanePointerRoute,
    kind: i32,
    state: NativePointerButtonState,
    button: UiPointerButton,
    button_id: i32,
    modifiers: UiInputModifiers,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    match &pointer.target {
        PanePointerTarget::ViewportToolbar {
            surface_key,
            control_id,
        } => Some(dispatch_viewport_toolbar_target_button(
            pane_host,
            presentation,
            pointer,
            surface_key,
            control_id.as_ref(),
            state,
            button,
            cleared_text_input_frame,
        )),
        PanePointerTarget::Viewport(_) => Some(dispatch_viewport_body_target_button(
            pane_host,
            pointer,
            kind,
            button_id,
            modifiers,
            cleared_text_input_frame,
        )),
        _ => None,
    }
}
