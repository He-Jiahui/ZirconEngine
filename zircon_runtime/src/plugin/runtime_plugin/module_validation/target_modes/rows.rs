mod state;

use crate::{plugin::PluginModuleManifest, RuntimeTargetMode};

use self::state::new_runtime_plugin_module_target_mode_row_state;
use super::row::validate_runtime_plugin_module_target_mode_row;

pub(super) fn validate_runtime_plugin_module_target_mode_rows(
    manifest_label: &str,
    module: &PluginModuleManifest,
    target_coverage: Option<(&str, &[RuntimeTargetMode])>,
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_module_target_mode_row_state();
    for target_mode in module.target_modes.iter().copied() {
        validate_runtime_plugin_module_target_mode_row(
            manifest_label,
            module,
            target_mode,
            &mut seen,
            target_coverage,
            diagnostics,
        );
    }
}
