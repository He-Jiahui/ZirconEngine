use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiInvalidationDiagnosticSeverity {
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiInvalidationDiagnostic {
    pub code: String,
    pub severity: UiInvalidationDiagnosticSeverity,
    pub message: String,
}
