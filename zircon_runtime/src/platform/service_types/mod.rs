mod driver;
mod manager;

pub use driver::{
    PlatformDriver, PreferenceStorageBackendInstallError, PreferenceStorageBackendInstallErrorKind,
};
pub use manager::PlatformManager;
