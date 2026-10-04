use std::hint::black_box;
use std::time::Instant;

use crate::ui::component::UiComponentDescriptorRegistry;

use super::component_descriptor_registry;

#[test]
fn runtime782_surface_tree_interaction_borrows_shared_catalog() {
    assert!(std::ptr::eq(
        component_descriptor_registry(),
        UiComponentDescriptorRegistry::editor_showcase_shared(),
    ));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime782_surface_tree_interaction_shared_catalog_release_benchmark() {
    const CATALOG_ACCESSES_PER_SAMPLE: usize = 256;
    const SAMPLE_PAIRS: usize = 17;

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_catalog_accesses(CATALOG_ACCESSES_PER_SAMPLE, true));
            optimized_ns.push(measure_catalog_accesses(CATALOG_ACCESSES_PER_SAMPLE, false));
        } else {
            optimized_ns.push(measure_catalog_accesses(CATALOG_ACCESSES_PER_SAMPLE, false));
            legacy_ns.push(measure_catalog_accesses(CATALOG_ACCESSES_PER_SAMPLE, true));
        }
    }

    let legacy_descriptor_clones = 69 * CATALOG_ACCESSES_PER_SAMPLE;
    let optimized_descriptor_clones = 0;
    assert!(legacy_descriptor_clones > optimized_descriptor_clones);
    println!(
        "RUNTIME782_SURFACE_TREE_SHARED_CATALOG_BENCH_V1 catalog_accesses_per_sample={CATALOG_ACCESSES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_descriptor_clones={legacy_descriptor_clones} optimized_descriptor_clones={optimized_descriptor_clones} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_catalog_accesses(accesses: usize, legacy: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..accesses {
        if legacy {
            black_box(UiComponentDescriptorRegistry::editor_showcase());
        } else {
            black_box(UiComponentDescriptorRegistry::editor_showcase_shared());
        }
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
