use std::collections::{BTreeMap, BTreeSet};

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
        return saturated_u32_len(
            extract
                .instances
                .iter()
                .map(|instance| instance.entity)
                .collect::<BTreeSet<_>>()
                .len(),
        );
    }

    saturated_u32_len(
        extract
            .clusters
            .iter()
            .map(|cluster| cluster.entity)
            .collect::<BTreeSet<_>>()
            .len(),
    )
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
    #[test]
    fn page_inspections_use_prebuilt_page_and_resident_indices() {
        let source = include_str!("page.rs");

        assert!(source.contains("build_page_size_index"));
        assert!(source.contains("resident_by_page_id"));
        assert!(!source.contains(concat!("extract", ".pages", ".iter()", ".find")));
        assert!(!source.contains(concat!("resident_page_inspections", ".iter()", ".find")));
    }
}
