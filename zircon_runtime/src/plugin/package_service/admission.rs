#[path = "admission/policy_index.rs"]
mod policy_index;
#[path = "admission/project_lock.rs"]
mod project_lock;
#[path = "admission/project_resolution.rs"]
mod project_resolution;
pub use project_lock::{capture_project_package_lock, ProjectPackageLockProviderError};

pub use policy_index::{
    load_host_policy_for_target, load_host_policy_index_for_target, LoadedNativePluginPolicy,
    NativePluginInstalledSelection, NativePluginPolicyError,
};
pub use project_resolution::{
    resolve_project_native_plugin_admission, NativePluginAdmission, NativePluginPolicyStatus,
    NativePluginSelectionOutcome, NativePluginSelectionStatus,
};
