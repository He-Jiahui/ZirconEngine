use super::hierarchy_pointer_layout::HierarchyPointerLayout;
use super::hierarchy_pointer_state::HierarchyPointerState;
use super::row_metrics::HierarchyRowMetrics;

#[derive(Default)]
pub(crate) struct HierarchyPointerBridge {
    pub(super) layout: HierarchyPointerLayout,
    pub(super) state: HierarchyPointerState,
    pub(super) row_metrics: HierarchyRowMetrics,
}
