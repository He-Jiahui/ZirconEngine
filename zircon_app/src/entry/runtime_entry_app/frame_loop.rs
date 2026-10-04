//! Winit about_to_wait 的宿主帧泵；连接 cadence、动态 Runtime tick、宿主请求和重绘。
//! 每轮只发布最终事件循环控制流，是否真正 tick 由 cadence 判定。

use super::RuntimeEntryApp;
use winit::event_loop::ActiveEventLoop;

impl RuntimeEntryApp {
    /// 生命周期准入后调用；先由 cadence 决定 tick，随后处理宿主请求并安排重绘。
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
#[path = "tests/frame_loop.rs"]
mod tests;
