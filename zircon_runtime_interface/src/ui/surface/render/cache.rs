use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::{UiBatchKey, UiBatchPlan, UiPaintElement};

#[cfg(test)]
#[path = "cache/tests/performance_tests.rs"]
mod performance_tests;

/// 把外部失效原因与 generation 投影为诊断状态；该 DTO 不持有或调度后端资源缓存。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiRenderCachePlan {
    pub surface_generation: u64,
    pub paint_entries: Vec<UiRenderCachePaintEntry>,
    pub batch_entries: Vec<UiRenderCacheBatchEntry>,
    pub stats: UiRenderCacheStats,
}

impl UiRenderCachePlan {
    pub fn from_paint_elements_and_batches(
        surface_generation: u64,
        elements: &[UiPaintElement],
        batch_plan: &UiBatchPlan,
        reason: UiRenderCacheInvalidationReason,
    ) -> Self {
        let mut reused_paint_count = 0;
        let paint_entries = elements
            .iter()
            .enumerate()
            .map(|(paint_index, element)| {
                let status = UiRenderCacheStatus::from_generation(element.cache_generation, reason);
                reused_paint_count += usize::from(status == UiRenderCacheStatus::Reused);
                UiRenderCachePaintEntry {
                    node_id: element.node_id,
                    paint_index,
                    cache_generation: element.cache_generation,
                    status,
                    reason,
                }
            })
            .collect::<Vec<_>>();

        let mut reused_batch_count = 0;
        let batch_entries = batch_plan
            .batches
            .iter()
            .enumerate()
            .map(|(batch_index, batch)| {
                let status = batch_cache_status(elements, &batch.source_indices, reason);
                reused_batch_count += usize::from(status == UiRenderCacheStatus::Reused);

                UiRenderCacheBatchEntry {
                    batch_index,
                    batch_key: batch.key.clone(),
                    node_ids: batch.node_ids.clone(),
                    status,
                    reason,
                }
            })
            .collect::<Vec<_>>();

        let stats = UiRenderCacheStats {
            paint_count: paint_entries.len(),
            reused_paint_count,
            rebuilt_paint_count: paint_entries.len() - reused_paint_count,
            batch_count: batch_entries.len(),
            reused_batch_count,
            rebuilt_batch_count: batch_entries.len() - reused_batch_count,
        };
        Self {
            surface_generation,
            paint_entries,
            batch_entries,
            stats,
        }
    }
}

fn batch_cache_status(
    elements: &[UiPaintElement],
    source_indices: &[usize],
    reason: UiRenderCacheInvalidationReason,
) -> UiRenderCacheStatus {
    // 批次只有在整帧未失效且每个来源元素都带稳定 generation 时才投影为复用。
    if reason != UiRenderCacheInvalidationReason::Unchanged {
        return UiRenderCacheStatus::Rebuilt;
    }

    if source_indices.iter().all(|&source_index| {
        elements
            .get(source_index)
            .is_some_and(|element| element.cache_generation.is_some())
    }) {
        UiRenderCacheStatus::Reused
    } else {
        UiRenderCacheStatus::Rebuilt
    }
}

#[cfg(test)]
#[path = "tests/cache.rs"]
mod tests;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRenderCachePaintEntry {
    pub node_id: UiNodeId,
    pub paint_index: usize,
    pub cache_generation: Option<u64>,
    pub status: UiRenderCacheStatus,
    pub reason: UiRenderCacheInvalidationReason,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRenderCacheBatchEntry {
    pub batch_index: usize,
    pub batch_key: UiBatchKey,
    pub node_ids: Vec<UiNodeId>,
    pub status: UiRenderCacheStatus,
    pub reason: UiRenderCacheInvalidationReason,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRenderCacheStatus {
    #[default]
    Rebuilt,
    Reused,
}

impl UiRenderCacheStatus {
    fn from_generation(generation: Option<u64>, reason: UiRenderCacheInvalidationReason) -> Self {
        if generation.is_some() && reason == UiRenderCacheInvalidationReason::Unchanged {
            Self::Reused
        } else {
            Self::Rebuilt
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRenderCacheInvalidationReason {
    #[default]
    Unchanged,
    SurfaceGenerationChanged,
    NodeDirty,
    LayoutGeometryChanged,
    ClipStateChanged,
    ResourceRevisionChanged,
    TextShapeChanged,
    ForcedRebuild,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRenderCacheStats {
    pub paint_count: usize,
    pub reused_paint_count: usize,
    pub rebuilt_paint_count: usize,
    pub batch_count: usize,
    pub reused_batch_count: usize,
    pub rebuilt_batch_count: usize,
}
