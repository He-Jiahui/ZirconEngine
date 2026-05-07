use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::{UiBatchKey, UiBatchPlan, UiPaintElement};

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
        let paint_entries = elements
            .iter()
            .enumerate()
            .map(|(paint_index, element)| UiRenderCachePaintEntry {
                node_id: element.node_id,
                paint_index,
                cache_generation: element.cache_generation,
                status: UiRenderCacheStatus::from_generation(element.cache_generation, reason),
                reason,
            })
            .collect::<Vec<_>>();

        let batch_entries = batch_plan
            .batches
            .iter()
            .enumerate()
            .map(|(batch_index, batch)| {
                let status = batch
                    .range
                    .first_element
                    .checked_add(batch.range.element_count.saturating_sub(1))
                    .and_then(|last_index| elements.get(batch.range.first_element..=last_index))
                    .filter(|batch_elements| {
                        reason == UiRenderCacheInvalidationReason::Unchanged
                            && batch_elements
                                .iter()
                                .all(|element| element.cache_generation.is_some())
                    })
                    .map(|_| UiRenderCacheStatus::Reused)
                    .unwrap_or(UiRenderCacheStatus::Rebuilt);

                UiRenderCacheBatchEntry {
                    batch_index,
                    batch_key: batch.key.clone(),
                    node_ids: batch.node_ids.clone(),
                    status,
                    reason,
                }
            })
            .collect::<Vec<_>>();

        let stats = UiRenderCacheStats::from_entries(&paint_entries, &batch_entries);
        Self {
            surface_generation,
            paint_entries,
            batch_entries,
            stats,
        }
    }
}

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

impl UiRenderCacheStats {
    fn from_entries(
        paint_entries: &[UiRenderCachePaintEntry],
        batch_entries: &[UiRenderCacheBatchEntry],
    ) -> Self {
        let reused_paint_count = paint_entries
            .iter()
            .filter(|entry| entry.status == UiRenderCacheStatus::Reused)
            .count();
        let reused_batch_count = batch_entries
            .iter()
            .filter(|entry| entry.status == UiRenderCacheStatus::Reused)
            .count();
        Self {
            paint_count: paint_entries.len(),
            reused_paint_count,
            rebuilt_paint_count: paint_entries.len() - reused_paint_count,
            batch_count: batch_entries.len(),
            reused_batch_count,
            rebuilt_batch_count: batch_entries.len() - reused_batch_count,
        }
    }
}
