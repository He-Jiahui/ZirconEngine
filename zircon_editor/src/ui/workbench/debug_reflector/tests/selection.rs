//! 测试用选择投影，检查命中目标和捕获状态的优先级；当前未接生产pane。
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiPoint, surface::UiSurfaceDebugSnapshot,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct EditorUiDebugReflectorSelection {
    pub selected_node: Option<UiNodeId>,
    pub pick_point: Option<UiPoint>,
}

impl EditorUiDebugReflectorSelection {
    /// 测试选择优先采用本次顶层命中，缺失时才读取捕获的节点。
    pub(crate) fn from_snapshot_top_hit(snapshot: &UiSurfaceDebugSnapshot) -> Self {
        Self {
            selected_node: snapshot
                .pick_hit_test
                .as_ref()
                .and_then(|dump| dump.hit_path.target)
                .or(snapshot.capture.selected_node),
            pick_point: snapshot
                .capture
                .pick_query
                .as_ref()
                .map(|query| query.hit_point()),
        }
    }
}
