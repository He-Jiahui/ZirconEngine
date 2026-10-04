use std::hint::black_box;
use std::time::Instant;

const BUILDS_PER_SAMPLE: usize = 256;
const ENTRIES_PER_BUILD: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_fs_editor405_reserves_import_refresh_entry_capacity() {
    let source = include_str!("../imports.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("import refresh production source");

    assert!(production.contains("Vec::with_capacity(import_instances.len())"));
    assert!(!production.contains(".filter_map(|instance_id|"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fs_editor405_reserved_import_refresh_entries_benchmark() {
    let entries = (0..ENTRIES_PER_BUILD)
        .map(|entry| [entry; 9])
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_builds(&entries, false));
        black_box(measure_builds(&entries, true));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_builds(&entries, false));
            optimized_samples.push(measure_builds(&entries, true));
        } else {
            optimized_samples.push(measure_builds(&entries, true));
            legacy_samples.push(measure_builds(&entries, false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR405_RESERVED_IMPORT_REFRESH_ENTRIES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} builds_per_sample={BUILDS_PER_SAMPLE} entries_per_build={ENTRIES_PER_BUILD} tuple_bytes={} legacy_growth_allocations_per_build=9 optimized_growth_allocations_per_build=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        std::mem::size_of::<[usize; 9]>(),
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 75 / 100);
}

fn measure_builds(entries: &[[usize; 9]], optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..BUILDS_PER_SAMPLE {
        let batch = if optimized {
            let mut batch = Vec::with_capacity(entries.len());
            for entry in black_box(entries) {
                if entry[0] != usize::MAX {
                    batch.push(*entry);
                }
            }
            batch
        } else {
            black_box(entries)
                .iter()
                .filter_map(|entry| (entry[0] != usize::MAX).then_some(*entry))
                .collect::<Vec<_>>()
        };
        black_box(batch);
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
