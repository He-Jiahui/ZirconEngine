use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::scene::reflect::{
    reflect_component_for_dynamic_descriptor, registration_from_component_descriptor,
    RuntimeTypeRegistration,
};
use crate::scene::World;

const BENCHMARK_SAMPLE_PAIRS: usize = 11;
const BENCHMARK_BASE_TYPES: usize = 512;
const BENCHMARK_BATCH_TYPES: usize = 64;
const MAX_OPTIMIZED_BATCH_NS: u128 = 3_000_000_000;

#[test]
fn component_type_registration_failure_is_atomic_and_retryable() {
    let mut world = World::empty();
    let component_types_before = world.component_types.clone();
    let type_registry_before = world.type_registry.clone();
    let component_registry_before = world.component_registry.clone();
    let component_schema_generation_before = world.component_types.schema_catalog_generation();
    let reflection_schema_generation_before = world.type_registry.schema_catalog_generation();
    let invalid = ComponentTypeDescriptor::new("atomic.Component.State", "atomic", "Atomic State")
        .with_property("value", "Scalar", true)
        .with_property("value", "Scalar", true);

    let error = world
        .register_component_type(invalid)
        .expect_err("duplicate reflected fields must reject registration");

    assert!(error
        .to_string()
        .contains("duplicate reflected field `value`"));
    assert_eq!(world.component_types, component_types_before);
    assert_eq!(world.type_registry, type_registry_before);
    assert_eq!(world.component_registry, component_registry_before);
    assert_eq!(
        world.component_types.schema_catalog_generation(),
        component_schema_generation_before
    );
    assert_eq!(
        world.type_registry.schema_catalog_generation(),
        reflection_schema_generation_before
    );
    assert!(world
        .component_registry
        .registered_dynamic_component_id("atomic.Component.State")
        .is_none());

    world
        .register_component_type(
            ComponentTypeDescriptor::new("atomic.Component.State", "atomic", "Atomic State")
                .with_property("value", "Scalar", true),
        )
        .expect("the same type id must be reusable after a rejected registration");

    assert!(world
        .component_type_descriptor("atomic.Component.State")
        .is_some());
    assert!(world.type_registry.contains("atomic.Component.State"));
    assert!(world
        .component_registry
        .registered_dynamic_component_id("atomic.Component.State")
        .is_some());
}

#[test]
#[ignore = "release performance gate; run through the Runtime63 managed validator"]
fn atomic_component_type_registration_benchmark() {
    let batch = (BENCHMARK_BASE_TYPES..BENCHMARK_BASE_TYPES + BENCHMARK_BATCH_TYPES)
        .map(|index| descriptor("atomic.Component.Benchmark", index))
        .collect::<Vec<_>>();
    let mut clone_baseline_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);

    for pair in 0..BENCHMARK_SAMPLE_PAIRS {
        let baseline_world = benchmark_world(BENCHMARK_BASE_TYPES);
        let optimized_world = benchmark_world(BENCHMARK_BASE_TYPES);
        let measure_baseline =
            || measure_registration_batch(baseline_world, &batch, clone_transactional_register);
        let measure_optimized = || {
            measure_registration_batch(optimized_world, &batch, |world, descriptor| {
                world.register_component_type(descriptor)
            })
        };
        if pair % 2 == 0 {
            clone_baseline_samples.push(measure_baseline());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            clone_baseline_samples.push(measure_baseline());
        }
    }

    let baseline_p50 = nearest_rank(&clone_baseline_samples, 50);
    let baseline_p95 = nearest_rank(&clone_baseline_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let count_world = benchmark_world(BENCHMARK_BASE_TYPES);
    let initial_existing_catalog_entries = count_world.component_types.descriptors().count()
        + count_world.type_registry.iter().count();
    let baseline_entry_copies = (0..BENCHMARK_BATCH_TYPES)
        .map(|offset| initial_existing_catalog_entries + offset * 2)
        .sum::<usize>();
    println!(
        "RUNTIME63_ATOMIC_COMPONENT_REGISTRATION_BENCH_V1 sample_pairs={} base_types={} batch_types={} initial_existing_catalog_entries={} clone_baseline_existing_catalog_entry_copies={} optimized_existing_catalog_entry_copies=0 entry_copy_reduction_percent=100.0000 clone_baseline_samples_ns={} optimized_samples_ns={} clone_baseline_p50_ns={} clone_baseline_p95_ns={} optimized_p50_ns={} optimized_p95_ns={}",
        BENCHMARK_SAMPLE_PAIRS,
        BENCHMARK_BASE_TYPES,
        BENCHMARK_BATCH_TYPES,
        initial_existing_catalog_entries,
        baseline_entry_copies,
        sample_csv(&clone_baseline_samples),
        sample_csv(&optimized_samples),
        baseline_p50,
        baseline_p95,
        optimized_p50,
        optimized_p95,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(60),
        "optimized P95 {optimized_p95}ns must be at most 60% of clone-transaction P95 {baseline_p95}ns"
    );
    assert!(
        optimized_p95 <= MAX_OPTIMIZED_BATCH_NS,
        "optimized P95 {optimized_p95}ns exceeds the 3 second batch budget"
    );
}

fn descriptor(prefix: &str, index: usize) -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(format!("{prefix}{index}"), "atomic", "Atomic State")
        .with_property("value", "Scalar", true)
}

fn benchmark_world(type_count: usize) -> World {
    let mut world = World::empty();
    for index in 0..type_count {
        world
            .register_component_type(descriptor("atomic.Component.Benchmark", index))
            .expect("benchmark catalog type must register");
    }
    world
}

fn clone_transactional_register(
    world: &mut World,
    descriptor: ComponentTypeDescriptor,
) -> crate::scene::SceneResult<()> {
    let registration = registration_from_component_descriptor(&descriptor)?;
    let component = reflect_component_for_dynamic_descriptor(&descriptor);
    let runtime_registration = RuntimeTypeRegistration {
        registration,
        component: Some(component),
        resource: None,
    };
    let component_type_id = descriptor.type_id.clone();
    let mut component_types = world.component_types.clone();
    component_types.register(descriptor)?;
    let mut type_registry = world.type_registry.clone();
    type_registry.register(runtime_registration)?;
    world.component_types = component_types;
    world.type_registry = type_registry;
    world
        .component_registry
        .dynamic_component_id(&component_type_id);
    Ok(())
}

fn measure_registration_batch(
    mut world: World,
    descriptors: &[ComponentTypeDescriptor],
    mut register: impl FnMut(&mut World, ComponentTypeDescriptor) -> crate::scene::SceneResult<()>,
) -> u128 {
    let started = Instant::now();
    for descriptor in descriptors {
        register(black_box(&mut world), black_box(descriptor.clone()))
            .expect("benchmark registration must succeed");
    }
    black_box(world.component_types.schema_catalog_generation());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
