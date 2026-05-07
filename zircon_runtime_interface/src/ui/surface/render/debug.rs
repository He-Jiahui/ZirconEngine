use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiTreeId};

use super::{
    UiBatchKey, UiBatchPlan, UiBatchSplitReason, UiRenderCachePlan, UiRenderExtract,
    UiRenderVisualizerSnapshot, UiRendererParitySnapshot,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiRenderDebugSnapshot {
    pub tree_id: UiTreeId,
    pub stats: UiRenderDebugStatsV2,
    pub batches: Vec<UiRenderBatchDebugEntry>,
    #[serde(default)]
    pub cache: UiRenderCachePlan,
    #[serde(default)]
    pub parity: UiRendererParitySnapshot,
    #[serde(default)]
    pub visualizer: UiRenderVisualizerSnapshot,
}

impl UiRenderDebugSnapshot {
    pub fn from_render_extract(extract: &UiRenderExtract) -> Self {
        let elements = extract.list.to_paint_elements();
        let plan = UiBatchPlan::from_paint_elements(&elements);
        let cache = UiRenderCachePlan::from_paint_elements_and_batches(
            0,
            &elements,
            &plan,
            Default::default(),
        );
        let visualizer =
            UiRenderVisualizerSnapshot::from_paint_elements_batches_cache(&elements, &plan, &cache);
        let parity = UiRendererParitySnapshot::from_paint_elements_batches(
            extract.tree_id.clone(),
            &elements,
            &plan,
        );
        Self {
            tree_id: extract.tree_id.clone(),
            stats: UiRenderDebugStatsV2 {
                element_count: elements.len(),
                batch_count: plan.stats.batch_count,
                draw_call_count: plan.stats.draw_call_count,
            },
            batches: plan
                .batches
                .iter()
                .map(|batch| UiRenderBatchDebugEntry {
                    key: batch.key.clone(),
                    first_element: batch.range.first_element,
                    element_count: batch.range.element_count,
                    node_ids: batch.node_ids.clone(),
                    split_reason: batch.split_reason,
                })
                .collect(),
            cache,
            parity,
            visualizer,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRenderDebugStatsV2 {
    pub element_count: usize,
    pub batch_count: usize,
    pub draw_call_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRenderBatchDebugEntry {
    pub key: UiBatchKey,
    pub first_element: usize,
    pub element_count: usize,
    pub node_ids: Vec<UiNodeId>,
    pub split_reason: UiBatchSplitReason,
}
