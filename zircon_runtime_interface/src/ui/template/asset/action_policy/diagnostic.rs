use serde::{Deserialize, Serialize};

use super::UiActionSideEffectClass;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiActionPolicyDiagnostic {
    pub severity: UiActionPolicyDiagnosticSeverity,
    pub node_id: String,
    pub binding_id: String,
    pub route: Option<String>,
    pub action: Option<String>,
    pub side_effect: UiActionSideEffectClass,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiActionPolicyDiagnosticSeverity {
    Error,
    Warning,
}
