mod present;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Once, OnceLock};
use std::time::{Duration, Instant};

use super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::redraw::{
    HostRedrawRequest, NativePointerDispatchResult,
};
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, record_ui_perf_counter, UiPerfCounter, UiPerfScenario,
};
use present::present_redraw;

const SLOW_REDRAW_TRACE_THRESHOLD: Duration = Duration::from_millis(100);
const SLOW_REDRAW_TRACE_LOG_LIMIT: usize = 32;

static SLOW_REDRAW_TRACE_LOG_COUNT: AtomicUsize = AtomicUsize::new(0);
static SLOW_REDRAW_TRACE_ENABLED: OnceLock<bool> = OnceLock::new();

fn slow_redraw_trace_enabled() -> bool {
    *SLOW_REDRAW_TRACE_ENABLED.get_or_init(|| {
        matches!(
            std::env::var("ZIRCON_EDITOR_TRACE_SLOW_REDRAW").as_deref(),
            Ok("1")
        )
    })
}

fn slow_redraw_trace_slot_available(counter: &AtomicUsize) -> bool {
    counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
            (count < SLOW_REDRAW_TRACE_LOG_LIMIT).then(|| count.saturating_add(1))
        })
        .is_ok()
}

fn slow_redraw_trace_is_slow(elapsed: Duration) -> bool {
    elapsed > SLOW_REDRAW_TRACE_THRESHOLD
}

fn record_slow_redraw_trace(
    started: Option<Instant>,
    frame_update_elapsed: Duration,
    present_elapsed: Duration,
    scenario: UiPerfScenario,
    requested_frame_update: bool,
    requested_present: bool,
) {
    let Some(started) = started else {
        return;
    };
    let elapsed = started.elapsed();
    if !slow_redraw_trace_is_slow(elapsed)
        || !slow_redraw_trace_slot_available(&SLOW_REDRAW_TRACE_LOG_COUNT)
    {
        return;
    }
    let other_elapsed = elapsed
        .saturating_sub(frame_update_elapsed)
        .saturating_sub(present_elapsed);
    eprintln!(
        "[zircon_editor] slow_redraw scenario={scenario:?} elapsed_ms={:.2} frame_update_ms={:.2} present_redraw_ms={:.2} other_ms={:.2} frame_update={} present={}",
        elapsed.as_secs_f64() * 1_000.0,
        frame_update_elapsed.as_secs_f64() * 1_000.0,
        present_elapsed.as_secs_f64() * 1_000.0,
        other_elapsed.as_secs_f64() * 1_000.0,
        requested_frame_update,
        requested_present,
    );
}

fn log_startup_boundary_once(marker: &'static str, once: &Once) {
    once.call_once(|| {
        let unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis());
        eprintln!("[zircon_editor] startup_boundary marker={marker} unix_ms={unix_ms}");
    });
}

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
        let slow_redraw_started = slow_redraw_trace_enabled().then(Instant::now);
        // The standalone startup presenter may redraw before the runtime-owned surface exists.
        let trace_runtime_first_redraw = self.shared_gpu_presenter_active;
        static REDRAW_ENTER: Once = Once::new();
        if trace_runtime_first_redraw {
            log_startup_boundary_once("redraw_enter", &REDRAW_ENTER);
        }
        let presenter_resize_pending = self.pending_presenter_resize.is_some();
        if presenter_resize_pending && !self.apply_pending_presenter_resize(event_loop) {
            record_slow_redraw_trace(
                slow_redraw_started,
                Duration::ZERO,
                Duration::ZERO,
                UiPerfScenario::WindowResize,
                false,
                false,
            );
            return;
        }
        let redraw_scenario = if presenter_resize_pending {
            crate::ui::retained_host::ui_perf::UiPerfScenario::WindowResize
        } else {
            redraw.scenario()
        };
        let requested_frame_update = redraw.requires_frame_update();
        let requested_present = redraw.requires_present();
        let redraw_scenario_guard = enter_ui_perf_scenario(redraw_scenario);
        let frame_update_started = if requested_frame_update {
            slow_redraw_started.map(|_| Instant::now())
        } else {
            None
        };
        if requested_frame_update {
            static FRAME_UPDATE_ENTER: Once = Once::new();
            static FRAME_UPDATE_RETURNED: Once = Once::new();
            if trace_runtime_first_redraw {
                log_startup_boundary_once("frame_update_enter", &FRAME_UPDATE_ENTER);
            }
            if redraw.prefers_interactive_frame_update() {
                self.host.request_interactive_frame_update();
            } else {
                self.host.request_frame_update();
            }
            if trace_runtime_first_redraw {
                log_startup_boundary_once("frame_update_returned", &FRAME_UPDATE_RETURNED);
            }
        }
        let frame_update_elapsed =
            frame_update_started.map_or(Duration::ZERO, |started| started.elapsed());
        self.sync_native_window_minimum_from_presentation();
        let present_scenario = self
            .host
            .take_completed_frame_update_scenario()
            .unwrap_or(redraw_scenario);
        drop(redraw_scenario_guard);
        if !requested_present {
            record_slow_redraw_trace(
                slow_redraw_started,
                frame_update_elapsed,
                Duration::ZERO,
                present_scenario,
                requested_frame_update,
                false,
            );
            return;
        }
        record_damage_region_metrics(&redraw, present_scenario);
        static PRESENT_ENTER: Once = Once::new();
        static PRESENT_RETURNED: Once = Once::new();
        if trace_runtime_first_redraw {
            log_startup_boundary_once("present_redraw_enter", &PRESENT_ENTER);
        }
        let present_started = slow_redraw_started.map(|_| Instant::now());
        present_redraw(self, event_loop, redraw, present_scenario);
        let present_elapsed = present_started.map_or(Duration::ZERO, |started| started.elapsed());
        record_slow_redraw_trace(
            slow_redraw_started,
            frame_update_elapsed,
            present_elapsed,
            present_scenario,
            requested_frame_update,
            true,
        );
        if trace_runtime_first_redraw {
            log_startup_boundary_once("present_redraw_returned", &PRESENT_RETURNED);
        }
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
#[path = "tests/redraw.rs"]
mod tests;
