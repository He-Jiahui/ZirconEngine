use serde::{Deserialize, Serialize};

/// 绑定静态校验的诊断集合；警告保留在报告中，只有错误使校验失败。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingReport {
    #[serde(default)]
    pub diagnostics: Vec<UiBindingDiagnostic>,
}

impl UiBindingReport {
    pub fn is_valid(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != UiBindingDiagnosticSeverity::Error)
    }

    pub fn first_error(&self) -> Option<&UiBindingDiagnostic> {
        self.diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == UiBindingDiagnosticSeverity::Error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 静态绑定校验的一条定位记录；稳定代码负责机器识别，path 和两类 ID 指回资产声明。
pub struct UiBindingDiagnostic {
    pub code: UiBindingDiagnosticCode,
    pub severity: UiBindingDiagnosticSeverity,
    pub path: String,
    pub node_id: String,
    pub binding_id: String,
    pub message: String,
}

impl UiBindingDiagnostic {
    pub const fn error_code(&self) -> &'static str {
        self.code.error_code()
    }

    pub const fn diagnostic_id(&self) -> &'static str {
        self.code.diagnostic_id()
    }

    pub const fn localization_key(&self) -> &'static str {
        self.code.localization_key()
    }
}

/// 每种绑定错误固定映射到错误码、诊断 ID 和本地化键，供调用方稳定识别。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingDiagnosticCode {
    InvalidTarget,
    InvalidValueKind,
    UnresolvedRef,
    UnsupportedOperator,
    UnsupportedBindingMode,
}

#[derive(Clone, Copy)]
struct UiBindingDiagnosticIdentity {
    error_code: &'static str,
    diagnostic_id: &'static str,
    localization_key: &'static str,
}

impl UiBindingDiagnosticCode {
    pub const ALL: [Self; 5] = [
        Self::InvalidTarget,
        Self::InvalidValueKind,
        Self::UnresolvedRef,
        Self::UnsupportedOperator,
        Self::UnsupportedBindingMode,
    ];

    const fn identity(self) -> UiBindingDiagnosticIdentity {
        match self {
            Self::InvalidTarget => UiBindingDiagnosticIdentity {
                error_code: "invalid_target",
                diagnostic_id: "ZUI-BIND-0001",
                localization_key: "diagnostic.ui.binding.invalid_target",
            },
            Self::InvalidValueKind => UiBindingDiagnosticIdentity {
                error_code: "invalid_value_kind",
                diagnostic_id: "ZUI-BIND-0002",
                localization_key: "diagnostic.ui.binding.invalid_value_kind",
            },
            Self::UnresolvedRef => UiBindingDiagnosticIdentity {
                error_code: "unresolved_ref",
                diagnostic_id: "ZUI-BIND-0003",
                localization_key: "diagnostic.ui.binding.unresolved_ref",
            },
            Self::UnsupportedOperator => UiBindingDiagnosticIdentity {
                error_code: "unsupported_operator",
                diagnostic_id: "ZUI-BIND-0004",
                localization_key: "diagnostic.ui.binding.unsupported_operator",
            },
            Self::UnsupportedBindingMode => UiBindingDiagnosticIdentity {
                error_code: "unsupported_binding_mode",
                diagnostic_id: "ZUI-BIND-0005",
                localization_key: "diagnostic.ui.binding.unsupported_binding_mode",
            },
        }
    }

    pub const fn error_code(self) -> &'static str {
        self.identity().error_code
    }

    pub const fn diagnostic_id(self) -> &'static str {
        self.identity().diagnostic_id
    }

    pub const fn localization_key(self) -> &'static str {
        self.identity().localization_key
    }

    pub const fn as_str(self) -> &'static str {
        self.error_code()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 报告有效性由 Error 决定；Warning 保留为作者提示而不改变有效判定。
pub enum UiBindingDiagnosticSeverity {
    Error,
    Warning,
}

#[cfg(test)]
#[path = "tests/diagnostic.rs"]
mod tests;
