use winit::event::KeyEvent;
use zircon_runtime_interface::ui::window::UiWindowInputPumpEvent;

use super::super::platform_input::{platform_keyboard_input, platform_text_input};
use super::super::UiHostWindowEventLoop;

impl UiHostWindowEventLoop {
    pub(super) fn handle_keyboard_input(
        &mut self,
        event: KeyEvent,
        platform_event: Option<UiWindowInputPumpEvent>,
    ) {
        let result = self
            .host
            .dispatch_keyboard_event(&event, platform_keyboard_input(platform_event));
        self.dispatch_pointer_result(result);
        self.sync_ime_allowed();
    }

    pub(super) fn handle_ime_input(&mut self, platform_event: Option<UiWindowInputPumpEvent>) {
        if let Some(text) = platform_text_input(platform_event) {
            let result = self.host.dispatch_focused_text_insert(&text);
            self.dispatch_pointer_result(result);
        }
    }
}
