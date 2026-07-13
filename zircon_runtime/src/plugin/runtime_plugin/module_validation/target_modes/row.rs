mod coverage;
mod editor_host;
mod uniqueness;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginModuleManifest;

use self::{
    coverage::validate_runtime_plugin_module_target_mode_coverage,
    editor_host::validate_runtime_plugin_module_editor_host_target_mode,
    uniqueness::validate_runtime_plugin_module_target_mode_uniqueness,
};

pub(super) fn validate_runtime_plugin_module_target_mode_row(
    manifest_label: &str,
    module: &PluginModuleManifest,
    target_mode: RuntimeTargetMode,
    seen: &mut Vec<RuntimeTargetMode>,
    target_coverage: Option<(&str, &[RuntimeTargetMode])>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_target_mode_uniqueness(
        manifest_label,
        module,
        target_mode,
        seen,
        diagnostics,
    );
    validate_runtime_plugin_module_editor_host_target_mode(
        manifest_label,
        module,
        target_mode,
        diagnostics,
    );
    validate_runtime_plugin_module_target_mode_coverage(
        manifest_label,
        module,
        target_mode,
        target_coverage,
        diagnostics,
    );
}
