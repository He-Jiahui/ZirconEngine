use winit::event::KeyEvent;
use winit::keyboard::ModifiersState;
use zircon_runtime_interface::ui::{
    dispatch::UiInputModifiers,
    window::{UiWindowInputContext, UiWindowPlatformInputEvent},
};

use super::super::keyboard_map::{
    dom_key_code, keyboard_state, logical_key_name, native_scan_code, physical_key_name,
};

pub fn translate_winit_modifiers(state: ModifiersState) -> UiInputModifiers {
    UiInputModifiers {
        shift: state.shift_key(),
        control: state.control_key(),
        alt: state.alt_key(),
        super_key: state.meta_key(),
        caps_lock: false,
        num_lock: false,
    }
}

pub(super) fn translate_keyboard_event(
    context: UiWindowInputContext,
    event: &KeyEvent,
    synthetic: bool,
) -> UiWindowPlatformInputEvent {
    let mut context = context;
    context.metadata.synthetic = synthetic;

    UiWindowPlatformInputEvent::keyboard(
        context,
        keyboard_state(event.state, event.repeat),
        dom_key_code(&event.logical_key),
        native_scan_code(event.physical_key),
        physical_key_name(event.physical_key),
        logical_key_name(&event.logical_key),
        event.text.as_ref().map(ToString::to_string),
    )
}
