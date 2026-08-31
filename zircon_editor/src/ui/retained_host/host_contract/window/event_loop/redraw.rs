mod present;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use std::time::{Duration, Instant};

use super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::redraw::{
    HostRedrawRequest, NativePointerDispatchResult,
};
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, record_ui_perf_counter, UiPerfCounter, UiPerfScenario,
};
use present::present_redraw;

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract) fn dispatch_pointer_result(
        &mut self,
        result: NativePointerDispatchResult,
    ) {
        let redraw = result.redraw().into_interactive_frame_update();
        self.finish_input_outcome(&redraw);
        if self.queue_redraw(redraw) {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
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
        let redraw = self.take_redraw_for_present();
        if !redraw.request_redraw() {
            return;
        }
        let presenter_resize_pending = self.pending_presenter_resize.is_some();
        if presenter_resize_pending && !self.apply_pending_presenter_resize(event_loop) {
            return;
        }
        let redraw_scenario = if presenter_resize_pending {
            crate::ui::retained_host::ui_perf::UiPerfScenario::WindowResize
        } else {
            redraw.scenario()
        };
        let redraw_scenario_guard = enter_ui_perf_scenario(redraw_scenario);
        if redraw.requires_frame_update() {
            if redraw.prefers_interactive_frame_update() {
                self.host.request_interactive_frame_update();
            } else {
                self.host.request_frame_update();
            }
        }
        let present_scenario = self
            .host
            .take_completed_frame_update_scenario()
            .unwrap_or(redraw_scenario);
        drop(redraw_scenario_guard);
        if !redraw.requires_present() {
            return;
        }
        record_damage_region_metrics(&redraw, present_scenario);
        present_redraw(self, event_loop, redraw, present_scenario);
    }

    fn take_pending_redraw(&mut self) -> HostRedrawRequest {
        std::mem::replace(&mut self.pending_redraw, HostRedrawRequest::None)
    }

    pub(super) fn defer_surface_present_retry(&mut self, redraw: HostRedrawRequest, now: Instant) {
        let existing = std::mem::replace(
            &mut self.pending_surface_present_retry,
            HostRedrawRequest::None,
        );
        self.pending_surface_present_retry = existing.merge(redraw);
        let delay = surface_present_retry_delay(self.surface_present_retry_attempt);
        zircon_runtime::profile_counter!(
            "editor",
            "ui.surface.retry_backoff_ms",
            delay.as_secs_f64() * 1_000.0
        );
        self.pending_surface_present_retry_deadline = Some(now + delay);
        self.surface_present_retry_attempt = self.surface_present_retry_attempt.saturating_add(1);
    }

    pub(super) fn reset_surface_present_retry_backoff(&mut self) {
        self.surface_present_retry_attempt = 0;
    }

    pub(super) fn take_due_surface_present_retry(&mut self, now: Instant) -> HostRedrawRequest {
        let Some(deadline) = self.pending_surface_present_retry_deadline else {
            return HostRedrawRequest::None;
        };
        if now < deadline {
            return HostRedrawRequest::None;
        }
        self.take_surface_present_retry()
    }

    fn take_redraw_for_present(&mut self) -> HostRedrawRequest {
        let queued = self.take_pending_redraw();
        queued.merge(self.take_surface_present_retry())
    }

    fn take_surface_present_retry(&mut self) -> HostRedrawRequest {
        self.pending_surface_present_retry_deadline = None;
        std::mem::replace(
            &mut self.pending_surface_present_retry,
            HostRedrawRequest::None,
        )
    }
}

fn record_damage_region_metrics(redraw: &HostRedrawRequest, scenario: UiPerfScenario) {
    let Some(metrics) = redraw.damage_region_metrics() else {
        return;
    };
    record_ui_perf_counter(
        scenario,
        UiPerfCounter::RedrawDamageRectCount,
        metrics.rect_count as f64,
    );
    record_ui_perf_counter(
        scenario,
        UiPerfCounter::RedrawDamageSourceRectCount,
        metrics.source_rect_count as f64,
    );
    record_ui_perf_counter(
        scenario,
        UiPerfCounter::RedrawDamageSimplificationCount,
        metrics.simplification_count as f64,
    );
    record_ui_perf_counter(
        scenario,
        UiPerfCounter::RedrawDamageRepresentedArea,
        metrics.represented_area,
    );
    record_ui_perf_counter(
        scenario,
        UiPerfCounter::RedrawDamageBoundingArea,
        metrics.bounding_area,
    );
    record_ui_perf_counter(
        scenario,
        UiPerfCounter::RedrawDamageBoundingOverdrawArea,
        metrics.bounding_overdraw_area,
    );
}

