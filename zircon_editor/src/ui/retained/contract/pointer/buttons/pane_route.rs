use crate::ui::retained_host::host_contract::data::{FrameRect, HostPresentationGeneration};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::dispatch::UiInputModifiers;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::routing::route_pointer_to_pane;
use super::super::NativePointerButtonState;
use super::pane_callbacks::dispatch_pane_button;

pub(super) fn dispatch_pane_route_button(
    ui: &UiHostWindow,
    generation: &HostPresentationGeneration,
    state: NativePointerButtonState,
    button: UiPointerButton,
    button_id: i32,
    modifiers: UiInputModifiers,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    let presentation = generation.structure();
    let pointer = route_pointer_to_pane(presentation, generation.pane_interaction_state(), x, y);
    if ui.dispatch_game_rmb_pane(generation, pointer.as_ref(), state, button, x, y) {
        return Some(cleared_text_input_frame.map_or_else(
            NativePointerDispatchResult::idle,
            NativePointerDispatchResult::region,
        ));
    }
    pointer.map(|pointer| {
        dispatch_pane_button(
            ui,
            presentation,
            pointer,
            state,
            button,
            button_id,
            modifiers,
            cleared_text_input_frame,
        )
    })
}
