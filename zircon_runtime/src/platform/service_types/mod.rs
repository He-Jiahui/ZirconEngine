#[path = "driver.rs"]
mod driver;
#[path = "manager.rs"]
mod manager;
#[path = "platform_application_suspend_error.rs"]
mod platform_application_suspend_error;
#[path = "application_suspend_transaction.rs"]
mod platform_application_suspend_transaction;
#[path = "platform_surface_lease_error.rs"]
mod platform_surface_lease_error;
#[path = "platform_window_close_error.rs"]
mod platform_window_close_error;
#[path = "platform_window_close_transaction.rs"]
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
