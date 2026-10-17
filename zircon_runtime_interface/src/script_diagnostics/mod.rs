//! 脚本构建诊断的公共重导出入口；具体 DTO 与源码位置定义在 `diagnostic` 中。

#[path = "diagnostic.rs"]
mod diagnostic;

pub use diagnostic::{ScriptDiagnostic, ScriptDiagnosticSeverity, ScriptSourceLocation};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
