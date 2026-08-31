use winit::event::KeyEvent;
use zircon_runtime_interface::ui::dispatch::UiKeyboardInputEvent;

use super::super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::globals::UiHostContext;

pub(super) fn dispatch_unhandled_keyboard_input(
    window: &UiHostWindow,
    _event: &KeyEvent,
    keyboard: Option<UiKeyboardInputEvent>,
) {
    if let Some(keyboard) = keyboard {
        window
            .global::<UiHostContext>()
            .invoke_unhandled_keyboard_input(keyboard);
    }
}
