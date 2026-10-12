#[cfg(feature = "native-host")]
#[path = "runtime_modules/assembly.rs"]
mod assembly;
#[cfg(feature = "native-host")]
#[path = "runtime_modules/availability.rs"]
mod availability;
#[cfg(feature = "native-host")]
#[path = "runtime_modules/composition.rs"]
mod composition;
#[cfg(feature = "native-host")]
#[path = "runtime_modules/core_modules.rs"]
mod core_modules;
#[path = "runtime_modules/ids.rs"]
mod ids;
#[cfg(feature = "native-host")]
#[path = "runtime_modules/load_report.rs"]
mod load_report;
#[cfg(feature = "native-host")]
#[path = "runtime_modules/manifest.rs"]
mod manifest;
#[cfg(feature = "native-host")]
#[path = "runtime_modules/plugin_modules.rs"]
mod plugin_modules;

#[cfg(feature = "native-host")]
use crate::core::framework::platform::RuntimeTargetMode;
#[cfg(feature = "native-host")]
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
#[cfg(feature = "native-host")]
use composition::{finish_runtime_module_composition, RuntimeModuleCompositionIdentitySeed};
#[cfg(feature = "native-host")]
pub use composition::{
    RuntimeModuleCompositionCompiler, RuntimeModuleCompositionIdentity,
    RuntimeModuleCompositionPlan, RuntimeModuleCompositionRejection,
    RuntimeModuleCompositionResult,
};
#[cfg(feature = "native-host")]
pub use core_modules::runtime_core_modules;

#[cfg(feature = "native-host")]
pub use ids::BuiltinRuntimeModuleId;
pub use ids::RuntimePluginId;
#[cfg(feature = "native-host")]
pub use load_report::RuntimeModuleLoadDiagnostic;
#[cfg(feature = "native-host")]
pub use manifest::{
    default_manifest_for_target, manifest_for_runtime_profile, manifest_with_mode_baseline,
};

#[cfg(all(test, feature = "native-host"))]
#[path = "runtime_modules/tests/mod.rs"]
mod tests;
