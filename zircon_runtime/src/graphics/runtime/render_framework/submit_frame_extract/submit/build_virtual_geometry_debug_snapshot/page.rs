use std::collections::{BTreeMap, HashSet};

use super::super::super::frame_submission_context::FrameSubmissionContext;
use super::support::saturated_u32_len;
use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryExtract, RenderVirtualGeometryPageRequestInspection,
    RenderVirtualGeometryResidentPageInspection,
};
use crate::graphics::VisibilityVirtualGeometryPageUploadPlan;

pub(super) type PageSizeIndex = BTreeMap<u32, u64>;

pub(super) fn build_page_size_index(extract: &RenderVirtualGeometryExtract) -> PageSizeIndex {
    let mut page_sizes = BTreeMap::new();
    for page in &extract.pages {
        page_sizes.entry(page.page_id).or_insert(page.size_bytes);
    }
    page_sizes
}

pub(super) fn build_cull_input_snapshot(
    extract: &RenderVirtualGeometryExtract,
    page_upload_plan: &VisibilityVirtualGeometryPageUploadPlan,
    available_page_slot_count: usize,
    evictable_page_count: usize,
) -> RenderVirtualGeometryCullInputSnapshot {
    RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: extract.cluster_budget,
        page_budget: extract.page_budget,
        instance_count: saturated_u32_len(extract.instances.len()),
        cluster_count: saturated_u32_len(extract.clusters.len()),
        page_count: saturated_u32_len(extract.pages.len()),
        visible_entity_count: unique_extract_entity_count(extract),
        visible_cluster_count: saturated_u32_len(extract.clusters.len()),
        resident_page_count: saturated_u32_len(page_upload_plan.resident_pages.len()),
        pending_page_request_count: saturated_u32_len(page_upload_plan.requested_pages.len()),
        available_page_slot_count: saturated_u32_len(available_page_slot_count),
        evictable_page_count: saturated_u32_len(evictable_page_count),
        debug: extract.debug,
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareDerivedFrameOwned,
    }
}

fn unique_extract_entity_count(extract: &RenderVirtualGeometryExtract) -> u32 {
    if !extract.instances.is_empty() {
        return unique_entity_count(
            extract.instances.iter().map(|instance| instance.entity),
            extract.instances.len(),
        );
    }

    unique_entity_count(
        extract.clusters.iter().map(|cluster| cluster.entity),
        extract.clusters.len(),
    )
}

fn unique_entity_count(entities: impl Iterator<Item = u64>, capacity: usize) -> u32 {
    let mut unique_entities = HashSet::with_capacity(capacity);
    unique_entities.extend(entities);
    saturated_u32_len(unique_entities.len())
}

pub(super) fn build_resident_page_inspections(
    page_upload_plan: &VisibilityVirtualGeometryPageUploadPlan,
    page_size_index: &PageSizeIndex,
) -> Vec<RenderVirtualGeometryResidentPageInspection> {
    page_upload_plan
        .resident_pages
        .iter()
        .enumerate()
        .map(
            |(slot, page_id)| RenderVirtualGeometryResidentPageInspection {
                page_id: *page_id,
                slot: u32::try_from(slot).unwrap_or(u32::MAX),
                size_bytes: page_size_bytes(page_size_index, *page_id),
            },
        )
        .collect()
}

pub(super) fn build_available_page_slots(
    extract: &RenderVirtualGeometryExtract,
    page_upload_plan: &VisibilityVirtualGeometryPageUploadPlan,
) -> Vec<u32> {
    let resident_slot_count = page_upload_plan.resident_pages.len() as u32;
    (resident_slot_count..extract.page_budget)
        .take(page_upload_plan.requested_pages.len())
        .collect()
}

pub(super) fn build_pending_page_request_inspections(
    context: &FrameSubmissionContext,
    page_upload_plan: &VisibilityVirtualGeometryPageUploadPlan,
    available_page_slots: &[u32],
    page_size_index: &PageSizeIndex,
) -> Vec<RenderVirtualGeometryPageRequestInspection> {
    page_upload_plan
        .requested_pages
        .iter()
        .enumerate()
        .map(
            |(frontier_rank, page_id)| RenderVirtualGeometryPageRequestInspection {
                page_id: *page_id,
                size_bytes: page_size_bytes(page_size_index, *page_id),
                generation: context.predicted_generation(),
                frontier_rank: u32::try_from(frontier_rank).unwrap_or(u32::MAX),
                assigned_slot: available_page_slots.get(frontier_rank).copied(),
                recycled_page_id: None,
            },
        )
        .collect()
}

