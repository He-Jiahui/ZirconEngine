use zircon_runtime::rhi::{UiSurfaceDescriptor, UiSurfacePresenter};

use super::error::{HostPresenterError, HostPresenterResult};

/// Editor-side factory for a retained UI surface sharing the runtime render device.
///
/// The host owns only this trait object. WGPU handles and viewport product lifetime remain inside
/// the runtime framework implementation, so a native host never stores raw graphics pointers.
pub(crate) trait RuntimeUiSurfacePresenterFactory: Send + Sync {
    fn create(
        &self,
        descriptor: UiSurfaceDescriptor,
    ) -> HostPresenterResult<Box<dyn UiSurfacePresenter>>;
}

pub(crate) fn runtime_factory_error(error: impl std::fmt::Display) -> HostPresenterError {
    HostPresenterError::gpu_unavailable(error.to_string())
}
