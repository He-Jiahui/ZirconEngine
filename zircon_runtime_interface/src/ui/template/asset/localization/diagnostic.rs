use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiLocalizationDiagnostic {
    #[serde(default)]
    pub code: String,
    pub severity: UiLocalizationDiagnosticSeverity,
    pub path: String,
    pub message: String,
}

impl UiLocalizationDiagnostic {
    pub fn new(
        code: impl Into<String>,
        severity: UiLocalizationDiagnosticSeverity,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            path: path.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLocalizationDiagnosticSeverity {
    Error,
    Warning,
}
