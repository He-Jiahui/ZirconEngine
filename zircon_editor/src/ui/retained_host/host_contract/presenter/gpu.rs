mod geometry;
mod lifecycle;
mod present;
mod stats;

use zircon_runtime::rhi::{UiSurfaceDrawList, UiSurfacePresenter};

use super::super::data::{FrameRect, HostPresentationGenerationCursor, HostWindowPresentationData};
use super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::error::HostPresenterResult;
use super::host_chrome_presenter::HostChromePresenter;

pub(in crate::ui::retained_host::host_contract) struct GpuChromePresenter<P: UiSurfacePresenter> {
    surface: P,
    size: (u32, u32),
    diagnostics: HostRefreshDiagnostics,
    last_upload_bytes: u64,
    last_draw_calls: u64,
    surface_cache_initialized: bool,
    native_resize_projection_size: (u32, u32),
    native_resize_draw_list: Option<UiSurfaceDrawList>,
    native_resize_generation: u64,
    submitted_text_profile: Option<serde_json::Value>,
    normal_capture_controller:
        super::super::profiling_artifacts::normal_capture_hold::NormalCaptureController,
    #[cfg(test)]
    native_resize_snapshot_build_count: u64,
    #[cfg(test)]
    native_resize_snapshot_reuse_count: u64,
}

impl<P: UiSurfacePresenter> HostChromePresenter for GpuChromePresenter<P> {
    fn resize(&mut self, size: (u32, u32)) -> HostPresenterResult<()> {
        GpuChromePresenter::resize(self, size)
    }

    fn present(
        &mut self,
        presentation: &HostWindowPresentationData,
        presentation_cursor: HostPresentationGenerationCursor,
        damage: Option<FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        GpuChromePresenter::present(
            self,
            presentation,
            presentation_cursor,
            damage,
            invalidation,
        )
    }

    fn present_during_native_resize(
        &mut self,
        presentation: &HostWindowPresentationData,
        presentation_cursor: HostPresentationGenerationCursor,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        GpuChromePresenter::present_during_native_resize(
            self,
            presentation,
            presentation_cursor,
            invalidation,
        )
    }

    fn submitted_text_profile(&self) -> Option<serde_json::Value> {
        self.submitted_text_profile.clone()
    }

    fn drain_normal_capture(
        &mut self,
        cursor: HostPresentationGenerationCursor,
    ) -> Option<super::super::profiling_artifacts::normal_frame_readback::NormalReadbackExport>
    {
        if self
            .normal_capture_controller
            .readback_wake_deadline()
            .is_none()
        {
            return None;
        }
        let current = self
            .normal_capture_controller
            .readback_current(cursor, self.size);
        if !current {
            self.surface
                .cancel_frame_readback("normal host generation, extent or deadline changed");
        }
        match self.surface.drain_frame_readback() {
            Ok(Some(result)) => self
                .normal_capture_controller
                .accept_frame_readback(result, cursor, self.size),
            Ok(None) => {
                if !current {
                    self.normal_capture_controller.fail_frame_readback("normal generation, extent or bounded completion deadline changed without a delivery");
                }
                None
            }
            Err(error) => {
                self.surface
                    .cancel_frame_readback("normal completion owner failed");
                self.normal_capture_controller
                    .fail_frame_readback(&format!("normal completion drain failed: {error}"));
                None
            }
        }
    }
    fn normal_capture_redraw_requested(&self) -> bool {
        self.normal_capture_controller
            .capture_present_requested(self.size)
    }
    fn normal_capture_wake_deadline(&self) -> Option<std::time::Instant> {
        match (
            self.normal_capture_controller.readback_wake_deadline(),
            self.normal_capture_controller.request_poll_deadline(),
        ) {
            (Some(readback), Some(request)) => Some(readback.min(request)),
            (readback, request) => readback.or(request),
        }
    }
    fn diagnostics_snapshot(&self) -> HostRefreshDiagnostics {
        GpuChromePresenter::diagnostics_snapshot(self)
    }
}

#[cfg(test)]
#[path = "gpu/tests/cases.rs"]
mod tests;
