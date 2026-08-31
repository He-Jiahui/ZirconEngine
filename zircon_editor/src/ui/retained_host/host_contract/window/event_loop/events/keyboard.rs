use super::super::platform_input::PlatformInputTranslation;
use super::super::platform_input::{platform_keyboard_input, platform_text_input};
use super::super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use winit::event::KeyEvent;

impl UiHostWindowEventLoop {
    pub(super) fn handle_keyboard_input(
        &mut self,
        event: KeyEvent,
        platform_event: PlatformInputTranslation,
    ) {
        self.begin_input_outcome(platform_event.sequence);
        self.host
            .global::<UiHostContext>()
            .invoke_workbench_input_activity();
        let result = self
            .host
            .dispatch_keyboard_event(&event, platform_keyboard_input(platform_event.event));
        self.dispatch_pointer_result(result);
        self.sync_ime_allowed();
    }

    pub(super) fn handle_ime_input(&mut self, platform_event: PlatformInputTranslation) {
        self.begin_input_outcome(platform_event.sequence);
        self.host
            .global::<UiHostContext>()
            .invoke_workbench_input_activity();
        if let Some(text) = platform_text_input(platform_event.event) {
            let result = self.host.dispatch_focused_text_insert(&text);
            self.dispatch_pointer_result(result);
        } else {
            self.reject_input_outcome();
        }
    }
}
