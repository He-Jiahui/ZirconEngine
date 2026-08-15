use std::sync::Arc;

use winit::window::Window;

use super::super::profiling_artifacts::profile_capture_enabled;
use super::backend::HostPresenterBackend;
use super::error::{HostPresenterError, HostPresenterResult};
use super::gpu::GpuChromePresenter;
use super::host_chrome_presenter::HostChromePresenter;
use super::runtime_factory::RuntimeUiSurfacePresenterFactory;
use super::softbuffer::SoftbufferHostPresenter;
use zircon_runtime::rhi::{create_default_ui_surface_presenter, UiSurfaceDescriptor};

pub(in crate::ui::retained_host::host_contract) fn create_host_chrome_presenter(
    backend: HostPresenterBackend,
    window: Arc<dyn Window>,
    runtime_factory: Option<&dyn RuntimeUiSurfacePresenterFactory>,
) -> HostPresenterResult<(Box<dyn HostChromePresenter>, bool)> {
    match backend {
        HostPresenterBackend::Gpu => {
            let mut descriptor =
                UiSurfaceDescriptor::from_winit_window("editor-host-chrome", window.as_ref())?;
            if profile_capture_enabled() {
                descriptor = descriptor.with_gpu_timing();
            }
            let size = descriptor.clamped_size();
            if let Some(runtime_factory) = runtime_factory {
                if runtime_factory.poll_ready().unwrap_or(false) {
                    if let Ok(surface) = runtime_factory.create(descriptor) {
                        return Ok((Box::new(GpuChromePresenter::new(surface, size)), true));
                    }
                }
            }
            let surface = create_default_ui_surface_presenter(descriptor)?;
            Ok((Box::new(GpuChromePresenter::new(surface, size)), false))
        }
        HostPresenterBackend::Softbuffer => Ok((
            Box::new(SoftbufferHostPresenter::new(window).map_err(HostPresenterError::softbuffer)?),
            false,
        )),
    }
}

pub(in crate::ui::retained_host::host_contract) fn create_runtime_host_chrome_presenter(
    window: Arc<dyn Window>,
    runtime_factory: &dyn RuntimeUiSurfacePresenterFactory,
) -> HostPresenterResult<Box<dyn HostChromePresenter>> {
    let mut descriptor =
        UiSurfaceDescriptor::from_winit_window("editor-host-chrome", window.as_ref())?;
    if profile_capture_enabled() {
        descriptor = descriptor.with_gpu_timing();
    }
    let size = descriptor.clamped_size();
    let surface = runtime_factory.create(descriptor)?;
    Ok(Box::new(GpuChromePresenter::new(surface, size)))
}
