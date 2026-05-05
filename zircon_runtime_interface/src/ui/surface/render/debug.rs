use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiTreeId};

use super::{UiBatchKey, UiBatchPlan, UiBatchSplitReason, UiRenderExtract};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiRenderDebugSnapshot {
    pub tree_id: UiTreeId,
    pub stats: UiRenderDebugStatsV2,
    pub batches: Vec<UiRenderBatchDebugEntry>,
}

impl UiRenderDebugSnapshot {
    pub fn from_render_extract(extract: &UiRenderExtract) -> Self {
        let elements = extract.list.to_paint_elements();
        let plan = UiBatchPlan::from_paint_elements(&elements);
        Self {
            tree_id: extract.tree_id.clone(),
            stats: UiRenderDebugStatsV2 {
                element_count: elements.len(),
                batch_count: plan.stats.batch_count,
                draw_call_count: plan.stats.draw_call_count,
            },
            batches: plan
                .batches
                .into_iter()
                .map(|batch| UiRenderBatchDebugEntry {
                    key: batch.key,
                    first_element: batch.range.first_element,
                    element_count: batch.range.element_count,
                    node_ids: batch.node_ids,
                    split_reason: batch.split_reason,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRenderDebugStatsV2 {
    pub element_count: usize,
    pub batch_count: usize,
    pub draw_call_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRenderBatchDebugEntry {
    pub key: UiBatchKey,
    pub first_element: usize,
    pub element_count: usize,
    pub node_ids: Vec<UiNodeId>,
    pub split_reason: UiBatchSplitReason,
}
