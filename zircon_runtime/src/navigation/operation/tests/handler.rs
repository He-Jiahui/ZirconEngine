use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::navigation::{
    NavMeshAsset, NavigationGeneratedBakeChange, NavigationGeneratedBakeSnapshot,
};

use super::{
    encode_prepared_snapshot_values, NavigationOperationHandler, NavigationOperationKind,
    NavigationSnapshotChange,
};

#[test]
fn optimization_batch_dx_navigation_snapshot_values_preserve_serialized_contracts() {
    let change = navigation_change_fixture(8);
    let expected_command = serde_json::to_value(&change).expect("legacy command value");
    let expected_result = serde_json::to_value(NavigationGeneratedBakeChange {
        before: change.before.clone(),
        after: change.after.clone(),
        report: None,
    })
    .expect("legacy result value");

    let (command, result) =
        encode_prepared_snapshot_values(change).expect("prepared snapshot values");

    assert_eq!(command, expected_command);
    assert_eq!(result, expected_result);
}

#[test]
fn optimization_batch_dx_navigation_snapshot_values_serialize_once() {
    let production = include_str!("../handler.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("navigation operation production source");
    let encoder = production
        .split("fn encode_prepared_snapshot_values")
        .nth(1)
        .expect("single serialization encoder");

    assert!(encoder.contains("let command = encode_snapshot_change(change)?"));
    assert!(encoder.contains("let mut result = command.clone()"));
    assert!(!encoder.contains("change.before.clone()"));
    assert!(!encoder.contains("change.after.clone()"));
}

#[test]
fn navigation_operation_rejects_malformed_surface_bake_before_owner_capture() {
    let handler = NavigationOperationHandler::new(NavigationOperationKind::BakeSurface);
    assert!(handler.operation_request(serde_json::json!({})).is_err());
}

#[test]
fn navigation_operation_source_retains_guarded_owner_closure() {
    let production = include_str!("../handler.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("navigation operation production source");
    assert!(production.contains("context.level().capture()"));
    assert!(production.contains("source.publish"));
    assert!(production.contains("apply_bake_operation"));
    assert!(production.contains("generated bake state changed before owner apply"));
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_dx_single_navigation_snapshot_serialization_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const PREPARATIONS_PER_SAMPLE: usize = 16;
    const VERTEX_COUNT: usize = 8_192;

    let prototype = navigation_change_fixture(VERTEX_COUNT);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_snapshot_preparations(
                &prototype,
                PREPARATIONS_PER_SAMPLE,
                false,
            ));
            optimized_samples.push(measure_snapshot_preparations(
                &prototype,
                PREPARATIONS_PER_SAMPLE,
                true,
            ));
        } else {
            optimized_samples.push(measure_snapshot_preparations(
                &prototype,
                PREPARATIONS_PER_SAMPLE,
                true,
            ));
            legacy_samples.push(measure_snapshot_preparations(
                &prototype,
                PREPARATIONS_PER_SAMPLE,
                false,
            ));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "RUNTIME432_SINGLE_NAVIGATION_SNAPSHOT_SERIALIZATION_BENCH_V1 preparations_per_sample={PREPARATIONS_PER_SAMPLE} vertex_count={VERTEX_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "single navigation snapshot serialization p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

fn navigation_change_fixture(vertex_count: usize) -> NavigationSnapshotChange {
    let mut before_asset = NavMeshAsset::default();
    before_asset.agent_type = "navigation-agent-before".repeat(16);
    before_asset.vertices = (0..vertex_count)
        .map(|index| [index as f32, (index % 17) as f32, (index % 31) as f32])
        .collect();
    before_asset.indices = (0..vertex_count as u32).collect();
    let mut after_asset = before_asset.clone();
    after_asset.agent_type = "navigation-agent-after".repeat(16);

    NavigationSnapshotChange {
        before: NavigationGeneratedBakeSnapshot {
            surface_entity: Some(17),
            asset: Some(before_asset),
            output_asset: Some("res://navigation/generated/before.navmesh".repeat(8)),
        },
        after: NavigationGeneratedBakeSnapshot {
            surface_entity: Some(17),
            asset: Some(after_asset),
            output_asset: Some("res://navigation/generated/after.navmesh".repeat(8)),
        },
    }
}

fn measure_snapshot_preparations(
    prototype: &NavigationSnapshotChange,
    preparation_count: usize,
    optimized: bool,
) -> u128 {
    let inputs = vec![prototype.clone(); preparation_count];
    let started_at = Instant::now();
    let mut checksum = 0_usize;
    for change in inputs {
        let values = if optimized {
            encode_prepared_snapshot_values(change).expect("optimized snapshot values")
        } else {
            legacy_prepared_snapshot_values(change)
        };
        checksum = checksum
            .wrapping_add(values.0.as_object().map_or(0, |value| value.len()))
            .wrapping_add(values.1.as_object().map_or(0, |value| value.len()));
        black_box(values);
    }
    black_box(checksum);
    started_at.elapsed().as_nanos()
}

fn legacy_prepared_snapshot_values(
    change: NavigationSnapshotChange,
) -> (serde_json::Value, serde_json::Value) {
    let result = serde_json::to_value(NavigationGeneratedBakeChange {
        before: change.before.clone(),
        after: change.after.clone(),
        report: None,
    })
    .expect("legacy result value");
    let command = serde_json::to_value(change).expect("legacy command value");
    (command, result)
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}
