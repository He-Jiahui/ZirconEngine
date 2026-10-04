use serde::{Deserialize, Serialize};

use super::UiActionSideEffectClass;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 宿主动作准入校验的一条结果，保留节点、绑定、路由与推断副作用以便定位源声明。
pub struct UiActionPolicyDiagnostic {
    pub severity: UiActionPolicyDiagnosticSeverity,
    pub node_id: String,
    pub binding_id: String,
    pub route: Option<String>,
    pub action: Option<String>,
    pub side_effect: UiActionSideEffectClass,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 动作策略诊断随包报告序列化，级别描述错误或提示；准入结果仍由策略报告的诊断集合判定。
pub enum UiActionPolicyDiagnosticSeverity {
    Error,
    Warning,
}