pub(super) fn build_evictable_page_inspections(
    page_upload_plan: &VisibilityVirtualGeometryPageUploadPlan,
    resident_page_inspections: &[RenderVirtualGeometryResidentPageInspection],
    page_size_index: &PageSizeIndex,
) -> Vec<RenderVirtualGeometryResidentPageInspection> {
    let mut resident_by_page_id = BTreeMap::new();
    for inspection in resident_page_inspections {
        resident_by_page_id
            .entry(inspection.page_id)
            .or_insert((inspection.slot, inspection.size_bytes));
    }

    page_upload_plan
        .evictable_pages
        .iter()
        .map(|page_id| {
            resident_by_page_id
                .get(page_id)
                .map(
                    |(slot, size_bytes)| RenderVirtualGeometryResidentPageInspection {
                        page_id: *page_id,
                        slot: *slot,
                        size_bytes: *size_bytes,
                    },
                )
                .unwrap_or(RenderVirtualGeometryResidentPageInspection {
                    page_id: *page_id,
                    slot: u32::MAX,
                    size_bytes: page_size_bytes(page_size_index, *page_id),
                })
        })
        .collect()
}

fn page_size_bytes(page_size_index: &PageSizeIndex, page_id: u32) -> u64 {
    page_size_index.get(&page_id).copied().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::unique_entity_count;

    #[test]
    fn page_inspections_use_prebuilt_page_and_resident_indices() {
        let source = include_str!("page.rs");

        assert!(source.contains("build_page_size_index"));
        assert!(source.contains("resident_by_page_id"));
        assert!(!source.contains(concat!("extract", ".pages", ".iter()", ".find")));
        assert!(!source.contains(concat!("resident_page_inspections", ".iter()", ".find")));
    }

    #[test]
    fn optimization_batch_20260830cu_unique_entity_hash_count_preserves_duplicate_semantics() {
        let entities = [9, 2, 9, 4, 2, 7, 4, 7, 11];

        assert_eq!(unique_entity_count(entities.into_iter(), entities.len()), 5);
        assert_eq!(unique_entity_count(std::iter::empty(), 0), 0);
    }

    #[test]
    fn optimization_batch_20260830cu_unique_entity_hash_count_source_contract() {
        let source = include_str!("page.rs");
        let unique_count = source
            .split("fn unique_extract_entity_count")
            .nth(1)
            .expect("unique entity count implementation")
            .split("pub(super) fn build_resident_page_inspections")
            .next()
            .expect("bounded unique entity count implementation");

        assert!(source.contains("use std::collections::{BTreeMap, HashSet};"));
        assert!(unique_count.contains("HashSet::with_capacity(capacity)"));
        assert!(!unique_count.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260830cu_runtime_unique_entity_hash_count_p95() {
        fn measure(entities: &[u64], count: impl Fn(&[u64]) -> usize) -> u128 {
            let started = std::time::Instant::now();
            for _ in 0..16 {
                std::hint::black_box(count(std::hint::black_box(entities)));
            }
            started.elapsed().as_nanos()
        }

        let entities = (0..65_536_u64)
            .map(|index| index.wrapping_mul(2_654_435_761) % 32_768)
            .collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(17);
        let mut optimized_samples = Vec::with_capacity(17);
        for sample_index in 0..17 {
            let legacy = |values: &[u64]| {
                values
                    .iter()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
            };
            let optimized =
                |values: &[u64]| unique_entity_count(values.iter().copied(), values.len()) as usize;
            if sample_index % 2 == 0 {
                legacy_samples.push(measure(&entities, legacy));
                optimized_samples.push(measure(&entities, optimized));
            } else {
                optimized_samples.push(measure(&entities, optimized));
                legacy_samples.push(measure(&entities, legacy));
            }
        }

        legacy_samples.sort_unstable();
        optimized_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let optimized_p95 = optimized_samples[16];
        println!(
            "RUNTIME397_UNIQUE_ENTITY_HASH_COUNT_BENCH_V1 entities={} legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=7000",
            entities.len(),
            legacy_p95,
            optimized_p95,
        );
        assert!(
            optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
            "hash unique-count P95 {optimized_p95} ns exceeded 70% of tree unique-count {legacy_p95} ns"
        );
    }
}
