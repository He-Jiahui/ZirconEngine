use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistrationDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrationDiagnostic {
    pub severity: RegistrationDiagnosticSeverity,
    pub code: String,
    pub plugin_id: String,
    pub message: String,
}

impl RegistrationDiagnostic {
    pub fn new(
        severity: RegistrationDiagnosticSeverity,
        code: impl Into<String>,
        plugin_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code: code.into(),
            plugin_id: plugin_id.into(),
            message: message.into(),
        }
    }

    pub fn missing_capability(plugin_id: impl Into<String>, capability: impl Into<String>) -> Self {
        let plugin_id = plugin_id.into();
        let capability = capability.into();
        Self::new(
            RegistrationDiagnosticSeverity::Error,
            "editor.capability.missing",
            plugin_id.clone(),
            format!("editor plugin `{plugin_id}` requires missing capability `{capability}`"),
        )
    }

    pub fn is_error(&self) -> bool {
        self.severity == RegistrationDiagnosticSeverity::Error
    }
}
