use crate::plugin::PluginModuleKind;

use super::super::behavior_calls::NativePluginBehavior;
use super::diagnostics::{
    degraded_diagnostic, invalid_diagnostic, module_kind_label, ValidationDiagnostic,
};
use super::schema::has_manifest_text;

pub(super) fn validate_callbacks(
    diagnostics: &mut Vec<ValidationDiagnostic>,
    plugin_id: &str,
    module_kind: PluginModuleKind,
    behavior: &NativePluginBehavior,
) {
    if behavior.is_stateless {
        if behavior.state_schema_version != 0 {
            diagnostics.push(degraded_diagnostic(format!(
                "native plugin {plugin_id} {} is stateless but declares state schema version {}",
                module_kind_label(module_kind),
                behavior.state_schema_version
            )));
        }
    } else {
        if !behavior.has_save_state() {
            diagnostics.push(invalid_diagnostic(format!(
                "native plugin {plugin_id} {} behavior callback save_state is missing for stateful behavior",
                module_kind_label(module_kind)
            )));
        }
        if !behavior.has_restore_state() {
            diagnostics.push(invalid_diagnostic(format!(
                "native plugin {plugin_id} {} behavior callback restore_state is missing for stateful behavior",
                module_kind_label(module_kind)
            )));
        }
    }

    if !behavior.has_unload() {
        diagnostics.push(degraded_diagnostic(format!(
            "native plugin {plugin_id} {} behavior callback unload is missing",
            module_kind_label(module_kind)
        )));
    }

    if !behavior.has_invoke_command() && !has_manifest_text(behavior.command_manifest.as_deref()) {
        diagnostics.push(degraded_diagnostic(format!(
            "native plugin {plugin_id} {} behavior callback invoke_command is missing",
            module_kind_label(module_kind)
        )));
    }
}
