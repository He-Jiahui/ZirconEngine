//! 脚本构建诊断的可序列化表示，由 Editor 日志投影为严重度和源码跳转。
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

/// 可选的源码跳转位置；Editor 日志投影将其转换为可点击的脚本位置。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScriptSourceLocation {
    pub path: String,
    pub line: u32,
    pub column: u32,
}

impl ScriptSourceLocation {
    pub fn new(path: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            path: path.into(),
            line,
            column,
        }
    }
}

/// 跨构建步骤传递代码、模块、消息及可选源码位置；展示和去重由 Editor 决定。
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScriptDiagnostic {
    pub severity: ScriptDiagnosticSeverity,
    pub code: String,
    pub module: String,
    pub message: String,
    pub location: Option<ScriptSourceLocation>,
}

impl ScriptDiagnostic {
    pub fn new(
        severity: ScriptDiagnosticSeverity,
        code: impl Into<String>,
        module: impl Into<String>,
        message: impl Into<String>,
        location: Option<ScriptSourceLocation>,
    ) -> Self {
        Self {
            severity,
            code: code.into(),
            module: module.into(),
            message: message.into(),
            location,
        }
    }
}
