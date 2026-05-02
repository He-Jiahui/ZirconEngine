use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingReport {
    #[serde(default)]
    pub diagnostics: Vec<UiBindingDiagnostic>,
}

impl UiBindingReport {
    pub fn is_valid(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != UiBindingDiagnosticSeverity::Error)
    }

    pub fn first_error(&self) -> Option<&UiBindingDiagnostic> {
        self.diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == UiBindingDiagnosticSeverity::Error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingDiagnostic {
    pub code: UiBindingDiagnosticCode,
    pub severity: UiBindingDiagnosticSeverity,
    pub path: String,
    pub node_id: String,
    pub binding_id: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingDiagnosticCode {
    InvalidTarget,
    InvalidValueKind,
    UnresolvedRef,
    UnsupportedOperator,
}

impl UiBindingDiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidTarget => "invalid_target",
            Self::InvalidValueKind => "invalid_value_kind",
            Self::UnresolvedRef => "unresolved_ref",
            Self::UnsupportedOperator => "unsupported_operator",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingDiagnosticSeverity {
    Error,
    Warning,
}
