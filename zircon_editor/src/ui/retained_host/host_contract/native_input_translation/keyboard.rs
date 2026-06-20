use winit::event::KeyEvent;
use winit::keyboard::ModifiersState;
use zircon_runtime_interface::ui::dispatch::{
    UiInputEvent, UiInputEventMetadata, UiKeyboardInputEvent,
};

use super::keys::{
    keyboard_state, legacy_key_code, logical_key_name, native_scan_code, physical_key_name,
};
use super::modifiers::native_modifiers_to_shared;

pub(crate) fn native_keyboard_event_to_shared_input(
    metadata: UiInputEventMetadata,
    event: &KeyEvent,
    modifiers: ModifiersState,
    synthetic: bool,
) -> UiInputEvent {
    let mut metadata = metadata;
    metadata.modifiers = native_modifiers_to_shared(modifiers);
    metadata.synthetic = synthetic;

    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata,
        state: keyboard_state(event.state, event.repeat),
        key_code: legacy_key_code(&event.logical_key),
        scan_code: native_scan_code(event.physical_key),
        physical_key: physical_key_name(event.physical_key),
        logical_key: logical_key_name(&event.logical_key),
        text: event.text.as_ref().map(ToString::to_string),
    })
}
