use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 本地化收集或解析的问题带有资产路径，编辑器服务据报告呈现缺失引用和无效声明。
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
