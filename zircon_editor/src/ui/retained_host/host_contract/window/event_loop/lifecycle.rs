mod native_minimum;
mod native_window;
mod presenter;

use std::sync::Once;
use std::time::{Duration, Instant};

use crate::ui::retained_host::host_contract::diagnostics::HostWindowDiagnosticSeverity;
use crate::ui::retained_host::host_contract::presenter::{
    create_runtime_host_chrome_presenter, HostPresenterBackend,
};
use crate::ui::retained_host::primitives::{PhysicalPosition, PhysicalSize};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::Window;
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel,
};

use super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::redraw::HostRedrawRequest;
use crate::ui::retained_host::ui_perf::UiPerfScenario;
use native_minimum::current_native_minimum_surface_size;
use native_window::create_native_window_or_exit;
use presenter::{create_presenter_or_exit, create_standalone_presenter_or_exit};

const RUNTIME_PRESENTER_UPGRADE_POLL_INTERVAL: Duration = Duration::from_millis(50);

fn log_startup_boundary_once(marker: &'static str, once: &Once) {
    once.call_once(|| {
        let unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis());
        eprintln!("[zircon_editor] startup_boundary marker={marker} unix_ms={unix_ms}");
    });
}

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract) fn can_create_surfaces_impl(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        if self.window.is_some() {
            return;
        }

        let size = self.host.window().size();
        let requested_size = size.clone();
        let minimum_size = current_native_minimum_surface_size(&self.host);
        let Some(window) =
            create_native_window_or_exit(event_loop, &self.host, requested_size, minimum_size)
        else {
            return;
        };
        self.sync_host_window_state(window.as_ref());
        self.sync_native_window_minimum(window.as_ref());
        let Some((presenter_backend, presenter, shared_gpu_presenter_active)) =
            create_presenter_or_exit(event_loop, &self.host, window.clone())
        else {
            return;
        };
        if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
            write_diagnostic_log(
                "editor_host_window",
                format!(
                    "created native window size={}x{} presenter_backend={}",
                    size.width,
                    size.height,
                    presenter_backend.label()
                ),
            );
        }
        window.request_redraw();
        self.window = Some(window);
        self.presenter = Some(presenter);
        self.presenter_backend = Some(presenter_backend);
        self.shared_gpu_presenter_active = shared_gpu_presenter_active;
        self.host
            .set_direct_viewport_products_active(shared_gpu_presenter_active);
    }

    pub(in crate::ui::retained_host::host_contract) fn about_to_wait_impl(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        // All scopes from the preceding event/present callback have dropped at this boundary.
        self.flush_pending_idle_pointer_move();
        self.restart_profile_measurement_if_ready();
        if self.host.state.borrow().exit_requested {
            event_loop.exit();
            return;
        }
        let now = Instant::now();
        self.try_upgrade_to_runtime_presenter(event_loop, now);
        if self.host.take_visual_asset_completion_wake() {
            if let Some(completion) =
                crate::ui::retained_host::host_contract::take_visual_asset_completion()
            {
                crate::ui::retained_host::ui_perf::record_ui_perf_counter(
                    completion.scenario,
                    crate::ui::retained_host::ui_perf::UiPerfCounter::VisualAssetAsyncCompletionRedrawCount,
                    1.0,
                );
                let redraw = match completion.damage_frame {
                    Some(damage_frame) => {
                        HostRedrawRequest::region_for_scenario(completion.scenario, damage_frame)
                    }
                    None => HostRedrawRequest::full_frame_for_scenario(completion.scenario, false),
                };
                self.host.queue_external_redraw(redraw);
            }
        }
        if self.host.take_background_event_wake() {
            self.host.request_maintenance_frame_update();
        }
        if self.host.has_window_attention_request() {
            if let Some(window) = self.window.as_ref() {
                if self.host.take_window_attention_request() {
                    window.focus_window();
                }
            }
        }
        self.drain_external_redraw_request();
        self.schedule_due_surface_present_retry(now);
        let runtime_frame_due = self.host.take_due_runtime_frame_wake(now);
        let maintenance_frame_due = self.host.take_due_maintenance_frame_wake(now);
        let input_timer_frame_due = self.host.take_due_input_timer_frame_wake(now);
        let lifecycle_frame_due = self.host.take_due_lifecycle_frame_wake(now);
        if runtime_frame_due
            || maintenance_frame_due
            || input_timer_frame_due
            || lifecycle_frame_due
        {
            // Materialize the wake through the regular external-redraw bridge so
            // redraw_requested_impl observes a pending frame update.
            self.drain_external_redraw_request();
        }
        match earliest_wake_deadline(
            self.host.runtime_frame_wake_deadline(),
            earliest_wake_deadline(
                self.host.maintenance_frame_wake_deadline(),
                earliest_wake_deadline(
                    self.host.input_timer_frame_wake_deadline(),
                    earliest_wake_deadline(
                        self.host.lifecycle_frame_wake_deadline(),
                        earliest_wake_deadline(
                            self.pending_surface_present_retry_deadline,
                            self.runtime_presenter_upgrade_poll_deadline,
                        ),
                    ),
                ),
            ),
        ) {
            Some(deadline) => event_loop.set_control_flow(ControlFlow::WaitUntil(deadline)),
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }

    fn schedule_due_surface_present_retry(&mut self, now: Instant) {
        let retry = self.take_due_surface_present_retry(now);
        if self.queue_redraw(retry) {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    pub(super) fn apply_pending_presenter_resize(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) -> bool {
        let Some(size) = self.pending_presenter_resize.take() else {
            return true;
        };
        let Some(presenter) = self.presenter.as_mut() else {
            // A newly-created presenter reads the window's current surface size directly.
            return true;
        };
        #[cfg(feature = "profiling")]
        let resize_started = Instant::now();
        let resize_result = presenter.resize(size);
        #[cfg(feature = "profiling")]
        {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.window_resize.surface_reconfigure_count",
                1.0
            );
            zircon_runtime::profile_counter!(
                "editor",
                "ui.window_resize.surface_reconfigure_us",
                resize_started.elapsed().as_secs_f64() * 1_000_000.0
            );
        }
        if let Err(error) = resize_result {
            self.host.report_fatal_failure(
                "editor_host_window",
                format!("presenter size={}x{}", size.0, size.1),
                format!("presenter resize failed: {error}"),
                "verify the graphics adapter and window surface, then restart zircon_editor",
            );
            event_loop.exit();
            return false;
        }
        true
    }

    fn try_upgrade_to_runtime_presenter(&mut self, event_loop: &dyn ActiveEventLoop, now: Instant) {
        if self.shared_gpu_presenter_active
            || self.presenter_backend != Some(HostPresenterBackend::Gpu)
            || self.runtime_presenter_upgrade_attempted
        {
            self.runtime_presenter_upgrade_poll_deadline = None;
            return;
        }
        if self
            .runtime_presenter_upgrade_poll_deadline
            .is_some_and(|deadline| now < deadline)
        {
            return;
        }
        let Some(factory) = self.host.runtime_presenter_factory() else {
            self.runtime_presenter_upgrade_poll_deadline = None;
            return;
        };
        zircon_runtime::profile_counter!("editor", "ui.presenter.runtime_upgrade_poll_count", 1_u8);
        match factory.poll_ready() {
            Ok(false) => {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.presenter.runtime_upgrade_pending_count",
                    1_u8
                );
                self.runtime_presenter_upgrade_poll_deadline =
                    Some(now + RUNTIME_PRESENTER_UPGRADE_POLL_INTERVAL);
                return;
            }
            Err(error) => {
                self.runtime_presenter_upgrade_attempted = true;
                self.runtime_presenter_upgrade_poll_deadline = None;
                self.host.record_host_diagnostic(
                    HostWindowDiagnosticSeverity::Warning,
                    format!(
                        "runtime presenter readiness failed; keeping standalone presenter: {error}"
                    ),
                );
                return;
            }
            Ok(true) => {}
        }
        let Some(window) = self.window.clone() else {
            self.runtime_presenter_upgrade_poll_deadline =
                Some(now + RUNTIME_PRESENTER_UPGRADE_POLL_INTERVAL);
            return;
        };
        self.runtime_presenter_upgrade_attempted = true;
        self.runtime_presenter_upgrade_poll_deadline = None;
        zircon_runtime::profile_counter!(
            "editor",
            "ui.presenter.runtime_upgrade_attempt_count",
            1_u8
        );
        // Native graphics backends cannot configure two surfaces for the same HWND at once.
        // Release the startup presenter before the runtime-owned presenter claims the window.
        static UPGRADE_ENTER: Once = Once::new();
        static UPGRADE_RETURNED: Once = Once::new();
        log_startup_boundary_once("runtime_presenter_upgrade_enter", &UPGRADE_ENTER);
        drop(self.presenter.take());
        let presenter_result =
            create_runtime_host_chrome_presenter(window.clone(), factory.as_ref());
        log_startup_boundary_once("runtime_presenter_upgrade_returned", &UPGRADE_RETURNED);
        match presenter_result {
            Ok(presenter) => {
                self.presenter = Some(presenter);
                self.shared_gpu_presenter_active = true;
                self.host.set_direct_viewport_products_active(true);
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.presenter.runtime_upgrade_success_count",
                    1_u8
                );
            }
            Err(error) => {
                self.host.record_host_diagnostic(
                    HostWindowDiagnosticSeverity::Warning,
                    format!(
                        "runtime presenter upgrade failed; restoring standalone presenter: {error}"
                    ),
                );
                let Some((backend, presenter, shared_gpu_presenter_active)) =
                    create_standalone_presenter_or_exit(event_loop, &self.host, window.clone())
                else {
                    return;
                };
                self.presenter = Some(presenter);
                self.presenter_backend = Some(backend);
                self.shared_gpu_presenter_active = shared_gpu_presenter_active;
                self.host
                    .set_direct_viewport_products_active(shared_gpu_presenter_active);
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.presenter.runtime_upgrade_fallback_count",
                    1_u8
                );
            }
        }
        static UPGRADE_READY_FOR_REDRAW: Once = Once::new();
        log_startup_boundary_once(
            "runtime_presenter_upgrade_ready_for_redraw",
            &UPGRADE_READY_FOR_REDRAW,
        );
        window.request_redraw();
    }

    pub(in crate::ui::retained_host::host_contract) fn sync_host_window_state(
        &self,
        window: &dyn Window,
    ) {
        let size = window.surface_size();
        let mut state = self.host.state.borrow_mut();
        state.window_size = PhysicalSize::new(size.width, size.height);
        state.set_window_scale_factor(window.scale_factor() as f32);
        state.window_visible = true;
        state.window_maximized = window.is_maximized();
        if let Ok(position) = window.outer_position() {
            state.window_position = PhysicalPosition::new(position.x, position.y);
        }
    }
}

fn earliest_wake_deadline(first: Option<Instant>, second: Option<Instant>) -> Option<Instant> {
    match (first, second) {
        (Some(first), Some(second)) => Some(first.min(second)),
        (Some(deadline), None) | (None, Some(deadline)) => Some(deadline),
        (None, None) => None,
    }
}

#[cfg(test)]
#[path = "tests/lifecycle.rs"]
mod tests;
