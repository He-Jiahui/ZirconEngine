use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::ZrRuntimeEventV1;

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_window_focus_changed(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        focused: bool,
    ) {
        let native_source_active = self.ime_composition_producer.is_some();
        if !focused && native_source_active {
            // A failed native cancellation still retires the producer below.  The lifecycle
            // background event must remain observable so a runtime-side failure cannot strand the
            // app in its foreground state.
            let _native_cancelled = self.handle_native_ime_focus_loss(event_loop);
        }
        if !focused {
            self.clear_native_ime_candidate();
            // A new focus lifetime receives a fresh producer allocator. Advance the app fence so
            // the runtime route does not mistake its first callback token for a replay from the
            // cancelled same-window composition.
            self.advance_native_ime_generation();
        }
        let events = super::super::ime_input::focus_changed_events_with_native_source(
            &mut self.ime_input_admission,
            self.viewport,
            focused,
            native_source_active,
        );
        if self.frame_cadence.set_window_focused(focused) {
            self.request_runtime_frame();
        }
        // Focus transitions still publish the lifecycle event for both paths.  A
        // native V2 source only replaces the legacy IME-disabled cancellation on
        // blur; it must not suppress foreground/background delivery itself.
        dispatch_focus_events(events, |event| {
            self.dispatch_runtime_event(event_loop, event)
        });
    }
}

fn dispatch_focus_events(
    events: [Option<ZrRuntimeEventV1>; 2],
    mut dispatch: impl FnMut(ZrRuntimeEventV1) -> bool,
) {
    for event in events.into_iter().flatten() {
        if !dispatch(event) {
            return;
        }
    }
}

#[cfg(test)]
#[path = "focus/tests/cases.rs"]
mod tests;
