use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 表达诊断消息的严重程度，供展示和失败摘要筛选使用。
///
/// # 状态关系
///
/// `Error` 严重级别本身不会把记录置为错误状态；记录阶段由资源管理器提交决定。Ready 记录也可以携带错误级别诊断。
///
/// # 参考
///
/// [`crate::resource::ResourceDiagnostic`]、[`crate::resource::ResourceRecord::failure_reason`]、[`crate::resource::ResourceState`]。
pub enum ResourceDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 携带资源导入或重载的诊断消息，作为记录快照的一部分传给展示端。
///
/// # 字段与状态
///
/// `severity` 选择消息级别，`message` 拥有可展示文本。记录的状态决定是否提供失败摘要，诊断级别不自行推进状态迁移。
///
/// # 使用时机
///
/// 构造本次导入或重载的诊断集，再由记录或管理器入口提交；仅创建诊断不会发布事件或改变载荷。
///
/// # 参考
///
/// [`Self::error`]、[`crate::resource::ResourceRecord::with_diagnostics`]、[`crate::resource::ResourceRecord::failure_reason`]。
pub struct ResourceDiagnostic {
    pub severity: ResourceDiagnosticSeverity,
    pub message: String,
}

impl ResourceDiagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: ResourceDiagnosticSeverity::Error,
            message: message.into(),
        }
    }
}
