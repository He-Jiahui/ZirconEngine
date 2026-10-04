//! 把测量和递归安排的临时存储留在运行时，重复布局清空长度而保留容量。

use std::cell::RefCell;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{AxisConstraint, DesiredSize, ResolvedAxisConstraint},
};

use super::super::taffy_bridge::TaffyLayoutBridgeScratch;

thread_local! {
    static TAFFY_ARRANGE_SCRATCH_POOL: RefCell<Vec<UiTaffyArrangeScratch>> =
        RefCell::default();
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct UiMeasurePostOrderEntry {
    pub node_id: UiNodeId,
    pub collapsed: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct UiContainerMeasureScratch {
    pub primary_extents: Vec<f32>,
    pub secondary_extents: Vec<f32>,
    pub column_counts: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
/// 每层递归安排独占一个 scratch；不能让子节点复用仍被父层持有的 children 或行缓存。
pub(crate) struct UiArrangeChildScratch {
    pub children: Vec<UiNodeId>,
    pub linear: UiLinearArrangeScratch,
    pub wrap_row_items: Vec<(UiNodeId, f32)>,
    pub wrap_content_desired: Vec<(UiNodeId, DesiredSize)>,
    pub masonry: UiMasonryArrangeScratch,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct UiLinearArrangeScratch {
    pub constraints: Vec<AxisConstraint>,
    pub resolved: Vec<ResolvedAxisConstraint>,
    pub priorities: Vec<i32>,
    pub active_indices: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct UiMasonryArrangeScratch {
    pub column_heights: Vec<f32>,
    pub column_counts: Vec<usize>,
}

#[derive(Debug, Default)]
pub(crate) struct UiTaffyArrangeScratch {
    pub layout_children: Vec<UiNodeId>,
    pub hidden_children: Vec<UiNodeId>,
    pub bridge: TaffyLayoutBridgeScratch,
}

impl UiTaffyArrangeScratch {
    pub fn clear_transient_lengths(&mut self) {
        self.layout_children.clear();
        self.hidden_children.clear();
        self.bridge.clear();
    }
}

pub(super) fn take_taffy_arrange_scratch() -> UiTaffyArrangeScratch {
    TAFFY_ARRANGE_SCRATCH_POOL.with(|pool| pool.borrow_mut().pop().unwrap_or_default())
}

/// 后端安排结束（包括失败）必须归还；线程局部池避免跨线程共享可变后端树。
pub(super) fn recycle_taffy_arrange_scratch(mut scratch: UiTaffyArrangeScratch) {
    scratch.clear_transient_lengths();
    TAFFY_ARRANGE_SCRATCH_POOL.with(|pool| pool.borrow_mut().push(scratch));
}

#[derive(Debug, Default)]
/// 与槽位索引同寿命的测量工作区，热帧重用旧容量；这些缓冲不代表已发布的布局结果。
pub(crate) struct UiLayoutPassWorkspace {
    pub post_order: Vec<UiMeasurePostOrderEntry>,
    pub child_desired: Vec<(UiNodeId, DesiredSize)>,
    pub container_scratch: UiContainerMeasureScratch,
    pub arrange_child_pool: Vec<UiArrangeChildScratch>,
    pub hidden_subtree_stack: Vec<UiNodeId>,
}

impl UiLayoutPassWorkspace {
    pub fn clear_transient_lengths(&mut self) {
        self.post_order.clear();
        self.child_desired.clear();
        self.container_scratch.primary_extents.clear();
        self.container_scratch.secondary_extents.clear();
        self.container_scratch.column_counts.clear();
    }
}
