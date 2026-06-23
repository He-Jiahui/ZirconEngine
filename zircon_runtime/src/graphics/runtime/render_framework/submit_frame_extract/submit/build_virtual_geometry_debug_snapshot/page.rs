use std::collections::BTreeSet;

use super::super::super::frame_submission_context::FrameSubmissionContext;
use super::support::saturated_u32_len;
use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryExtract, RenderVirtualGeometryPageRequestInspection,
    RenderVirtualGeometryResidentPageInspection,
};
use crate::graphics::VisibilityVirtualGeometryPageUploadPlan;

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
    extract: &RenderVirtualGeometryExtract,
    page_upload_plan: &VisibilityVirtualGeometryPageUploadPlan,
) -> Vec<RenderVirtualGeometryResidentPageInspection> {
    page_upload_plan
        .resident_pages
        .iter()
        .enumerate()
        .map(
            |(slot, page_id)| RenderVirtualGeometryResidentPageInspection {
                page_id: *page_id,
                slot: u32::try_from(slot).unwrap_or(u32::MAX),
                size_bytes: page_size_bytes(extract, *page_id),
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
    extract: &RenderVirtualGeometryExtract,
    context: &FrameSubmissionContext,
    page_upload_plan: &VisibilityVirtualGeometryPageUploadPlan,
    available_page_slots: &[u32],
) -> Vec<RenderVirtualGeometryPageRequestInspection> {
    page_upload_plan
        .requested_pages
        .iter()
        .enumerate()
        .map(
            |(frontier_rank, page_id)| RenderVirtualGeometryPageRequestInspection {
                page_id: *page_id,
                size_bytes: page_size_bytes(extract, *page_id),
                generation: context.predicted_generation(),
                frontier_rank: u32::try_from(frontier_rank).unwrap_or(u32::MAX),
                assigned_slot: available_page_slots.get(frontier_rank).copied(),
                recycled_page_id: None,
            },
        )
        .collect()
}

pub(super) fn build_evictable_page_inspections(
    extract: &RenderVirtualGeometryExtract,
    page_upload_plan: &VisibilityVirtualGeometryPageUploadPlan,
    resident_page_inspections: &[RenderVirtualGeometryResidentPageInspection],
) -> Vec<RenderVirtualGeometryResidentPageInspection> {
    page_upload_plan
        .evictable_pages
        .iter()
        .map(|page_id| {
            resident_page_inspections
                .iter()
                .find(|inspection| inspection.page_id == *page_id)
                .cloned()
                .unwrap_or(RenderVirtualGeometryResidentPageInspection {
                    page_id: *page_id,
                    slot: u32::MAX,
                    size_bytes: page_size_bytes(extract, *page_id),
                })
        })
        .collect()
}

fn page_size_bytes(extract: &RenderVirtualGeometryExtract, page_id: u32) -> u64 {
    extract
        .pages
        .iter()
        .find(|page| page.page_id == page_id)
        .map(|page| page.size_bytes)
        .unwrap_or(0)
}
