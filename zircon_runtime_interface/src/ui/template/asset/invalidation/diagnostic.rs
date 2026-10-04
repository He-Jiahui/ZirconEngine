use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiInvalidationDiagnosticSeverity {
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 失效分类发现异常时保留阶段与源路径，供报告消费者呈现而不改变已判定的阶段集合。
pub struct UiInvalidationDiagnostic {
    pub code: String,
    pub severity: UiInvalidationDiagnosticSeverity,
    pub message: String,
}
