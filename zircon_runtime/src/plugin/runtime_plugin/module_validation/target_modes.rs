mod presence;
mod row;
mod rows;

use crate::{plugin::PluginModuleManifest, RuntimeTargetMode};

use self::{
    presence::validate_runtime_plugin_module_target_mode_presence,
    rows::validate_runtime_plugin_module_target_mode_rows,
};

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_module_target_modes(
    manifest_label: &str,
    module: &PluginModuleManifest,
    target_coverage: Option<(&str, &[RuntimeTargetMode])>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_target_mode_presence(manifest_label, module, diagnostics);
    validate_runtime_plugin_module_target_mode_rows(
        manifest_label,
        module,
        target_coverage,
        diagnostics,
    );
}
