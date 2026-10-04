use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiTreeId};

use super::{
    UiBatchKey, UiBatchPlan, UiBatchSplitReason, UiRenderCachePlan, UiRenderExtract,
    UiRenderFrameExtract, UiRenderVisualizerSnapshot, UiRendererParitySnapshot,
};

#[cfg(test)]
#[path = "debug/tests/performance_tests.rs"]
mod performance_tests;

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
        Self::from_paint_elements(extract.tree_id.clone(), extract.list.to_paint_elements())
    }

    /// 从接口帧重建 Editor/Runtime 可读取的调试投影，不读取后端私有缓存状态。
    pub fn from_render_frame_extract(extract: &UiRenderFrameExtract) -> Self {
        Self::from_paint_elements(extract.tree_id.clone(), extract.list.to_paint_elements())
    }

    fn from_paint_elements(tree_id: UiTreeId, elements: Vec<super::UiPaintElement>) -> Self {
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
            tree_id.clone(),
            &elements,
            &plan,
        );
        let stats = UiRenderDebugStatsV2 {
            element_count: elements.len(),
            batch_count: plan.stats.batch_count,
            draw_call_count: plan.stats.draw_call_count,
        };
        let batches = plan
            .batches
            .into_iter()
            .map(|batch| UiRenderBatchDebugEntry {
                layer: batch.layer,
                key: batch.key,
                first_element: batch.range.first_element,
                element_count: batch.range.element_count,
                source_indices: batch.source_indices,
                node_ids: batch.node_ids,
                split_reason: batch.split_reason,
            })
            .collect();
        Self {
            tree_id,
            stats,
            batches,
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
    pub layer: i32,
    pub key: UiBatchKey,
    pub first_element: usize,
    pub element_count: usize,
    pub source_indices: Vec<usize>,
    pub node_ids: Vec<UiNodeId>,
    pub split_reason: UiBatchSplitReason,
}
