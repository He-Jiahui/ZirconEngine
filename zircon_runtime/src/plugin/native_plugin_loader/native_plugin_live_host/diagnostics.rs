use crate::plugin::PluginModuleKind;

use super::super::{
    NativePluginBehaviorCallReport, NativePluginLoadReport, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use super::keys::module_kind_label;

pub(super) fn load_report_diagnostics(report: &NativePluginLoadReport) -> Vec<String> {
    let mut diagnostics = report.diagnostics.clone();
    diagnostics.extend(report.descriptor_diagnostics());
    diagnostics.extend(report.entry_diagnostics());
    diagnostics
}

pub(super) fn unloaded_plugin_error(plugin_id: &str, module_kind: PluginModuleKind) -> String {
    format!(
        "plugin {plugin_id} is not loaded in the {} live host; run Hot Reload after building its native dynamic package",
        module_kind_label(module_kind)
    )
}

pub(super) fn diagnostics_for_plugin(
    report: &NativePluginLoadReport,
    plugin_id: &str,
    module_kind: PluginModuleKind,
) -> Vec<String> {
    match module_kind {
        PluginModuleKind::Runtime => report.diagnostics_for_runtime_plugin(plugin_id),
        PluginModuleKind::Editor => report.diagnostics_for_editor_plugin(plugin_id),
        PluginModuleKind::Native | PluginModuleKind::Vm => report.diagnostics_for_plugin(plugin_id),
    }
}

pub(super) fn diagnostics_from_behavior_report(
    label: &str,
    report: NativePluginBehaviorCallReport,
) -> Result<Vec<String>, String> {
    if report.status_code == ZIRCON_NATIVE_PLUGIN_STATUS_OK {
        return Ok(report.diagnostics);
    }
    let diagnostics = if report.diagnostics.is_empty() {
        vec![format!("{label} returned status {}", report.status_code)]
    } else {
        report
            .diagnostics
            .into_iter()
            .map(|message| format!("{label}: {message}"))
            .collect()
    };
    Err(diagnostics.join("; "))
}

pub(super) fn report_diagnostics(
    plugin_id: &str,
    operation: &str,
    report: &NativePluginBehaviorCallReport,
) -> Vec<String> {
    let mut diagnostics = report
        .diagnostics
        .iter()
        .map(|diagnostic| format!("runtime plugin {plugin_id} {operation}: {diagnostic}"))
        .collect::<Vec<_>>();
    if report.status_code != ZIRCON_NATIVE_PLUGIN_STATUS_OK && diagnostics.is_empty() {
        diagnostics.push(format!(
            "runtime plugin {plugin_id} {operation} returned status {}",
            report.status_code
        ));
    }
    diagnostics
}

pub(super) fn combine_diagnostics<const N: usize>(
    diagnostic_groups: [Vec<String>; N],
) -> Vec<String> {
    sorted_unique_diagnostics(diagnostic_groups.into_iter().flatten().collect::<Vec<_>>())
}

pub(super) fn sorted_unique_diagnostics(mut diagnostics: Vec<String>) -> Vec<String> {
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}
