mod geometry;
mod lifecycle;
mod present;
mod stats;

use zircon_runtime::rhi::{UiSurfaceDrawList, UiSurfacePresenter};

use super::super::data::{FrameRect, HostWindowPresentationData};
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
        damage: Option<FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        GpuChromePresenter::present(self, presentation, damage, invalidation)
    }

    fn present_during_native_resize(
        &mut self,
        presentation: &HostWindowPresentationData,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        GpuChromePresenter::present_during_native_resize(self, presentation, invalidation)
    }

    fn diagnostics_snapshot(&self) -> HostRefreshDiagnostics {
        GpuChromePresenter::diagnostics_snapshot(self)
    }
}

#[cfg(test)]
mod tests;
