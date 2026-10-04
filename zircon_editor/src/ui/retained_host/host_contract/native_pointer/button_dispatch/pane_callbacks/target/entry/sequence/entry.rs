use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::dispatch::UiInputModifiers;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::super::super::routing::PanePointerRoute;
use super::super::super::super::super::super::NativePointerButtonState;
use super::super::input::PaneButtonDispatchInput;
use super::run::dispatch_pane_button_sequence;

pub(in crate::ui::retained_host::host_contract) fn dispatch_pane_button(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    pointer: PanePointerRoute,
    state: NativePointerButtonState,
    button: UiPointerButton,
    button_id: i32,
    modifiers: UiInputModifiers,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    let interaction_damage = if button == UiPointerButton::Primary {
        let (source_id, frame) = match &pointer.target {
            super::super::super::super::super::super::routing::PanePointerTarget::ViewportToolbar { source_control_id, control_frame, .. } => (source_control_id.as_deref(), Some(control_frame)),
            _ => (None, None),
        };
        ui.set_template_button_pointer_state(
            source_id,
            frame,
            state == NativePointerButtonState::Pressed,
        )
    } else {
        None
    };
    let result = dispatch_pane_button_sequence(PaneButtonDispatchInput {
        ui,
        presentation,
        pointer,
        state,
        button,
        button_id,
        modifiers,
        cleared_text_input_frame,
    });
    match interaction_damage {
        Some(damage) => result.merge(NativePointerDispatchResult::region(damage)),
        None => result,
    }
}
