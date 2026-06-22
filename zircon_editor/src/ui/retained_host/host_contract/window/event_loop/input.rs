use winit::window::Window;
use zircon_runtime_interface::ui::dispatch::UiInputEventMetadata;

use super::UiHostWindowEventLoop;

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

#[allow(deprecated)]
fn set_window_ime_allowed(window: &dyn Window, allowed: bool) {
    window.set_ime_allowed(allowed);
}
