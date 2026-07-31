use crate::plugin::PluginModuleKind;

use super::super::{
    NativePluginBehaviorCallReport, NativePluginLoadProjection, NativePluginLoadReport,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use super::keys::module_kind_label;

pub(super) type NativePluginBehaviorDiagnosticResult<T> =
    std::result::Result<T, NativePluginBehaviorDiagnosticError>;

#[derive(Debug)]
pub(super) enum NativePluginBehaviorDiagnosticError {
    FailedStatus {
        label: String,
        status_code: u32,
        diagnostics: Vec<String>,
    },
}

impl std::fmt::Display for NativePluginBehaviorDiagnosticError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedStatus { diagnostics, .. } => formatter.write_str(&diagnostics.join("; ")),
        }
    }
}

impl std::error::Error for NativePluginBehaviorDiagnosticError {}

pub(super) fn load_projected_report_diagnostics(
    report: &NativePluginLoadReport,
    projection: &NativePluginLoadProjection,
) -> Vec<String> {
    let mut diagnostics = report.diagnostics().to_vec();
    diagnostics.extend(projection.descriptor_diagnostics().iter().cloned());
    diagnostics.extend(projection.entry_diagnostics().iter().cloned());
    diagnostics
}

pub(super) fn unloaded_plugin_error(plugin_id: &str, module_kind: PluginModuleKind) -> String {
    format!(
        "plugin {plugin_id} is not loaded in the {} live host; run Hot Reload after building its native dynamic package",
        module_kind_label(module_kind)
    )
}

pub(super) fn projected_diagnostics_for_plugin(
    projection: &NativePluginLoadProjection,
    plugin_id: &str,
    module_kind: PluginModuleKind,
) -> Vec<String> {
    match module_kind {
        PluginModuleKind::Runtime => projection.runtime_diagnostics_for_plugin(plugin_id),
        PluginModuleKind::Editor => projection.editor_diagnostics_for_plugin(plugin_id),
        PluginModuleKind::Native | PluginModuleKind::Vm => {
            projection.diagnostics_for_plugin(plugin_id)
        }
    }
}

pub(super) fn diagnostics_from_behavior_report(
    label: &str,
    report: NativePluginBehaviorCallReport,
) -> NativePluginBehaviorDiagnosticResult<Vec<String>> {
    if report.status_code == ZIRCON_NATIVE_PLUGIN_STATUS_OK {
        return Ok(report.diagnostics);
    }
    let status_code = report.status_code;
    let diagnostics = if report.diagnostics.is_empty() {
        vec![format!("{label} returned status {status_code}")]
    } else {
        report
            .diagnostics
            .into_iter()
            .map(|message| format!("{label}: {message}"))
            .collect()
    };
    Err(NativePluginBehaviorDiagnosticError::FailedStatus {
        label: label.to_string(),
        status_code,
        diagnostics,
    })
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
