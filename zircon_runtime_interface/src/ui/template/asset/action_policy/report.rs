use serde::{Deserialize, Serialize};

use super::UiActionPolicyDiagnostic;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiActionPolicyReport {
    pub diagnostics: Vec<UiActionPolicyDiagnostic>,
}

impl UiActionPolicyReport {
    pub fn is_allowed(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
