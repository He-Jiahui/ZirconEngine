use serde::{Deserialize, Serialize};

use super::UiActionPolicyDiagnostic;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiActionPolicyReport {
    pub diagnostics: Vec<UiActionPolicyDiagnostic>,
}

impl UiActionPolicyReport {
    /// 仅当本次策略报告没有诊断时判为允许；调用方可同时保留诊断原文供编辑器展示。
    pub fn is_allowed(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
