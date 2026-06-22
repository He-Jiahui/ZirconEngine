use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::routing::route_pointer_to_pane;
use super::super::NativePointerButtonState;
use super::pane_callbacks::dispatch_pane_button;

pub(super) fn dispatch_pane_route_button(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    state: NativePointerButtonState,
    button: UiPointerButton,
    button_id: i32,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    route_pointer_to_pane(presentation, x, y).map(|pointer| {
        dispatch_pane_button(
            ui,
            presentation,
            pointer,
            state,
            button,
            button_id,
            cleared_text_input_frame,
        )
    })
}
