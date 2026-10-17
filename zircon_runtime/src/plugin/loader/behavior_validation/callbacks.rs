use crate::plugin::PluginModuleKind;

use super::super::behavior_calls::NativePluginBehavior;
use super::diagnostics::{
    degraded_diagnostic, invalid_diagnostic, module_kind_label, ValidationDiagnostic,
};

/// 汇总行为可用性约束：有状态行为必须可保存和恢复，无状态行为无需这两项回调。
/// 缺少卸载或只有状态功能时保留降级诊断，供报告与生命周期调用方分别处理。
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

    // A nonempty command manifest does not establish that invoke_command exists.
    // Keep the missing-callback diagnostic even when all remaining behavior fields are valid.
    // This check follows manifest validation and still runs for declared commands.
    // nonempty_command_manifest_does_not_mask_a_missing_command_callback covers this path.
    if !behavior.has_invoke_command() {
        diagnostics.push(degraded_diagnostic(format!(
            "native plugin {plugin_id} {} behavior callback invoke_command is missing",
            module_kind_label(module_kind)
        )));
    }
}
