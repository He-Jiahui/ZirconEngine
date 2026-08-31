mod driver;
mod manager;
mod platform_application_suspend_error;
mod platform_application_suspend_transaction;
mod platform_surface_lease_error;
mod platform_window_close_error;
mod platform_window_close_transaction;

pub use driver::{
    PlatformDriver, PreferenceStorageBackendInstallError, PreferenceStorageBackendInstallErrorKind,
};
pub use manager::PlatformManager;
pub(crate) use platform_application_suspend_error::PlatformApplicationSuspendError;
pub(crate) use platform_application_suspend_transaction::PlatformApplicationSuspendTransaction;
pub(crate) use platform_surface_lease_error::PlatformSurfaceLeaseError;
pub(crate) use platform_window_close_error::PlatformWindowCloseError;
pub(crate) use platform_window_close_transaction::PlatformWindowCloseTransaction;