fn surface_present_retry_delay(attempt: u8) -> Duration {
    let multiplier = 1_u32 << u32::from(attempt.min(5));
    super::SURFACE_PRESENT_RETRY_BASE_DELAY
        .saturating_mul(multiplier)
        .min(super::SURFACE_PRESENT_RETRY_MAX_DELAY)
}

fn schedule_native_redraw(window: &dyn Window) {
    window.request_redraw();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::data::FrameRect;
    use std::time::Duration;

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

    #[test]
    fn native_resize_configures_the_latest_surface_before_the_frame_update() {
        let source = include_str!("redraw.rs");
        let function = source
            .split("fn redraw_requested_impl")
            .nth(1)
            .and_then(|body| body.split("fn take_pending_redraw").next())
            .expect("redraw implementation");
        let resize = function
            .find("self.apply_pending_presenter_resize(event_loop)")
            .expect("pending swapchain resize should configure the latest size");
        let frame_update = function
            .find("redraw.requires_frame_update()")
            .expect("interactive resize must publish retained geometry");
        let present = function
            .find("present_redraw(")
            .expect("interactive resize should still present the retained snapshot");

        assert!(resize < frame_update);
        assert!(frame_update < present);
        assert!(function.contains("pending_presenter_resize.is_some()"));
        assert!(!function.contains("native_resize_present"));
    }

    #[test]
    fn retryable_surface_present_uses_bounded_exponential_backoff() {
        assert_eq!(surface_present_retry_delay(0), Duration::from_millis(8));
        assert_eq!(surface_present_retry_delay(1), Duration::from_millis(16));
        assert_eq!(surface_present_retry_delay(4), Duration::from_millis(128));
        assert_eq!(surface_present_retry_delay(5), Duration::from_millis(250));
        assert_eq!(
            surface_present_retry_delay(u8::MAX),
            Duration::from_millis(250)
        );
    }

    #[test]
    fn retryable_surface_present_is_deferred_outside_the_native_redraw_queue() {
        let host = crate::ui::retained_host::host_contract::window::UiHostWindow::new()
            .expect("host window");
        let mut event_loop = UiHostWindowEventLoop::new(host);
        let _startup = event_loop.take_pending_redraw();
        let now = Instant::now();
        let damage = FrameRect {
            x: 4.0,
            y: 8.0,
            width: 32.0,
            height: 16.0,
        };

        event_loop.defer_surface_present_retry(HostRedrawRequest::region(damage.clone()), now);

        assert!(!event_loop.pending_redraw.request_redraw());
        assert_eq!(
            event_loop.pending_surface_present_retry_deadline,
            Some(now + Duration::from_millis(8))
        );
        assert!(!event_loop
            .take_due_surface_present_retry(now + Duration::from_millis(7))
            .request_redraw());
        let retry = event_loop.take_due_surface_present_retry(now + Duration::from_millis(8));
        assert_eq!(retry.damage_region(), Some(&damage));
        assert!(!retry.requires_frame_update());
        assert_eq!(event_loop.pending_surface_present_retry_deadline, None);
    }

    #[test]
    fn real_redraw_consumes_a_deferred_retry_and_success_resets_backoff() {
        let host = crate::ui::retained_host::host_contract::window::UiHostWindow::new()
            .expect("host window");
        let mut event_loop = UiHostWindowEventLoop::new(host);
        let _startup = event_loop.take_pending_redraw();
        let now = Instant::now();
        event_loop.defer_surface_present_retry(
            HostRedrawRequest::full_frame_for_scenario(
                crate::ui::retained_host::ui_perf::UiPerfScenario::WindowResize,
                false,
            ),
            now,
        );
        assert!(
            event_loop.queue_redraw(HostRedrawRequest::region(FrameRect {
                x: 10.0,
                y: 12.0,
                width: 8.0,
                height: 6.0,
            }))
        );

        let merged = event_loop.take_redraw_for_present();
        assert!(merged.requires_present());
        assert_eq!(merged.damage_region(), None);
        assert_eq!(event_loop.pending_surface_present_retry_deadline, None);
        assert_eq!(event_loop.surface_present_retry_attempt, 1);

        event_loop.reset_surface_present_retry_backoff();
        assert_eq!(event_loop.surface_present_retry_attempt, 0);
    }
}
