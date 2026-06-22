use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::window::Window;

use super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::super::paint_frame::HostRgbaFrame;
use super::error::{HostPresenterError, HostPresenterResult};
use super::host_chrome_presenter::HostChromePresenter;

mod backbuffer;
mod diagnostics;
mod lifecycle;
mod present;
mod surface_io;
#[cfg(test)]
mod tests;

pub(in crate::ui::retained_host::host_contract) struct SoftbufferHostPresenter {
    context: Context<Arc<dyn Window>>,
    surface: Surface<Arc<dyn Window>, Arc<dyn Window>>,
    size: (u32, u32),
    backbuffer: Option<HostRgbaFrame>,
    diagnostics: HostRefreshDiagnostics,
    last_debug_overlay_text: Option<String>,
    last_logged_presentation: Option<String>,
    last_logged_size: Option<(u32, u32)>,
}

impl SoftbufferHostPresenter {
    pub(in crate::ui::retained_host::host_contract) fn new(
        window: Arc<dyn Window>,
    ) -> Result<Self, softbuffer::SoftBufferError> {
        lifecycle::new_presenter(window)
    }

    pub(in crate::ui::retained_host::host_contract) fn resize(
        &mut self,
        size: (u32, u32),
    ) -> Result<(), softbuffer::SoftBufferError> {
        lifecycle::resize_presenter(self, size)
    }

    pub(in crate::ui::retained_host::host_contract) fn present(
        &mut self,
        presentation: &HostWindowPresentationData,
        damage: Option<FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> Result<HostRefreshDiagnostics, softbuffer::SoftBufferError> {
        present::present(self, presentation, damage, invalidation)
    }

    pub(in crate::ui::retained_host::host_contract) fn diagnostics_snapshot(
        &self,
    ) -> HostRefreshDiagnostics {
        self.diagnostics.clone()
    }
}

impl HostChromePresenter for SoftbufferHostPresenter {
    fn resize(&mut self, size: (u32, u32)) -> HostPresenterResult<()> {
        SoftbufferHostPresenter::resize(self, size).map_err(HostPresenterError::softbuffer)
    }

    fn present(
        &mut self,
        presentation: &HostWindowPresentationData,
        damage: Option<FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        SoftbufferHostPresenter::present(self, presentation, damage, invalidation)
            .map_err(HostPresenterError::softbuffer)
    }

    fn diagnostics_snapshot(&self) -> HostRefreshDiagnostics {
        SoftbufferHostPresenter::diagnostics_snapshot(self)
    }
}
