use winit::event::{ButtonSource, ElementState, MouseButton, MouseScrollDelta};
use winit::window::Window;
use zircon_runtime_interface::ui::dispatch::UiInputEventMetadata;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::native_pointer::NativePointerButtonState;

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract) fn sync_ime_allowed(&mut self) {
        let allowed = self.host.text_input_focus_active();
        if self.ime_allowed == allowed {
            return;
        }
        if let Some(window) = self.window.as_ref() {
            set_window_ime_allowed(window.as_ref(), allowed);
            self.ime_allowed = allowed;
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn next_input_metadata(
        &mut self,
    ) -> UiInputEventMetadata {
        let metadata = super::super::metadata::native_input_metadata(self.next_input_sequence);
        self.next_input_sequence = self.next_input_sequence.saturating_add(1);
        metadata
    }
}

pub(in crate::ui::retained_host::host_contract) fn pointer_button(
    button: ButtonSource,
) -> Option<UiPointerButton> {
    match button.mouse_button() {
        Some(MouseButton::Left) => Some(UiPointerButton::Primary),
        Some(MouseButton::Right) => Some(UiPointerButton::Secondary),
        Some(MouseButton::Middle) => Some(UiPointerButton::Middle),
        _ => None,
    }
}

pub(in crate::ui::retained_host::host_contract) fn pointer_button_state(
    state: ElementState,
) -> Option<NativePointerButtonState> {
    match state {
        ElementState::Pressed => Some(NativePointerButtonState::Pressed),
        ElementState::Released => Some(NativePointerButtonState::Released),
    }
}

pub(in crate::ui::retained_host::host_contract) fn scroll_delta(delta: MouseScrollDelta) -> f32 {
    match delta {
        MouseScrollDelta::LineDelta(_, y) => y,
        MouseScrollDelta::PixelDelta(position) => position.y as f32 * 0.1,
    }
}

#[allow(deprecated)]
fn set_window_ime_allowed(window: &dyn Window, allowed: bool) {
    window.set_ime_allowed(allowed);
}
