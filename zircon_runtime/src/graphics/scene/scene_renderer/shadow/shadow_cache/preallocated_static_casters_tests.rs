use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 10_000;
const STATIC_CASTER_COUNT: usize = 512;

#[test]
fn optimization_batch_20260829am_runtime313_preallocated_casters_preserve_order() {
    let legacy = legacy_static_casters(STATIC_CASTER_COUNT);
    let optimized = optimized_static_casters(STATIC_CASTER_COUNT);

    assert_eq!(optimized, legacy);
    assert_eq!(optimized.capacity(), STATIC_CASTER_COUNT);
    assert_eq!(optimized_static_casters(0).capacity(), 0);
}

#[test]
fn optimization_batch_20260829am_runtime313_shadow_path_preallocates_from_mesh_count() {
    let source = include_str!("../shadow_cache.rs");
    let builder = source
        .split("fn static_shadow_caster_revision_from_meshes_with_resource_revisions")
        .nth(1)
        .expect("resource-backed static shadow caster builder")
        .split("fn resource_revision_fingerprint")
        .next()
        .expect("resource-backed static shadow caster builder body");

    let reserve = builder
        .find("casters.reserve_exact(meshes.len());")
        .expect("caster upper-bound reservation");
    let push = builder.find("casters.push(").expect("caster append");
    assert!(builder.contains("if casters.is_empty()"));
    assert!(reserve < push);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829am_runtime313_preallocated_static_shadow_casters_bench() {
    assert_eq!(
        optimized_static_casters(STATIC_CASTER_COUNT),
        legacy_static_casters(STATIC_CASTER_COUNT)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME313_PREALLOCATED_STATIC_SHADOW_CASTERS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} static_casters_per_build={STATIC_CASTER_COUNT} \
legacy_vector_allocations_per_build=8 optimized_vector_allocations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_static_casters(count: usize) -> Vec<u64> {
    build_static_casters(Vec::new(), count)
}

fn optimized_static_casters(count: usize) -> Vec<u64> {
    let mut casters = Vec::new();
    if count > 0 {
        casters.reserve_exact(count);
    }
    build_static_casters(casters, count)
}

fn build_static_casters(mut casters: Vec<u64>, count: usize) -> Vec<u64> {
    for index in 0..count {
        casters.push(index as u64);
    }
    casters
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let casters = if optimized {
            optimized_static_casters(black_box(STATIC_CASTER_COUNT))
        } else {
            legacy_static_casters(black_box(STATIC_CASTER_COUNT))
        };
        checksum = checksum.wrapping_add(black_box(casters).len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
