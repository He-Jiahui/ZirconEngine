use crate::plugin::PluginModuleKind;

use super::report::NativePluginBehaviorHealth;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DiagnosticSeverity {
    Degraded,
    Invalid,
}

pub(super) struct ValidationDiagnostic {
    pub(super) severity: DiagnosticSeverity,
    pub(super) message: String,
}

pub(super) fn invalid_diagnostic(message: String) -> ValidationDiagnostic {
    ValidationDiagnostic {
        severity: DiagnosticSeverity::Invalid,
        message,
    }
}

pub(super) fn degraded_diagnostic(message: String) -> ValidationDiagnostic {
    ValidationDiagnostic {
        severity: DiagnosticSeverity::Degraded,
        message,
    }
}

pub(super) fn health_from_diagnostics(
    diagnostics: &[ValidationDiagnostic],
) -> NativePluginBehaviorHealth {
    if diagnostics.is_empty() {
        NativePluginBehaviorHealth::Clean
    } else if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Invalid)
    {
        NativePluginBehaviorHealth::Invalid
    } else {
        NativePluginBehaviorHealth::Degraded
    }
}

pub(super) fn module_kind_label(module_kind: PluginModuleKind) -> &'static str {
    match module_kind {
        PluginModuleKind::Runtime => "runtime",
        PluginModuleKind::Editor => "editor",
        PluginModuleKind::Native => "native",
        PluginModuleKind::Vm => "vm",
    }
}
