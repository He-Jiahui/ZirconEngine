use super::RuntimeEntryApp;
use winit::event_loop::ActiveEventLoop;

impl RuntimeEntryApp {
    pub(super) fn pump_frame_loop(&mut self, event_loop: &dyn ActiveEventLoop) {
        let now = std::time::Instant::now();
        let should_pump = self.frame_cadence.take_frame_request(now);
        self.apply_event_loop_policy(event_loop);
        if !should_pump {
            return;
        }
        #[cfg(feature = "gamepad-gilrs")]
        self.poll_gamepads(event_loop);
        let demand = match self.session.tick_frame() {
            Ok(demand) => demand,
            Err(error) => {
                self.report_fatal_failure(
                    "runtime_frame_loop",
                    "runtime_session",
                    format!("frame tick failed: {error}"),
                    "verify the runtime project and restart zircon_runtime",
                );
                event_loop.exit();
                return;
            }
        };
        let wake_host = self
            .frame_cadence
            .apply_runtime_demand(std::time::Instant::now(), demand);
        self.apply_event_loop_policy(event_loop);
        if wake_host {
            self.session.wake_host();
        }
        if !self.apply_runtime_host_requests(event_loop) {
            return;
        }
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
            self.frame_cadence.record_redraw_request();
        }
    }

    pub(super) fn request_runtime_frame(&mut self) {
        self.frame_cadence.request_frame();
    }
}
