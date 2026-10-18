#[path = "../lifecycle/native_minimum.rs"]
mod native_minimum;
#[path = "../lifecycle/native_window.rs"]
mod native_window;
#[path = "lifecycle/presenter.rs"]
mod presenter;

use crate::ui::retained_host::host_contract::normal_capture_redraw_trace as capture_trace;
use std::time::Instant;

use crate::ui::retained_host::host_contract::diagnostics::HostWindowDiagnosticSeverity;
use crate::ui::retained_host::host_contract::presenter::HostPresenterBackend;
use crate::ui::retained_host::primitives::{PhysicalPosition, PhysicalSize};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::Window;
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel,
};

use super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::redraw::HostRedrawRequest;
use super::presenter_startup::PresenterStartupAction;
use native_minimum::current_native_minimum_surface_size;
use native_window::create_native_window_or_exit;
use presenter::{create_presenter_or_exit, create_standalone_presenter_or_exit};

pub(super) fn normal_capture_redraw(requested: bool) -> HostRedrawRequest {
    if requested {
        // Present the current retained projection once; a capture demand must
        // not change business, selection, focus, layout or generation counters.
        HostRedrawRequest::full_frame_for_scenario(
            crate::ui::retained_host::ui_perf::current_ui_perf_scenario(),
            false,
        )
    } else {
        HostRedrawRequest::none()
    }
}

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract) fn can_create_surfaces_impl(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        self.native_surfaces_available = true;
        if self.window.is_some() {
            return;
        }
        let Some(use_runtime_presenter) = self.poll_presenter_startup(Instant::now()) else {
            return;
        };

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
        let created = if use_runtime_presenter {
            create_presenter_or_exit(event_loop, &self.host, window.clone())
        } else {
            create_standalone_presenter_or_exit(event_loop, &self.host, window.clone())
        };
        let Some((presenter_backend, presenter, shared_gpu_presenter_active)) = created
        else {
            return;
        };
        eprintln!(
            "[zircon_editor] startup_boundary marker=presenter_selected presenter_backend={} runtime_shared={shared_gpu_presenter_active}",
            presenter_backend.label(),
        );
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
        if self.native_surfaces_available && self.window.is_none() {
            self.can_create_surfaces_impl(event_loop);
        }
        let capture_cursor = self.host.get_host_presentation_generation().cursor();
        let capture_export = self
            .presenter
            .as_mut()
            .and_then(|presenter| presenter.drain_normal_capture(capture_cursor));
        if let Some(export) = capture_export {
            match self.host.profile_artifact_jobs() {
                Some(jobs)=>match crate::ui::retained_host::host_contract::profiling_artifacts::normal_frame_readback::submit_normal_readback(&jobs,export) {
                    Ok(Some(job))=>self.host.track_profile_artifact_job(job),Ok(None)=>{},Err(error)=>self.host.record_host_diagnostic(HostWindowDiagnosticSeverity::Warning,format!("actual normal GPU readback export rejected: {error}")),
                },
                None=>self.host.record_host_diagnostic(HostWindowDiagnosticSeverity::Warning,"actual normal GPU readback export has no injected editor job owner"),
            }
        }
        let capture_requested = self
            .presenter
            .as_ref()
            .is_some_and(|presenter| presenter.normal_capture_redraw_requested());
        if capture_requested {
            capture_trace::record("capture_demand", || {
                format!(
                    "owner={:#x} cursor={capture_cursor:?} pending={:?} retry={:?}",
                    self.host.state.as_ptr() as usize,
                    self.pending_redraw,
                    self.pending_surface_present_retry
                )
            });
        }
        self.host
            .queue_external_redraw(normal_capture_redraw(capture_requested));
        let capture_deadline = self
            .presenter
            .as_ref()
            .and_then(|presenter| presenter.normal_capture_wake_deadline());
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
            capture_trace::record("idle_background_wake_consumed", || {
                format!(
                    "owner={:#x} capture_requested={capture_requested}",
                    self.host.state.as_ptr() as usize
                )
            });
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
            capture_trace::record("due_wakes", || {
                format!("owner={:#x} capture_requested={capture_requested} runtime={runtime_frame_due} maintenance={maintenance_frame_due} input_timer={input_timer_frame_due} lifecycle={lifecycle_frame_due} pending={:?}",
                    self.host.state.as_ptr() as usize, self.pending_redraw)
            });
        }
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
            earliest_wake_deadline(self.host.runtime_frame_wake_deadline(), capture_deadline),
            earliest_wake_deadline(
                self.host.maintenance_frame_wake_deadline(),
                earliest_wake_deadline(
                    self.host.input_timer_frame_wake_deadline(),
                    earliest_wake_deadline(
                        self.host.lifecycle_frame_wake_deadline(),
                        earliest_wake_deadline(
                            self.pending_surface_present_retry_deadline,
                            self.presenter_startup.wake_deadline(),
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

    fn poll_presenter_startup(&mut self, now: Instant) -> Option<bool> {
        if !HostPresenterBackend::default_native().is_gpu() {
            self.presenter_startup.select(false);
            return Some(false);
        }
        let Some(factory) = self.host.runtime_presenter_factory() else {
            self.presenter_startup.select(false);
            return Some(false);
        };
        match self.presenter_startup.action(now) {
            PresenterStartupAction::Wait => None,
            PresenterStartupAction::Selected { runtime } => Some(runtime),
            PresenterStartupAction::TimedOut => {
                self.host.set_graphics_degradation(Some(
                    "Standalone GPU presenter: runtime readiness timed out after 2 s; viewport/PIE capture uses CPU copies. Restart to retry shared GPU presentation.".to_owned(),
                ));
                Some(false)
            }
            PresenterStartupAction::Poll => {
                zircon_runtime::profile_counter!("editor", "ui.presenter.startup_readiness_poll_count", 1_u8);
                match factory.poll_ready() {
                    Ok(false) => None,
                    Ok(true) => {
                        self.presenter_startup.select(true);
                        Some(true)
                    }
                    Err(error) => {
                        self.presenter_startup.select(false);
                        self.host.set_graphics_degradation(Some(format!(
                            "Standalone GPU presenter: runtime readiness failed ({error}); viewport/PIE capture uses CPU copies. Restart to retry."
                        )));
                        Some(false)
                    }
                }
            }
        }
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
