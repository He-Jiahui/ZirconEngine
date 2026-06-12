use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::write_error;

use super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(super) fn pump_frame_loop(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.apply_event_loop_policy(event_loop);
        #[cfg(feature = "gamepad-gilrs")]
        self.poll_gamepads(event_loop);
        if let Err(error) = self.session.tick_frame() {
            write_error(
                "runtime_frame_loop",
                format!("runtime_tick_frame_failed error={error}"),
            );
            event_loop.exit();
            return;
        }
        if !self.apply_runtime_host_requests(event_loop) {
            return;
        }
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}
