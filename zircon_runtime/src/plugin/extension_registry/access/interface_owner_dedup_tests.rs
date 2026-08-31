use std::hint::black_box;
use std::time::Instant;

use super::{sorted_unique_interface_owners, PluginModuleId};

const MARKER: &str = "RUNTIME242_INTERFACE_OWNER_HASH_DEDUP_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 2_048;

#[test]
fn optimization_batch_20260826gv_runtime242_interface_owners_stay_unique_and_sorted() {
    let owners = [7, 3, 7, 1, 3, 9, 1]
        .into_iter()
        .map(PluginModuleId::from_raw);
    assert_eq!(
        sorted_unique_interface_owners(owners)
            .into_iter()
            .map(PluginModuleId::raw)
            .collect::<Vec<_>>(),
        [1, 3, 7, 9]
    );
}

#[test]
fn optimization_batch_20260826gv_runtime242_interface_owners_dedup_before_sorting() {
    let source = include_str!("../access.rs");
    let implementation = source
        .split("fn sorted_unique_interface_owners")
        .nth(1)
        .and_then(|tail| tail.split("#[cfg(test)]").next())
        .expect("owner dedup implementation");
    assert!(implementation.contains("unique.insert(owner)"));
    assert!(implementation.contains("unique.into_iter().collect::<Vec<_>>()"));
    assert!(!implementation.contains("owners.dedup()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gv_runtime242_interface_owner_hash_dedup_bench() {
    let owners = (0..4_096)
        .map(|index| PluginModuleId::from_raw((index % 16) as u32))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&owners, legacy_sorted_unique_interface_owners));
            optimized_samples.push(measure(&owners, optimized_sorted_unique_interface_owners));
        } else {
            optimized_samples.push(measure(&owners, optimized_sorted_unique_interface_owners));
            legacy_samples.push(measure(&owners, legacy_sorted_unique_interface_owners));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "hash deduplication must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_sorted_unique_interface_owners(owners: &[PluginModuleId]) -> Vec<PluginModuleId> {
    let mut owners = owners.to_vec();
    owners.sort_by_key(|owner| owner.raw());
    owners.dedup();
    owners
}

fn optimized_sorted_unique_interface_owners(owners: &[PluginModuleId]) -> Vec<PluginModuleId> {
    sorted_unique_interface_owners(owners.iter().copied())
}

fn measure(
    owners: &[PluginModuleId],
    implementation: fn(&[PluginModuleId]) -> Vec<PluginModuleId>,
) -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..REPEATS {
        let unique = implementation(black_box(owners));
        checksum = checksum.wrapping_add(unique.len());
        black_box(&unique);
    }
    black_box(checksum);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
