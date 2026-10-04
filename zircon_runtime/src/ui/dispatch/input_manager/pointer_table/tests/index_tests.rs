use std::hint::black_box;
use std::time::Instant;
use zircon_runtime_interface::ui::dispatch::{UiPointerId, UiPointerSource};

use super::UiActivePointerTable;

#[test]
fn indexed_pointer_lookup_preserves_entry_order_and_state_after_middle_remove() {
    let first = UiPointerId::new(11);
    let middle = UiPointerId::new(22);
    let last = UiPointerId::new(33);
    let mut table = UiActivePointerTable::default();

    table.upsert(first, UiPointerSource::Mouse, true);
    table.upsert(middle, UiPointerSource::Touch, false);
    table.upsert(last, UiPointerSource::Pen, false);
    table.record_point(
        last,
        zircon_runtime_interface::ui::layout::UiPoint::new(4.0, 8.0),
    );

    let removed = table.remove(middle).expect("middle pointer is active");
    assert_eq!(removed.pointer_id, middle);
    assert_eq!(
        table
            .entries()
            .iter()
            .map(|entry| entry.pointer_id)
            .collect::<Vec<_>>(),
        vec![first, last]
    );
    assert_eq!(
        table.entry(first).expect("first pointer").source,
        UiPointerSource::Mouse
    );
    assert_eq!(
        table.entry(last).expect("last pointer").last_point,
        Some(zircon_runtime_interface::ui::layout::UiPoint::new(4.0, 8.0))
    );
    assert!(table.entry(middle).is_none());
}

#[test]
fn indexed_pointer_lookup_clear_drops_membership_without_losing_capacity() {
    let pointer = UiPointerId::new(7);
    let mut table = UiActivePointerTable::default();
    table.upsert(pointer, UiPointerSource::Mouse, true);

    table.clear();

    assert!(table.entries().is_empty());
    assert!(table.entry(pointer).is_none());
    table.upsert(pointer, UiPointerSource::Mouse, true);
    assert!(table.entry(pointer).is_some());
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime200_primary_pointer_source_counter_bench() {
    const SAMPLE_COUNT: usize = 17;
    const PROBES_PER_SAMPLE: usize = 4_096;
    const POINTER_COUNT: usize = 4_096;

    let mut table = UiActivePointerTable::default();
    for index in 0..POINTER_COUNT {
        table.upsert(
            UiPointerId::new(index as u64 + 1),
            UiPointerSource::Touch,
            index + 1 == POINTER_COUNT,
        );
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure_primary_scan(&table, PROBES_PER_SAMPLE));
            optimized_samples.push(measure_primary_counter(&table, PROBES_PER_SAMPLE));
        } else {
            optimized_samples.push(measure_primary_counter(&table, PROBES_PER_SAMPLE));
            legacy_samples.push(measure_primary_scan(&table, PROBES_PER_SAMPLE));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME200_PRIMARY_POINTER_SOURCE_COUNTER_BENCH_V1 samples={SAMPLE_COUNT} probes_per_sample={PROBES_PER_SAMPLE} pointers={POINTER_COUNT} legacy_probes_per_sample={PROBES_PER_SAMPLE} optimized_probes_per_sample={PROBES_PER_SAMPLE} legacy_membership_steps_per_probe={POINTER_COUNT} optimized_membership_steps_per_probe=1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_ns={} optimized_ns={}",
        legacy_samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(","),
        optimized_samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(","),
    );
    assert!(
        optimized_p95.saturating_mul(2) <= legacy_p95,
        "primary-source counter P95 {optimized_p95}ns must be at most 50% of scan P95 {legacy_p95}ns"
    );
}

fn measure_primary_scan(table: &UiActivePointerTable, probes: usize) -> u128 {
    let started = Instant::now();
    let mut checksum = false;
    for _ in 0..probes {
        checksum ^= black_box(
            black_box(table.entries())
                .iter()
                .any(|entry| entry.source == UiPointerSource::Touch && entry.is_primary),
        );
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_primary_counter(table: &UiActivePointerTable, probes: usize) -> u128 {
    let started = Instant::now();
    let mut checksum = false;
    for _ in 0..probes {
        checksum ^= black_box(black_box(table).has_primary_for_source(UiPointerSource::Touch));
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
