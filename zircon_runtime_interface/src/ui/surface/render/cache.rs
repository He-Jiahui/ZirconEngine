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
                let status = batch_cache_status(elements, &batch.source_indices, reason);

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

fn batch_cache_status(
    elements: &[UiPaintElement],
    source_indices: &[usize],
    reason: UiRenderCacheInvalidationReason,
) -> UiRenderCacheStatus {
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
mod tests {
    use super::*;
    use crate::ui::layout::UiGeometry;
    use crate::ui::surface::{UiPaintEffects, UiPaintPayload};

    fn element(node_id: u64, generation: Option<u64>) -> UiPaintElement {
        UiPaintElement {
            node_id: UiNodeId::new(node_id),
            geometry: UiGeometry::default(),
            clip: None,
            z_index: 0,
            paint_order: node_id,
            payload: UiPaintPayload::Empty,
            effects: UiPaintEffects::default(),
            cache_generation: generation,
            debug_label: None,
        }
    }

    #[test]
    fn cache_plan_reuses_a_batch_without_collecting_source_elements() {
        let mut elements = vec![element(1, Some(7)), element(2, Some(7))];
        let batch_plan = UiBatchPlan::from_paint_elements(&elements);

        let cache_plan = UiRenderCachePlan::from_paint_elements_and_batches(
            1,
            &elements,
            &batch_plan,
            UiRenderCacheInvalidationReason::Unchanged,
        );
        assert_eq!(
            cache_plan.batch_entries[0].status,
            UiRenderCacheStatus::Reused
        );

        elements[1].cache_generation = None;
        let cache_plan = UiRenderCachePlan::from_paint_elements_and_batches(
            2,
            &elements,
            &batch_plan,
            UiRenderCacheInvalidationReason::Unchanged,
        );
        assert_eq!(
            cache_plan.batch_entries[0].status,
            UiRenderCacheStatus::Rebuilt
        );
    }

    #[test]
    fn cache_plan_rebuilds_when_a_batch_source_index_is_missing() {
        let elements = vec![element(3, Some(9))];
        let mut batch_plan = UiBatchPlan::from_paint_elements(&elements);
        batch_plan.batches[0].source_indices.push(99);

        let cache_plan = UiRenderCachePlan::from_paint_elements_and_batches(
            3,
            &elements,
            &batch_plan,
            UiRenderCacheInvalidationReason::Unchanged,
        );
        assert_eq!(
            cache_plan.batch_entries[0].status,
            UiRenderCacheStatus::Rebuilt
        );
    }

    #[test]
    fn cache_plan_rebuilds_batches_for_non_unchanged_reasons() {
        let elements = vec![element(4, Some(11))];
        let batch_plan = UiBatchPlan::from_paint_elements(&elements);

        let cache_plan = UiRenderCachePlan::from_paint_elements_and_batches(
            4,
            &elements,
            &batch_plan,
            UiRenderCacheInvalidationReason::NodeDirty,
        );
        assert_eq!(
            cache_plan.batch_entries[0].status,
            UiRenderCacheStatus::Rebuilt
        );
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
