use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiResourceDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 静态收集诊断描述引用形状等资产问题；注册表缺失和运行时种类不匹配由解析器另行报告。
pub struct UiResourceDiagnostic {
    pub code: String,
    pub severity: UiResourceDiagnosticSeverity,
    pub message: String,
    pub path: String,
}
