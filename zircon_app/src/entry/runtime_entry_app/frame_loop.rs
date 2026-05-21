use winit::event_loop::ActiveEventLoop;

use super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(super) fn pump_frame_loop(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.apply_event_loop_policy(event_loop);
        #[cfg(feature = "gamepad-gilrs")]
        self.poll_gamepads(event_loop);
        if self.session.tick_frame().is_err() {
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
