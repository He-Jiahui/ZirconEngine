//! Capability validation result for an editor-plugin catalog.

use zircon_runtime_interface::RegistrationDiagnostic;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorCapabilityReport {
    pub diagnostics: Vec<RegistrationDiagnostic>,
}

impl EditorCapabilityReport {
    pub fn is_success(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(RegistrationDiagnostic::is_error)
    }
}
