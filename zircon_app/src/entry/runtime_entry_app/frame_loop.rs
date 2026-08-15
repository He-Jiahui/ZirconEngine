use super::RuntimeEntryApp;
use winit::event_loop::ActiveEventLoop;

impl RuntimeEntryApp {
    pub(super) fn pump_frame_loop(&mut self, event_loop: &dyn ActiveEventLoop) {
        let now = std::time::Instant::now();
        let should_pump = self.frame_cadence.take_frame_request(now);
        if should_pump {
            zircon_runtime::profile_counter!("app", "runtime_entry.frame_pump", 1_u8);
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
            zircon_runtime::profile_counter!("app", "runtime_entry.runtime_tick", 1_u8);
            let wake_host = self
                .frame_cadence
                .apply_runtime_demand(std::time::Instant::now(), demand);
            if wake_host {
                self.session.wake_host();
            }
            if !self.apply_runtime_host_requests(event_loop) {
                return;
            }
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
                self.frame_cadence.record_redraw_request();
                zircon_runtime::profile_counter!("app", "runtime_entry.redraw_request", 1_u8);
            }
        } else {
            zircon_runtime::profile_counter!("app", "runtime_entry.frame_pump_suppressed", 1_u8);
        }
        self.apply_event_loop_policy(event_loop);
    }

    pub(super) fn request_runtime_frame(&mut self) {
        self.frame_cadence.request_frame();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn frame_pump_publishes_only_the_final_control_flow() {
        let source = include_str!("frame_loop.rs");
        let pump_start = source
            .find("pub(super) fn pump_frame_loop")
            .expect("frame pump owner");
        let request_start = source[pump_start..]
            .find("pub(super) fn request_runtime_frame")
            .map(|offset| pump_start + offset)
            .expect("frame request owner after pump");
        let pump_body = &source[pump_start..request_start];

        assert_eq!(
            pump_body
                .matches("self.apply_event_loop_policy(event_loop);")
                .count(),
            1,
            "each frame pump must publish only its final control flow",
        );
    }

    #[test]
    fn frame_pump_keeps_the_p1_cadence_measurement_points() {
        let source = include_str!("frame_loop.rs");

        for name in [
            "runtime_entry.frame_pump",
            "runtime_entry.frame_pump_suppressed",
            "runtime_entry.runtime_tick",
            "runtime_entry.redraw_request",
        ] {
            assert!(
                source.contains(name),
                "P1 cadence reporting must retain the `{name}` counter"
            );
        }
    }
}
