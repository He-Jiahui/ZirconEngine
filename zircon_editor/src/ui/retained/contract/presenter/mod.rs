#[path = "backend.rs"]
mod backend;
#[path = "error.rs"]
mod error;
#[path = "factory.rs"]
mod factory;
#[path = "gpu.rs"]
mod gpu;
#[path = "host_chrome_presenter.rs"]
mod host_chrome_presenter;
#[path = "runtime_factory.rs"]
mod runtime_factory;
#[path = "snapshot.rs"]
mod snapshot;
#[path = "softbuffer.rs"]
mod softbuffer;

pub(in crate::ui::retained_host::host_contract) use backend::HostPresenterBackend;
pub(crate) use error::{HostPresenterError, HostPresenterResult};
pub(in crate::ui::retained_host::host_contract) use factory::{
    create_host_chrome_presenter, create_runtime_host_chrome_presenter,
};
pub(in crate::ui::retained_host::host_contract) use host_chrome_presenter::HostChromePresenter;
pub(crate) use runtime_factory::{runtime_factory_error, RuntimeUiSurfacePresenterFactory};
pub(in crate::ui::retained_host::host_contract) use snapshot::paint_host_presentation_snapshot;
