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
        let mut message = String::with_capacity(
            "editor plugin `` requires missing capability ``"
                .len()
                .saturating_add(plugin_id.len())
                .saturating_add(capability.len()),
        );
        message.push_str("editor plugin `");
        message.push_str(&plugin_id);
        message.push_str("` requires missing capability `");
        message.push_str(&capability);
        message.push('`');
        Self::new(
            RegistrationDiagnosticSeverity::Error,
            "editor.capability.missing",
            plugin_id,
            message,
        )
    }

    pub fn is_error(&self) -> bool {
        self.severity == RegistrationDiagnosticSeverity::Error
    }
}

#[cfg(test)]
#[path = "tests/plugin_diagnostics.rs"]
mod tests;
