mod assembly;
mod availability;
mod core_modules;
mod ids;
mod load_report;
mod manifest;
mod plugin_modules;

use crate::core::framework::platform::RuntimeTargetMode;
pub use assembly::{
    builtin_runtime_modules, runtime_modules_for_runtime_profile,
    runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports,
    runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports,
    runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports,
    runtime_modules_for_runtime_profile_with_plugin_registration_reports,
    runtime_modules_for_target, runtime_modules_for_target_with_linked_plugins,
    runtime_modules_for_target_with_plugin_and_feature_registration_reports,
    runtime_modules_for_target_with_plugin_registration_reports,
};
pub use core_modules::runtime_core_modules;

pub use ids::RuntimePluginId;
pub use load_report::{RuntimeModuleLoadDiagnostic, RuntimeModuleLoadReport};
pub use manifest::{
    default_manifest_for_target, manifest_for_runtime_profile, manifest_with_mode_baseline,
};

#[cfg(test)]
mod tests;
