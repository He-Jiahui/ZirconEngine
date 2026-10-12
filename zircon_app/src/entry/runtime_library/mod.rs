#[path = "artifact_manifest.rs"]
mod artifact_manifest;
#[path = "library_path.rs"]
mod library_path;
#[path = "loaded_runtime.rs"]
mod loaded_runtime;
#[path = "runtime_library_error.rs"]
mod runtime_library_error;
#[path = "runtime_session.rs"]
mod runtime_session;
#[path = "runtime_session_create_failure.rs"]
mod runtime_session_create_failure;
#[path = "runtime_teardown_failure.rs"]
mod runtime_teardown_failure;
#[path = "wake_registry.rs"]
mod wake_registry;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(crate) use library_path::{
    default_runtime_library_path, runtime_library_environment_override_request,
    RuntimeLibraryPathError, RuntimeLibraryPathSelection,
};
#[cfg(all(
    test,
    target_os = "windows",
    feature = "dynamic-api",
    not(feature = "gamepad-gilrs")
))]
pub(in crate::entry) use loaded_runtime::surface_teardown_fixture;
pub(crate) use loaded_runtime::{LoadedRuntime, RuntimeLibraryPreflight};
pub(crate) use runtime_library_error::RuntimeLibraryError;
pub(crate) use runtime_session::{
    RuntimeFrame, RuntimeFrameDemand, RuntimeSession, MAX_HOST_RUNTIME_FRAME_DELAY,
};
pub use runtime_session_create_failure::{
    retry_runtime_startup_cleanup, RuntimeSessionCreateFailure,
};
pub(in crate::entry) use runtime_teardown_failure::RuntimeSessionTeardownFailureState;
pub(in crate::entry) use wake_registry::RuntimeWakeRegistration;
