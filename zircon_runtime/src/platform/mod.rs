//! Platform/windowing integration absorbed into the runtime layer.

mod capability;
mod config;
mod feature_selection;
mod module;
pub mod preferences;
mod service_types;
mod target;

pub use crate::core::framework::platform::PLATFORM_MODULE_NAME;
pub use crate::core::manager::PLATFORM_MANAGER_NAME;
pub use capability::{
    CapabilityStatus, CursorBoundaryBackend, CursorOptionsBackend, EventLoopPolicy,
    FileDragDropBackend, GamepadBackend, GamepadEventBackend, GamepadRumbleBackend,
    GestureEventBackend, ImeBackend, InputBackend, KeyboardEventBackend, LinuxWindowProtocol,
    MonitorBackend, MouseButtonBackend, MouseWheelBackend, PlatformCapabilityMatrix,
    PlatformCapabilityReport, PointerPositionBackend, RawMouseMotionBackend, TouchEventBackend,
    WindowBackend, WindowEventBackend, WindowLifecycleBackend, WindowMetricsBackend,
};
pub use config::{PlatformConfig, PLATFORM_CONFIG_KEY};
pub use feature_selection::PlatformFeatureSelection;
pub use module::{module_descriptor, PlatformModule, PLATFORM_DRIVER_NAME};
pub use preferences::{
    AtomicFilePreferenceStorageBackend, PreferenceBackendWorkAuthority, PreferenceStorageBackend,
    PreferenceStorageBackendDiagnostics,
};
pub use service_types::{
    PlatformDriver, PlatformManager, PreferenceStorageBackendInstallError,
    PreferenceStorageBackendInstallErrorKind,
};
pub use target::PlatformTarget;

#[cfg(test)]
mod tests;
