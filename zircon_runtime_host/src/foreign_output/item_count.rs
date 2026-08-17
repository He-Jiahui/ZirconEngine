//! Structural item accounting for bounded runtime payloads.

use zircon_runtime_interface::world_sync::{InvalidationBatch, WorldQueryResult};
use zircon_runtime_interface::{
    ProfileControlResponse, ProfileSnapshot, ZrRuntimeOperationResultV1,
    ZrRuntimePluginEventDeliveryBatchV1,
};

pub fn json_value_item_count(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Array(values) => values.iter().fold(1_usize, |count, value| {
            count.saturating_add(json_value_item_count(value))
        }),
        serde_json::Value::Object(values) => values.values().fold(1_usize, |count, value| {
            count.saturating_add(json_value_item_count(value))
        }),
        _ => 1,
    }
}

pub fn operation_result_item_count(result: &ZrRuntimeOperationResultV1) -> usize {
    result
        .succeeded_output()
        .map(json_value_item_count)
        .unwrap_or(1)
        .saturating_add(1)
}

pub fn plugin_event_batch_item_count(batch: &ZrRuntimePluginEventDeliveryBatchV1) -> usize {
    batch.deliveries.len()
}

pub fn profile_control_response_item_count(response: &ProfileControlResponse) -> usize {
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

pub fn world_query_item_count(result: &WorldQueryResult) -> usize {
    match result {
        WorldQueryResult::Rows(rows) => rows.iter().fold(rows.len(), |count, row| {
            row.components.values().fold(count, |count, value| {
                count.saturating_add(json_value_item_count(value))
            })
        }),
        WorldQueryResult::NotModified { .. } => 1,
    }
}

pub fn world_invalidation_item_count(batch: &InvalidationBatch) -> usize {
    batch
        .dirty
        .len()
        .saturating_add(batch.facts.len())
        .saturating_add(1)
}
