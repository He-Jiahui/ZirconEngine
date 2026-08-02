use super::row_metrics::{HierarchyRowMetrics, hierarchy_content_height};

pub(super) fn content_height(item_count: usize, metrics: HierarchyRowMetrics) -> f32 {
    hierarchy_content_height(item_count, metrics)
}
