use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 静态声明诊断的严重度，序列化为 snake_case；不等同于运行时查找结果。
pub enum UiResourceDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 静态收集诊断描述引用形状等资产问题；注册表缺失和运行时种类不匹配由解析器另行报告。
pub struct UiResourceDiagnostic {
    /// 供调用方识别的诊断代码。
    pub code: String,
    /// 声明问题的严重度，不触发加载或异常处理。
    pub severity: UiResourceDiagnosticSeverity,
    /// 面向作者的拥有消息；调用方不必借用原始文档。
    pub message: String,
    /// 复制的声明位置标签，用于诊断定位。
    pub path: String,
}
