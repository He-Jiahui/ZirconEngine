use std::hint::black_box;
use std::time::Instant;

use super::{world_runtime_extension_registration_capacity, RuntimeExtensionRegistry};
use crate::core::framework::scene::SceneResource;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const REGISTRATIONS_PER_BUILD: usize = 256;

#[derive(Default)]
struct Runtime195FirstResource;

impl SceneResource for Runtime195FirstResource {}

#[derive(Default)]
struct Runtime195SecondResource;

impl SceneResource for Runtime195SecondResource {}

#[test]
fn optimization_batch_20260826ez_runtime195_capacity_preserves_world_extension_plan() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("runtime195.plugin")
        .expect("test owner");
    registry
        .register_resource(owner, Runtime195FirstResource::default)
        .expect("first resource");
    registry
        .register_resource(owner, Runtime195SecondResource::default)
        .expect("second resource");

    assert_eq!(world_runtime_extension_registration_capacity(&registry), 2);
    let plan = registry.world_runtime_extension_plan().expect("world plan");
    assert_eq!(plan.registration_count(), 2);
}

#[test]
fn optimization_batch_20260826ez_runtime195_plan_reserves_all_extension_families() {
    let source = include_str!("../apply_to_world.rs");
    assert!(source.contains("fn world_runtime_extension_registration_capacity("));
    assert!(source.contains("let mut registrations ="));
    assert!(
        source.contains("Vec::with_capacity(world_runtime_extension_registration_capacity(self))")
    );
    assert!(source.contains(".saturating_add(registry.plugin_runtime_systems().count())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ez_runtime195_world_extension_plan_capacity_bench() {
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
        "RUNTIME195_WORLD_EXTENSION_PLAN_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} registrations_per_build={REGISTRATIONS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut registrations = if reserve {
            Vec::with_capacity(REGISTRATIONS_PER_BUILD)
        } else {
            Vec::new()
        };
        for registration in 0..REGISTRATIONS_PER_BUILD {
            registrations.push(black_box(registration));
        }
        checksum ^= black_box(registrations.len() ^ registrations.capacity());
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
