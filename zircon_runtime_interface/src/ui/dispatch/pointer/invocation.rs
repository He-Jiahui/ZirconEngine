use serde::{Deserialize, Serialize};

use crate::ui::dispatch::UiDispatchPhase;
use crate::ui::event_ui::UiNodeId;

use super::UiPointerDispatchEffect;

/// 保留非 Unhandled 处理器的节点、阶段和效果，供统一输入回复还原处理阶段，
/// 并让 UiSurface 按发起节点应用脏标记；组件默认行为不属于此列表。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerDispatchInvocation {
    pub node_id: UiNodeId,
    #[serde(default = "default_pointer_dispatch_phase")]
    pub phase: UiDispatchPhase,
    pub effect: UiPointerDispatchEffect,
}

// TODO: [CR-DISPATCH-0005] 旧载荷缺少 phase 时统一解释为 Target；
// 需确认旧记录是否可能来自 Bubble 或 PreviewTunnel，避免追溯阶段失真。
const fn default_pointer_dispatch_phase() -> UiDispatchPhase {
    UiDispatchPhase::Target
}
