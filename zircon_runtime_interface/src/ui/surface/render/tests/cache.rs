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
