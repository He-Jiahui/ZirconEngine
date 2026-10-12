use super::super::data::{FrameRect, HostPresentationGenerationCursor, HostWindowPresentationData};
use super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::error::HostPresenterResult;

#[cfg(test)]
#[path = "host_chrome_presenter/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract) trait HostChromePresenter {
    fn resize(&mut self, size: (u32, u32)) -> HostPresenterResult<()>;

    fn present(
        &mut self,
        presentation: &HostWindowPresentationData,
        presentation_cursor: HostPresentationGenerationCursor,
        damage: Option<FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics>;

    fn present_during_native_resize(
        &mut self,
        presentation: &HostWindowPresentationData,
        presentation_cursor: HostPresentationGenerationCursor,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        self.present(presentation, presentation_cursor, None, invalidation)
    }

    fn submitted_text_profile(&self) -> Option<serde_json::Value> {
        None
    }
    fn drain_normal_capture(
        &mut self,
        _cursor: HostPresentationGenerationCursor,
    ) -> Option<super::super::profiling_artifacts::normal_frame_readback::NormalReadbackExport>
    {
        None
    }
    fn normal_capture_redraw_requested(&self) -> bool {
        false
    }
    fn normal_capture_wake_deadline(&self) -> Option<std::time::Instant> {
        None
    }
    fn diagnostics_snapshot(&self) -> HostRefreshDiagnostics;
}
