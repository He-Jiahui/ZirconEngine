mod state;

use crate::plugin::PluginModuleManifest;

use self::state::new_runtime_plugin_module_capability_row_state;
use super::row::validate_runtime_plugin_module_capability_row;

pub(super) fn validate_runtime_plugin_module_capability_rows(
    manifest_label: &str,
    module: &PluginModuleManifest,
    validate_field: Option<fn(&str, &str, &mut Vec<String>)>,
    validate_namespace: fn(&str, &str, &mut Vec<String>),
    diagnostics: &mut Vec<String>,
) {
    let mut seen = new_runtime_plugin_module_capability_row_state();
    for capability in &module.capabilities {
        validate_runtime_plugin_module_capability_row(
            manifest_label,
            module,
            capability,
            &mut seen,
            validate_field,
            validate_namespace,
            diagnostics,
        );
    }
}
