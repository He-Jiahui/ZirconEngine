use std::hint::black_box;
use std::time::Instant;

use super::{ComponentId, normalize_components};

const MARKER: &str = "RUNTIME243_ARCHETYPE_SIGNATURE_HASH_DEDUP_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 2_048;

#[test]
fn optimization_batch_20260826gw_runtime243_signature_components_stay_unique_and_sorted() {
    let components = [7, 3, 7, 1, 3, 9, 1]
        .into_iter()
        .map(ComponentId::new)
        .collect::<Vec<_>>();

    assert_eq!(
        normalize_components(components)
            .into_iter()
            .map(ComponentId::index)
            .collect::<Vec<_>>(),
        [1, 3, 7, 9]
    );
}

#[test]
fn optimization_batch_20260826gw_runtime243_signature_dedups_large_inputs_before_sorting() {
    let source = include_str!("../signature.rs");
    let implementation = source
        .split("fn normalize_components")
        .nth(1)
        .and_then(|tail| tail.split("fn insert_component").next())
        .expect("component normalization implementation");
    assert!(implementation.contains("HASH_DEDUP_COMPONENT_THRESHOLD"));
    assert!(implementation.contains("HashSet::with_capacity"));
    assert!(implementation.contains("unique.insert(component)"));
    assert!(implementation.contains("unique.into_iter().collect::<Vec<_>>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gw_runtime243_archetype_signature_hash_dedup_bench() {
    let components = (0..4_096)
        .map(|index| ComponentId::new(index % 16))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&components, legacy_normalize_components));
            optimized_samples.push(measure(&components, optimized_normalize_components));
        } else {
            optimized_samples.push(measure(&components, optimized_normalize_components));
            legacy_samples.push(measure(&components, legacy_normalize_components));
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

fn legacy_normalize_components(components: &[ComponentId]) -> Vec<ComponentId> {
    let mut components = components.to_vec();
    components.sort_unstable();
    components.dedup();
    components
}

fn optimized_normalize_components(components: &[ComponentId]) -> Vec<ComponentId> {
    normalize_components(components.to_vec())
}

fn measure(
    components: &[ComponentId],
    implementation: fn(&[ComponentId]) -> Vec<ComponentId>,
) -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..REPEATS {
        let normalized = implementation(black_box(components));
        checksum = checksum.wrapping_add(normalized.len());
        black_box(&normalized);
    }
    black_box(checksum);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
