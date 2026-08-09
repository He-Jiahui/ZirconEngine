use super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::error::HostPresenterResult;

#[cfg(test)]
mod tests;

pub(in crate::ui::retained_host::host_contract) trait HostChromePresenter {
    fn resize(&mut self, size: (u32, u32)) -> HostPresenterResult<()>;

    fn present(
        &mut self,
        presentation: &HostWindowPresentationData,
        damage: Option<FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics>;

    fn present_during_native_resize(
        &mut self,
        presentation: &HostWindowPresentationData,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        self.present(presentation, None, invalidation)
    }

    fn diagnostics_snapshot(&self) -> HostRefreshDiagnostics;
}
