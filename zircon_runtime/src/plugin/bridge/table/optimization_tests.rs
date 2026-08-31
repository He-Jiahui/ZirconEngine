use std::collections::HashMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{BridgeEntry, FrozenBridgeTable, FrozenBridgeTableInner, InterfaceExport};
use crate::core::framework::bridge::{BridgeOwnerTransitionMode, InterfaceSlot, PluginInterface};
use crate::plugin::extension_registry::PluginModuleId;

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 64;
const EXPORT_COUNT: usize = 2_048;

trait FixtureBridge: Send + Sync {
    fn value(&self) -> i32;
}

impl PluginInterface for dyn FixtureBridge {
    const INTERFACE_ID: &'static str = "test.runtime58.fixture.bridge.v1";
}

struct FixtureProvider;

impl FixtureBridge for FixtureProvider {
    fn value(&self) -> i32 {
        7
    }
}

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn fixture_export(index: usize) -> (PluginModuleId, String, InterfaceExport) {
    let provider: Arc<dyn FixtureBridge> = Arc::new(FixtureProvider);
    (
        PluginModuleId::from_raw(3),
        format!("runtime58.fixture.{index:04}"),
        InterfaceExport::new(provider),
    )
}

fn legacy_table_from_exports(
    exports: Vec<(PluginModuleId, String, InterfaceExport)>,
) -> FrozenBridgeTable {
    let mut entries = Vec::new();
    let mut slots_by_interface = HashMap::new();
    for (owner, interface_id, export) in exports {
        let slot = InterfaceSlot::from_raw(entries.len() as u32);
        slots_by_interface.insert(interface_id.clone(), slot);
        entries.push(BridgeEntry::new(interface_id, export.provider, owner));
    }
    FrozenBridgeTable {
        inner: Arc::new(FrozenBridgeTableInner {
            entries: entries.into_boxed_slice(),
            slots_by_interface,
        }),
    }
}

fn optimized_table_from_exports(
    exports: Vec<(PluginModuleId, String, InterfaceExport)>,
) -> FrozenBridgeTable {
    FrozenBridgeTable::from_exports(exports)
}

#[test]
fn runtime58_batch_export_table_preserves_slots_and_entries() {
    let table = optimized_table_from_exports((0..EXPORT_COUNT).map(fixture_export).collect());
    assert_eq!(table.entries().len(), EXPORT_COUNT);
    assert_eq!(
        table.resolve_slot("runtime58.fixture.0000"),
        Some(InterfaceSlot::from_raw(0))
    );
    assert_eq!(
        table.resolve_slot("runtime58.fixture.2047"),
        Some(InterfaceSlot::from_raw((EXPORT_COUNT - 1) as u32))
    );
}

#[test]
fn runtime58_batch_transition_snapshots_preserve_report_shape() {
    let table = optimized_table_from_exports((0..EXPORT_COUNT).map(fixture_export).collect());
    let report = table.set_owner_enabled_with_report(PluginModuleId::from_raw(3), false);
    assert_eq!(report.mode, BridgeOwnerTransitionMode::Disable);
    assert_eq!(report.affected_slots.len(), EXPORT_COUNT);
    assert_eq!(report.snapshots.len(), EXPORT_COUNT);
    assert!(report
        .snapshots
        .iter()
        .all(|snapshot| !snapshot.provider_installed));
}

#[test]
fn runtime58_batch_table_source_contract() {
    let source = include_str!("../table.rs");
    assert!(source.contains("Vec::with_capacity(export_count)"));
    assert!(source.contains("HashMap::with_capacity(export_count)"));
    assert!(source.contains("Vec::with_capacity(affected_slots.len())"));
    assert!(!source.contains("let mut entries = Vec::new();"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime58_batch_export_table_bench() {
    let mut legacy = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy.push(measure_table_construction(false));
            optimized.push(measure_table_construction(true));
        } else {
            optimized.push(measure_table_construction(true));
            legacy.push(measure_table_construction(false));
        }
    }
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME58_PREALLOCATED_EXPORT_TABLE_BENCH_V1 sample_order=alternating percentile_method=nearest_rank legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} exports={} entry_capacity_reserve=0->{} slot_capacity_reserve=0->{}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        EXPORT_COUNT,
        EXPORT_COUNT,
        EXPORT_COUNT,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime58_batch_transition_snapshots_bench() {
    let table = optimized_table_from_exports((0..EXPORT_COUNT).map(fixture_export).collect());
    let slots = (0..EXPORT_COUNT)
        .map(|index| InterfaceSlot::from_raw(index as u32))
        .collect::<Vec<_>>();
    let mut legacy = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy.push(measure_transition_snapshots(&table, &slots, false));
            optimized.push(measure_transition_snapshots(&table, &slots, true));
        } else {
            optimized.push(measure_transition_snapshots(&table, &slots, true));
            legacy.push(measure_transition_snapshots(&table, &slots, false));
        }
    }
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME58_PREALLOCATED_TRANSITION_SNAPSHOTS_BENCH_V1 sample_order=alternating percentile_method=nearest_rank legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} affected_slots={} snapshot_capacity_reserve=0->{}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        slots.len(),
        slots.len(),
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}

fn measure_table_construction(optimized: bool) -> Duration {
    let started = Instant::now();
    for _ in 0..ITERATIONS {
        let exports = (0..EXPORT_COUNT).map(fixture_export).collect::<Vec<_>>();
        if optimized {
            black_box(optimized_table_from_exports(exports));
        } else {
            black_box(legacy_table_from_exports(exports));
        }
    }
    started.elapsed()
}

fn measure_transition_snapshots(
    table: &FrozenBridgeTable,
    slots: &[InterfaceSlot],
    optimized: bool,
) -> Duration {
    let started = Instant::now();
    for _ in 0..ITERATIONS {
        let mut snapshots = if optimized {
            Vec::with_capacity(slots.len())
        } else {
            Vec::new()
        };
        for slot in slots {
            if let Some(entry) = table.entry(*slot) {
                snapshots.push(table.snapshot_for_entry(slot.index(), entry));
            }
        }
        black_box(snapshots);
    }
    started.elapsed()
}
