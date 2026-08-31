use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{BridgeInterfaceSnapshot, BridgeOwnerTransitionReport};
use crate::core::framework::bridge::{
    BridgeDiagnosticsSnapshot, BridgeInterfaceStatus, BridgeOwnerTransitionMode, InterfaceSlot,
};
use crate::plugin::extension_registry::PluginModuleId;

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 2_048;
const SNAPSHOT_COUNT: usize = 256;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn fixture_report() -> BridgeOwnerTransitionReport {
    let affected_slots = (0..SNAPSHOT_COUNT)
        .map(|index| InterfaceSlot::from_raw(index as u32))
        .collect::<Vec<_>>();
    let snapshots = affected_slots
        .iter()
        .enumerate()
        .map(|(index, slot)| BridgeInterfaceSnapshot {
            slot: *slot,
            interface_id: format!("runtime58.fixture.{index:04}"),
            owner: PluginModuleId::from_raw(3),
            generation: index as u32,
            provider_installed: true,
            status: BridgeInterfaceStatus::Enabled,
            diagnostics: BridgeDiagnosticsSnapshot {
                enabled_calls: index as u64,
                not_enabled_calls: 0,
            },
        })
        .collect();
    BridgeOwnerTransitionReport {
        owner: PluginModuleId::from_raw(3),
        mode: BridgeOwnerTransitionMode::Reload,
        affected_slots,
        snapshots,
    }
}

fn legacy_diagnostic(report: &BridgeOwnerTransitionReport) -> String {
    format!(
        "bridge.owner_transition: owner module slot {} {:?} affected {} interface(s): [{}]",
        report.owner.raw(),
        report.mode,
        report.affected_slots.len(),
        report
            .snapshots
            .iter()
            .map(|snapshot| format!(
                "`{}`@slot{} generation={} provider_installed={} status={:?}",
                snapshot.interface_id,
                snapshot.slot.raw(),
                snapshot.generation,
                snapshot.provider_installed,
                snapshot.status
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[test]
fn runtime58_batch_owner_transition_diagnostic_preserves_output() {
    let report = fixture_report();
    assert_eq!(report.diagnostic(), legacy_diagnostic(&report));
}

#[test]
fn runtime58_batch_owner_transition_diagnostic_source_contract() {
    let source = include_str!("../reports.rs");
    assert!(source.contains("String::with_capacity("));
    assert!(source.contains("diagnostic.push_str(\", \")"));
    assert!(!source.contains("collect::<Vec<_>>()\n                .join(\", \")"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime58_batch_owner_transition_diagnostic_bench() {
    let report = fixture_report();
    let mut legacy = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy.push(measure_diagnostic(&report, false));
            optimized.push(measure_diagnostic(&report, true));
        } else {
            optimized.push(measure_diagnostic(&report, true));
            legacy.push(measure_diagnostic(&report, false));
        }
    }
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME58_SINGLE_STRING_OWNER_TRANSITION_DIAGNOSTIC_BENCH_V1 sample_order=alternating percentile_method=nearest_rank legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} snapshots={} intermediate_string_vecs=1->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        SNAPSHOT_COUNT,
    );
    assert_eq!(report.diagnostic(), legacy_diagnostic(&report));
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}

fn measure_diagnostic(report: &BridgeOwnerTransitionReport, optimized: bool) -> Duration {
    let started = Instant::now();
    for _ in 0..ITERATIONS {
        if optimized {
            black_box(report.diagnostic());
        } else {
            black_box(legacy_diagnostic(report));
        }
    }
    started.elapsed()
}
