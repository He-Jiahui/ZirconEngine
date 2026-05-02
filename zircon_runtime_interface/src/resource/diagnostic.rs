use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceDiagnostic {
    pub severity: ResourceDiagnosticSeverity,
    pub message: String,
}

impl ResourceDiagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: ResourceDiagnosticSeverity::Error,
            message: message.into(),
        }
    }
}
