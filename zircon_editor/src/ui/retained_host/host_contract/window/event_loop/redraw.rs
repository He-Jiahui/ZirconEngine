use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use zircon_runtime::diagnostic_log::write_error;

use super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::profiling_artifacts::export_present_artifacts;
use crate::ui::retained_host::host_contract::redraw::{
    HostRedrawRequest, NativePointerDispatchResult,
};
use crate::ui::retained_host::ui_perf::enter_ui_perf_scenario;

impl UiHostWindowEventLoop {
    pub(super) fn dispatch_pointer_result(&mut self, result: NativePointerDispatchResult) {
        let redraw = result.redraw();
        if redraw.request_redraw() {
            self.queue_redraw(redraw);
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    pub(super) fn queue_redraw(&mut self, redraw: HostRedrawRequest) {
        self.pending_redraw = self.pending_redraw.clone().merge(redraw);
    }

    pub(super) fn drain_external_redraw_request(&mut self) {
        let redraw = self.host.take_external_redraw();
        if redraw.request_redraw() {
            self.queue_redraw(redraw);
            if let Some(window) = self.window.as_ref() {
                schedule_native_redraw(window.as_ref());
            }
        }
    }

    pub(super) fn redraw_requested_impl(&mut self, event_loop: &dyn ActiveEventLoop) {
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
        let _present_scenario_guard = enter_ui_perf_scenario(present_scenario);
        if let Some(presenter) = self.presenter.as_mut() {
            let presentation = self.host.get_host_presentation();
            let invalidation = self.host.refresh_invalidation_diagnostics();
            match presenter.present(&presentation, redraw.damage_region().cloned(), invalidation) {
                Ok(diagnostics) => {
                    if let Some(backend) = self.presenter_backend {
                        export_present_artifacts(
                            &presentation,
                            &self.host.window().size(),
                            backend,
                        );
                    }
                    self.host.set_host_refresh_diagnostics_overlay(diagnostics)
                }
                Err(error) => {
                    write_error(
                        "editor_host_window",
                        format!("presenter present failed: {error}"),
                    );
                    event_loop.exit();
                }
            }
        }
    }

    fn take_pending_redraw(&mut self) -> HostRedrawRequest {
        std::mem::replace(&mut self.pending_redraw, HostRedrawRequest::None)
    }
}

fn schedule_native_redraw(window: &dyn Window) {
    window.request_redraw();
}
