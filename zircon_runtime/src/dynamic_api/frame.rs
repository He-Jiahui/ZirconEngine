use std::ptr;

use zircon_runtime_interface::ui::accessibility::UiAccessibilityTreeSnapshot;
use zircon_runtime_interface::world_sync::{InvalidationBatch, WorldQueryResult};
use zircon_runtime_interface::{
    ProfileControlResponse, ProfileSnapshot, ZrOwnedResultV2, ZrRuntimeFrameV2,
    ZrRuntimeHostRequestBatchV1, ZrStatus, ZrStatusCode,
    ZR_RUNTIME_ACCESSIBILITY_TREE_OUTPUT_LIMIT_V1, ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1,
    ZR_RUNTIME_PROFILE_RESPONSE_OUTPUT_LIMIT_V1, ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1,
    ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1,
};

use crate::core::framework::render::CapturedFrame;

use super::bounded_json::{self, BoundedJsonError};

pub(super) struct EncodedRuntimeFrame {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) generation: u64,
    pub(super) rgba: Vec<u8>,
}

pub(super) fn encode_frame(frame: CapturedFrame) -> EncodedRuntimeFrame {
    EncodedRuntimeFrame {
        width: frame.width,
        height: frame.height,
        generation: frame.generation,
        rgba: frame.rgba,
    }
}

pub(super) fn encode_accessibility_tree(
    snapshot: &UiAccessibilityTreeSnapshot,
) -> Result<Vec<u8>, BoundedJsonError> {
    bounded_json::encode(
        snapshot,
        ZR_RUNTIME_ACCESSIBILITY_TREE_OUTPUT_LIMIT_V1,
        || accessibility_tree_item_count(snapshot),
    )
}

pub(super) fn encode_profile_response(
    response: &ProfileControlResponse,
) -> Result<Vec<u8>, BoundedJsonError> {
    bounded_json::encode(
        response,
        ZR_RUNTIME_PROFILE_RESPONSE_OUTPUT_LIMIT_V1,
        || profile_control_response_item_count(response),
    )
}

pub(super) fn encode_host_request_batch(
    batch: &ZrRuntimeHostRequestBatchV1,
) -> Result<Vec<u8>, BoundedJsonError> {
    bounded_json::encode(batch, ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1, || {
        batch.requests.len()
    })
}

pub(super) fn encode_world_query_payload(
    result: &WorldQueryResult,
) -> Result<Vec<u8>, BoundedJsonError> {
    bounded_json::encode(result, ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1, || {
        world_query_item_count(result)
    })
}

pub(super) fn encode_world_invalidations_payload(
    batches: &[InvalidationBatch],
) -> Result<Vec<u8>, BoundedJsonError> {
    bounded_json::encode(
        batches,
        ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1,
        || {
            batches.iter().fold(batches.len(), |count, batch| {
                count.saturating_add(world_invalidation_item_count(batch))
            })
        },
    )
}

fn accessibility_tree_item_count(snapshot: &UiAccessibilityTreeSnapshot) -> usize {
    snapshot.nodes.iter().fold(
        snapshot
            .roots
            .len()
            .saturating_add(snapshot.nodes.len())
            .saturating_add(snapshot.diagnostics.len()),
        |count, node| {
            count
                .saturating_add(node.actions.len())
                .saturating_add(node.children.len())
        },
    )
}

fn profile_control_response_item_count(response: &ProfileControlResponse) -> usize {
    let mut count = 1_usize.saturating_add(response.files.len());
    if let Some(snapshot) = &response.snapshot {
        count = count.saturating_add(profile_snapshot_item_count(snapshot));
    }
    if let Some(diagnostics) = &response.runtime_diagnostics {
        count = diagnostics.diagnostic_series.iter().fold(
            count.saturating_add(diagnostics.diagnostic_series.len()),
            |count, series| {
                count
                    .saturating_add(series.subsystem_tags.len())
                    .saturating_add(series.history.len())
            },
        );
        count = count.saturating_add(profile_snapshot_item_count(&diagnostics.profile));
    }
    if let Some(report) = &response.hotspot_report {
        count = count
            .saturating_add(report.hotspots.len())
            .saturating_add(report.hints.len());
    }
    if let Some(report) = &response.counter_hotspot_report {
        count = count
            .saturating_add(report.counters.len())
            .saturating_add(report.hints.len());
    }
    if let Some(report) = &response.ui_hotspot_report {
        count = count
            .saturating_add(report.scenarios.len())
            .saturating_add(report.alerts.len());
    }
    count
}

fn profile_snapshot_item_count(snapshot: &ProfileSnapshot) -> usize {
    snapshot
        .frames
        .len()
        .saturating_add(snapshot.spans.len())
        .saturating_add(snapshot.counters.len())
        .saturating_add(snapshot.recorder_retention.len())
}

fn world_query_item_count(result: &WorldQueryResult) -> usize {
    match result {
        WorldQueryResult::Rows(rows) => rows.iter().fold(rows.len(), |count, row| {
            row.components.values().fold(
                count.saturating_add(row.components.len()),
                |count, value| {
                    count.saturating_add(value.as_object().map_or(0, serde_json::Map::len))
                },
            )
        }),
        WorldQueryResult::NotModified { .. } => 1,
    }
}

fn world_invalidation_item_count(batch: &InvalidationBatch) -> usize {
    batch
        .dirty
        .len()
        .saturating_add(batch.facts.len())
        .saturating_add(1)
}

pub(super) fn write_frame(destination: *mut ZrRuntimeFrameV2, frame: ZrRuntimeFrameV2) -> ZrStatus {
    if destination.is_null() {
        return missing_output(b"missing frame output");
    }
    unsafe { ptr::write(destination, frame) };
    ZrStatus::ok()
}

pub(super) fn write_accessibility_tree(
    destination: *mut ZrOwnedResultV2,
    output: ZrOwnedResultV2,
) -> ZrStatus {
    write_output(destination, output, b"missing accessibility tree output")
}

pub(super) fn write_profile_response(
    destination: *mut ZrOwnedResultV2,
    output: ZrOwnedResultV2,
) -> ZrStatus {
    write_output(destination, output, b"missing profile output")
}

pub(super) fn write_host_requests(
    destination: *mut ZrOwnedResultV2,
    output: ZrOwnedResultV2,
) -> ZrStatus {
    write_output(destination, output, b"missing host request output")
}

pub(super) fn write_world_sync_payload(
    destination: *mut ZrOwnedResultV2,
    output: ZrOwnedResultV2,
) -> ZrStatus {
    write_output(destination, output, b"missing runtime world sync output")
}

fn write_output(
    destination: *mut ZrOwnedResultV2,
    output: ZrOwnedResultV2,
    missing_message: &'static [u8],
) -> ZrStatus {
    if destination.is_null() {
        return missing_output(missing_message);
    }
    unsafe { ptr::write(destination, output) };
    ZrStatus::ok()
}

fn missing_output(message: &'static [u8]) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::InvalidArgument,
        zircon_runtime_interface::ZrByteSlice::from_static(message),
    )
}
