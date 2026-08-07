mod present;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::redraw::{
    HostRedrawRequest, NativePointerDispatchResult,
};
use crate::ui::retained_host::ui_perf::enter_ui_perf_scenario;
#[cfg(feature = "profiling")]
use crate::ui::retained_host::ui_perf::{record_ui_perf_counter, UiPerfCounter};
use present::present_redraw;

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract) fn dispatch_pointer_result(
        &mut self,
        result: NativePointerDispatchResult,
    ) {
        let redraw = result.redraw();
        #[cfg(feature = "profiling")]
        if let Some(started_at) = self.pending_input_started_at.take() {
            if redraw.request_redraw() {
                record_ui_perf_counter(
                    redraw.scenario(),
                    UiPerfCounter::InputToDamageUs,
                    started_at.elapsed().as_secs_f64() * 1_000_000.0,
                );
                if self.pending_damage_started_at.is_none() {
                    self.pending_damage_started_at = Some(std::time::Instant::now());
                }
            }
        }
        if self.queue_redraw(redraw) {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    pub(super) fn begin_input_latency_sample(&mut self) {
        #[cfg(feature = "profiling")]
        {
            self.pending_input_started_at = Some(std::time::Instant::now());
        }
    }

    pub(super) fn cancel_input_latency_sample(&mut self) {
        #[cfg(feature = "profiling")]
        {
            self.pending_input_started_at = None;
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn queue_redraw(
        &mut self,
        redraw: HostRedrawRequest,
    ) -> bool {
        if !redraw.request_redraw() {
            return false;
        }
        let existing = std::mem::replace(&mut self.pending_redraw, HostRedrawRequest::None);
        let should_schedule = !existing.request_redraw();
        self.pending_redraw = existing.merge(redraw);
        should_schedule
    }

    pub(in crate::ui::retained_host::host_contract) fn drain_external_redraw_request(&mut self) {
        let redraw = self.host.take_external_redraw();
        if self.queue_redraw(redraw) {
            if let Some(window) = self.window.as_ref() {
                schedule_native_redraw(window.as_ref());
            }
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn redraw_requested_impl(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let redraw = self.take_pending_redraw();
        if !redraw.request_redraw() {
            return;
        }
        let redraw_scenario = redraw.scenario();
        let redraw_scenario_guard = enter_ui_perf_scenario(redraw_scenario);
        if redraw.requires_frame_update() {
            self.host.request_frame_update();
        }
        let present_scenario = self
            .host
            .take_completed_frame_update_scenario()
            .unwrap_or(redraw_scenario);
        drop(redraw_scenario_guard);
        present_redraw(
            self,
            event_loop,
            redraw.damage_region().cloned(),
            present_scenario,
        );
    }

    fn take_pending_redraw(&mut self) -> HostRedrawRequest {
        std::mem::replace(&mut self.pending_redraw, HostRedrawRequest::None)
    }
}

fn schedule_native_redraw(window: &dyn Window) {
    window.request_redraw();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::data::FrameRect;

    #[test]
    fn redraw_queue_schedules_only_on_empty_to_pending_transition() {
        let host = crate::ui::retained_host::host_contract::window::UiHostWindow::new()
            .expect("host window");
        let mut event_loop = UiHostWindowEventLoop::new(host);
        let startup = event_loop.take_pending_redraw();
        assert!(startup.request_redraw());

        assert!(
            event_loop.queue_redraw(HostRedrawRequest::region(FrameRect {
                x: 4.0,
                y: 8.0,
                width: 20.0,
                height: 16.0,
            }))
        );
        assert!(
            !event_loop.queue_redraw(HostRedrawRequest::region(FrameRect {
                x: 40.0,
                y: 48.0,
                width: 12.0,
                height: 10.0,
            }))
        );
    }
}
