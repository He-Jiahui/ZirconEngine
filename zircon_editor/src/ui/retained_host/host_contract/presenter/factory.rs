use std::sync::Arc;

use winit::window::Window;

use super::backend::HostPresenterBackend;
use super::error::{HostPresenterError, HostPresenterResult};
use super::gpu::GpuChromePresenter;
use super::host_chrome_presenter::HostChromePresenter;
use super::softbuffer::SoftbufferHostPresenter;
use zircon_runtime::rhi::{create_default_ui_surface_presenter, UiSurfaceDescriptor};

pub(in crate::ui::retained_host::host_contract) fn create_host_chrome_presenter(
    backend: HostPresenterBackend,
    window: Arc<dyn Window>,
) -> HostPresenterResult<Box<dyn HostChromePresenter>> {
    match backend {
        HostPresenterBackend::Gpu => {
            let descriptor =
                UiSurfaceDescriptor::from_winit_window("editor-host-chrome", window.as_ref())?;
            let size = descriptor.clamped_size();
            let surface = create_default_ui_surface_presenter(descriptor)?;
            Ok(Box::new(GpuChromePresenter::new(surface, size)))
        }
        HostPresenterBackend::Softbuffer => Ok(Box::new(
            SoftbufferHostPresenter::new(window).map_err(HostPresenterError::softbuffer)?,
        )),
    }
}
