mod assembly;
mod availability;
mod composition;
mod core_modules;
mod ids;
mod load_report;
mod manifest;
mod plugin_modules;

use crate::core::framework::platform::RuntimeTargetMode;
pub use assembly::{
    runtime_modules_for_compiled_project_plugin_plan, runtime_modules_for_runtime_profile,
    runtime_modules_for_runtime_profile_compiled_project_plugin_plan,
    runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports,
    runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports,
    runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports,
    runtime_modules_for_runtime_profile_with_plugin_registration_reports,
    runtime_modules_for_target, runtime_modules_for_target_with_linked_plugins,
    runtime_modules_for_target_with_plugin_and_feature_registration_reports,
    runtime_modules_for_target_with_plugin_registration_reports,
};
use composition::{finish_runtime_module_composition, RuntimeModuleCompositionIdentitySeed};
pub use composition::{
    RuntimeModuleCompositionCompiler, RuntimeModuleCompositionIdentity,
    RuntimeModuleCompositionPlan, RuntimeModuleCompositionRejection,
    RuntimeModuleCompositionResult,
};
pub use core_modules::runtime_core_modules;

pub use ids::{BuiltinRuntimeModuleId, RuntimePluginId};
pub use load_report::RuntimeModuleLoadDiagnostic;
pub use manifest::{
    default_manifest_for_target, manifest_for_runtime_profile, manifest_with_mode_baseline,
};

#[cfg(test)]
mod tests;
