//! 统一输入分发的公开门面：管理器保存跨事件状态，专用分发器保存各自回调。
//!
//! # 使用与生命周期
//! 宿主持有管理器以延续指针、文本与计时状态；派发时借用当前管理器及传入表面。
//! 阶段表提供策略的诊断投影，实际分发由表面输入与各专用分发器决定。
//! # 参考
//! 见 [输入管理](input_manager) 与去重集合的实例范围说明。

#[path = "input_manager/mod.rs"]
mod input_manager;
#[path = "navigation/mod.rs"]
mod navigation;
#[path = "pointer/mod.rs"]
mod pointer;
#[path = "visited_node_set.rs"]
mod visited_node_set;

pub use input_manager::{
    route_policy_uses_stage, route_stage_name, route_stage_names_for_policy, UiActivePointerEntry,
    UiActivePointerTable, UiInputDispatchOutcome, UiInputManager, UiInputRouteStage,
    UiInputTimerState, DEFAULT_TOOLTIP_DELAY_MS, DEFAULT_TOOLTIP_INTRO_DURATION_MS,
    UI_INPUT_ROUTE_ORDER,
};
pub(in crate::ui) use input_manager::{
    UiTextDocumentSession, UiTextDocumentSessionError, UiTextHistoryCommit, UiTextHistoryDirection,
};
pub use navigation::UiNavigationDispatcher;
pub use pointer::UiPointerDispatcher;
