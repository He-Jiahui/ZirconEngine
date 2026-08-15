mod native_window;
mod presenter;

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
use native_window::create_native_window_or_exit;
use presenter::{create_presenter_or_exit, create_standalone_presenter_or_exit};

const RUNTIME_PRESENTER_UPGRADE_POLL_INTERVAL: Duration = Duration::from_millis(50);

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
        let Some(window) = create_native_window_or_exit(event_loop, &self.host, requested_size)
        else {
            return;
        };
        self.sync_host_window_state(window.as_ref());
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
        self.restart_profile_measurement_if_ready();
        if self.host.state.borrow().exit_requested {
            event_loop.exit();
            return;
        }
        let now = Instant::now();
        self.try_upgrade_to_runtime_presenter(event_loop, now);
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
        self.schedule_due_resize_reflow(event_loop);
        self.schedule_due_surface_present_retry(now);
        let runtime_frame_due = self.host.take_due_runtime_frame_wake(now);
        let maintenance_frame_due = self.host.take_due_maintenance_frame_wake(now);
        if runtime_frame_due || maintenance_frame_due {
            // Materialize the wake through the regular external-redraw bridge so
            // redraw_requested_impl observes a pending frame update.
            self.drain_external_redraw_request();
        }
        match earliest_wake_deadline(
            self.host.runtime_frame_wake_deadline(),
            earliest_wake_deadline(
                self.host.maintenance_frame_wake_deadline(),
                earliest_wake_deadline(
                    self.pending_resize_reflow_deadline,
                    earliest_wake_deadline(
                        self.pending_surface_present_retry_deadline,
                        self.runtime_presenter_upgrade_poll_deadline,
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

    fn schedule_due_resize_reflow(&mut self, event_loop: &dyn ActiveEventLoop) {
        let Some(deadline) = self.pending_resize_reflow_deadline else {
            return;
        };
        if Instant::now() < deadline {
            return;
        }
        self.pending_resize_reflow_deadline = None;
        if !self.apply_pending_presenter_resize(event_loop) {
            return;
        }
        self.host.commit_native_resize_reflow();
        if self.queue_redraw(HostRedrawRequest::full_frame_for_scenario(
            UiPerfScenario::WindowResize,
            true,
        )) {
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
        drop(self.presenter.take());
        match create_runtime_host_chrome_presenter(window.clone(), factory.as_ref()) {
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
mod tests {
    use std::time::{Duration, Instant};

    use super::earliest_wake_deadline;

    #[test]
    fn no_runtime_deadline_resets_the_native_wait_policy() {
        let source = include_str!("lifecycle.rs");
        assert!(source.contains("None => event_loop.set_control_flow(ControlFlow::Wait)"));
    }

    #[test]
    fn about_to_wait_does_not_poll_native_window_metrics_per_event_batch() {
        let source = include_str!("lifecycle.rs");
        let function = source
            .split("pub(in crate::ui::retained_host::host_contract) fn about_to_wait_impl")
            .nth(1)
            .and_then(|body| body.split("fn schedule_due_surface_present_retry").next())
            .expect("about-to-wait implementation");

        assert!(!function.contains("sync_host_window_state"));
        assert!(!function.contains("window.surface_size()"));
        assert!(!function.contains("window.outer_position()"));
        assert!(!function.contains("window.is_maximized()"));
    }

    #[test]
    fn about_to_wait_restarts_profile_measurement_after_callback_scopes_drop() {
        let source = include_str!("lifecycle.rs");
        let function = source
            .split("pub(in crate::ui::retained_host::host_contract) fn about_to_wait_impl")
            .nth(1)
            .and_then(|body| body.split("fn schedule_due_surface_present_retry").next())
            .expect("about-to-wait implementation");

        assert!(function.contains("self.restart_profile_measurement_if_ready();"));
    }

    #[test]
    fn runtime_presenter_upgrade_releases_previous_native_surface_first() {
        let source = include_str!("lifecycle.rs");
        let release = source
            .find("drop(self.presenter.take());")
            .expect("upgrade must release the startup surface");
        let create = source
            .find("match create_runtime_host_chrome_presenter")
            .expect("upgrade must create the runtime presenter");
        assert!(release < create);
    }

    #[test]
    fn runtime_presenter_upgrade_waits_for_readiness_before_releasing_fallback() {
        let source = include_str!("lifecycle.rs");
        let function = source
            .split("fn try_upgrade_to_runtime_presenter")
            .nth(1)
            .and_then(|body| {
                body.split(
                    "pub(in crate::ui::retained_host::host_contract) fn sync_host_window_state",
                )
                .next()
            })
            .expect("runtime presenter upgrade implementation");
        let readiness = function
            .find("factory.poll_ready()")
            .expect("upgrade must poll the runtime presenter factory without releasing fallback");
        let release = function
            .find("drop(self.presenter.take());")
            .expect("ready handoff must release the standalone native surface first");

        assert!(readiness < release);
        assert!(function.contains("runtime_presenter_upgrade_attempted"));
        assert!(function.contains("runtime_presenter_upgrade_poll_deadline"));
    }

    #[test]
    fn pending_runtime_presenter_upgrade_participates_in_native_wait_policy() {
        let source = include_str!("lifecycle.rs");
        let about_to_wait = source
            .split("pub(in crate::ui::retained_host::host_contract) fn about_to_wait_impl")
            .nth(1)
            .and_then(|body| body.split("fn schedule_due_surface_present_retry").next())
            .expect("about-to-wait implementation");

        assert!(about_to_wait.contains("runtime_presenter_upgrade_poll_deadline"));
        assert!(source.contains("RUNTIME_PRESENTER_UPGRADE_POLL_INTERVAL"));
    }

    #[test]
    fn resize_reflow_deadline_wakes_before_a_later_runtime_frame() {
        let now = Instant::now();
        let resize = now + Duration::from_millis(80);
        let runtime = now + Duration::from_secs(1);

        assert_eq!(
            earliest_wake_deadline(Some(runtime), Some(resize)),
            Some(resize)
        );
        assert_eq!(earliest_wake_deadline(None, Some(resize)), Some(resize));
    }

    #[test]
    fn surface_retry_deadline_participates_in_the_native_wait_policy() {
        let now = Instant::now();
        let surface_retry = now + Duration::from_millis(8);
        let resize = now + Duration::from_millis(80);
        let runtime = now + Duration::from_secs(1);

        assert_eq!(
            earliest_wake_deadline(
                Some(runtime),
                earliest_wake_deadline(Some(resize), Some(surface_retry)),
            ),
            Some(surface_retry)
        );
    }

    #[test]
    fn due_resize_reconfigures_the_latest_surface_before_committing_reflow() {
        let source = include_str!("lifecycle.rs");
        let function = source
            .split("fn schedule_due_resize_reflow")
            .nth(1)
            .and_then(|body| body.split("fn try_upgrade_to_runtime_presenter").next())
            .expect("resize reflow scheduler");
        let configure = function
            .find("self.apply_pending_presenter_resize(event_loop)")
            .expect("due resize must apply any latest unpresented physical size");
        let commit = function
            .find("self.host.commit_native_resize_reflow();")
            .expect("due resize must commit deferred metrics");
        let redraw = function
            .find("HostRedrawRequest::full_frame_for_scenario")
            .expect("due resize should request one frame update");

        assert!(configure < commit);
        assert!(commit < redraw);
    }

    #[test]
    fn pending_resize_is_consumed_before_testing_presenter_availability() {
        let source = include_str!("lifecycle.rs");
        let function = source
            .split("pub(super) fn apply_pending_presenter_resize")
            .nth(1)
            .and_then(|body| body.split("fn try_upgrade_to_runtime_presenter").next())
            .expect("pending presenter resize implementation");
        let consume_size = function
            .find("self.pending_presenter_resize.take()")
            .expect("pending size consumption");
        let presenter_gate = function
            .find("self.presenter.as_mut()")
            .expect("presenter availability gate");

        assert!(consume_size < presenter_gate);
    }
}
