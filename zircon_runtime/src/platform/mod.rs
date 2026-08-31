//! Platform/windowing integration absorbed into the runtime layer.

mod application_lifecycle;
mod capability;
mod config;
mod event_loop_scheduler;
mod feature_selection;
mod host;
mod host_command_broker;
mod module;
pub mod preferences;
mod service_types;
mod target;
#[cfg(test)]
pub(crate) mod test_support;
mod window_registry;
mod window_state_registry;

pub use crate::core::framework::platform::PLATFORM_MODULE_NAME;
pub use crate::core::manager::PLATFORM_MANAGER_NAME;
pub use application_lifecycle::ApplicationLifecycleServiceError;
pub use capability::{
    CapabilityStatus, CursorBoundaryBackend, CursorOptionsBackend, EventLoopPolicy,
    FileDragDropBackend, GamepadBackend, GamepadEventBackend, GamepadRumbleBackend,
    GestureEventBackend, ImeBackend, InputBackend, KeyboardEventBackend, LinuxWindowProtocol,
    MonitorBackend, MouseButtonBackend, MouseWheelBackend, PlatformCapabilityMatrix,
    PlatformCapabilityReport, PlatformRuntimeCapabilityReport, PlatformRuntimeCapabilityStatus,
    PlatformRuntimeHostRequirement, PointerPositionBackend, RawMouseMotionBackend,
    TouchEventBackend, WindowBackend, WindowEventBackend, WindowLifecycleBackend,
    WindowMetricsBackend,
};
pub use config::{PlatformConfig, PLATFORM_CONFIG_KEY};
pub use feature_selection::PlatformFeatureSelection;
pub use host::PlatformHostServiceError;
pub(crate) use host_command_broker::{
    HostCommandAdmissionError, HostCommandBroker, HostCommandBrokerAccessError,
    HostCommandBrokerError, HostCommandDispatch, HostWindowCommandCompletion,
    PlatformWindowCommandError, WindowCommandFailure,
};
pub use module::{module_descriptor, PlatformModule, PLATFORM_DRIVER_NAME};
pub use preferences::{
    AtomicFilePreferenceStorageBackend, PreferenceBackendWorkAuthority, PreferenceStorageBackend,
    PreferenceStorageBackendDiagnostics,
};
pub(crate) use service_types::{
    PlatformApplicationSuspendError, PlatformApplicationSuspendTransaction,
    PlatformSurfaceLeaseError, PlatformWindowCloseError, PlatformWindowCloseTransaction,
};
pub use service_types::{
    PlatformDriver, PlatformManager, PreferenceStorageBackendInstallError,
    PreferenceStorageBackendInstallErrorKind,
};
pub use target::PlatformTarget;
pub(in crate::platform) use window_registry::allocate_window_registry_id;
pub(crate) use window_registry::{WindowParentKind, WindowRegistry, WindowRegistryError};
pub(crate) use window_state_registry::{WindowStateRegistry, WindowStateRegistryError};

#[cfg(test)]
mod tests;
