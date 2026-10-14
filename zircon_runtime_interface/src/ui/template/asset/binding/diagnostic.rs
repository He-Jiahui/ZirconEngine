use serde::{Deserialize, Serialize};

/// 绑定静态校验的诊断集合；警告保留在报告中，只有错误使校验失败。
///
/// # 使用时机
/// Runtime 资产绑定校验器返回此报告，Editor 的报告服务将其交给诊断展示层。
///
/// # 约束
/// 报告可由调用方直接构造或反序列化；这不会自动执行校验，需通过 [`Self::is_valid`] 判断是否含错误。
/// 缺省的 `diagnostics` 字段按空列表反序列化。
///
/// # 参考
/// Runtime `binding/validation.rs::collect_asset_binding_report` 构造报告，
/// `validate_asset_bindings` 读取首个错误；Editor `UiEditorAssetService::collect_binding_report` 暴露完整报告。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingReport {
    /// 静态校验产生的诊断条目；空集合本身不表示曾运行过校验。
    #[serde(default)]
    pub diagnostics: Vec<UiBindingDiagnostic>,
}

impl UiBindingReport {
    /// 判断报告中是否不存在错误级别的条目；警告不改变此结果。
    pub fn is_valid(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != UiBindingDiagnosticSeverity::Error)
    }

    /// 返回列表中第一条错误；没有错误时返回 `None`。
    pub fn first_error(&self) -> Option<&UiBindingDiagnostic> {
        self.diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == UiBindingDiagnosticSeverity::Error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 静态绑定校验的一条定位记录；稳定代码负责机器识别，path 和两类 ID 指回资产声明。
///
/// # 使用时机
/// Runtime 校验器把错误码、严重性、资产路径与人类可读消息一起写入 [`UiBindingReport`]；
/// Editor `asset_editor/diagnostics/binding.rs::map_binding_diagnostic` 将严重性、稳定代码和消息转换为编辑器诊断。
pub struct UiBindingDiagnostic {
    /// 稳定的校验类别；具体机器码和本地化键由下方查询方法提供。
    pub code: UiBindingDiagnosticCode,
    /// 决定报告有效性，并由 Editor 映射为编辑器错误或警告。
    pub severity: UiBindingDiagnosticSeverity,
    /// 校验器报告的问题位置，格式由产生该条目的资产校验路径决定。
    pub path: String,
    /// 发生问题的资产节点标识。
    pub node_id: String,
    /// 发生问题的绑定标识。
    pub binding_id: String,
    /// 面向作者的说明文本；机器分支应使用 `code`，不要解析此文本。
    pub message: String,
}

impl UiBindingDiagnostic {
    /// 返回稳定的诊断错误码，用于机器侧分类。
    pub const fn error_code(&self) -> &'static str {
        self.code.error_code()
    }

    /// 返回稳定的诊断 ID，供诊断系统关联同一类别的问题。
    pub const fn diagnostic_id(&self) -> &'static str {
        self.code.diagnostic_id()
    }

    /// 返回本地化资源键；实际消息文本仍由校验器写入 `message`。
    pub const fn localization_key(&self) -> &'static str {
        self.code.localization_key()
    }
}

/// 每种绑定错误固定映射到错误码、诊断 ID 和本地化键，供调用方稳定识别。
///
/// Runtime 静态校验器产生这些类别，Editor 诊断映射保留其错误码；Serde 名称使用 snake_case。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingDiagnosticCode {
    /// 目标声明缺失、冲突或不满足绑定写入契约。
    InvalidTarget,
    /// 表达式结果类型与目标或运算符要求不兼容。
    InvalidValueKind,
    /// 参数、属性或 control 属性引用无法在静态校验上下文中解析。
    UnresolvedRef,
    /// 表达式包含当前绑定语法不支持的运算符。
    UnsupportedOperator,
    /// 绑定模式没有对应的 Runtime 执行器。
    UnsupportedBindingMode,
}

#[derive(Clone, Copy)]
struct UiBindingDiagnosticIdentity {
    error_code: &'static str,
    diagnostic_id: &'static str,
    localization_key: &'static str,
}

impl UiBindingDiagnosticCode {
    /// 按稳定定义顺序列出全部诊断码；契约测试同时核对每项身份映射。
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
    /// 使 [`UiBindingReport::is_valid`] 返回 `false`。
    Error,
    /// 保留在报告中，但不使报告判为无效。
    Warning,
}

#[cfg(test)]
#[path = "tests/diagnostic.rs"]
mod tests;
