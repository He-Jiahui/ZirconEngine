use std::{hint::black_box, time::Instant};

use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::{NodeKind, World};

use super::{
    collect_perception_samples, perception_receiver, perception_source, AiPerceptionReceiver,
    AiPerceptionSource, ReceiverSample, SourceSample,
};

const BENCHMARK_NODE_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn single_pass_sampling_preserves_stable_receiver_and_source_order() {
    let world = benchmark_world(32);
    let (legacy_receivers, legacy_sources) = legacy_collect_perception_samples(&world);
    let (receivers, sources) = collect_perception_samples(&world);

    assert_eq!(
        sampled_entities(&legacy_receivers),
        sampled_entities(&receivers)
    );
    assert_eq!(
        sampled_entities(&legacy_sources),
        sampled_entities(&sources)
    );
}

#[test]
fn perception_tick_uses_one_world_projection_without_redundant_sample_sorts() {
    let source = include_str!("../scan.rs");
    let tick = source
        .split("pub(crate) fn tick_perception")
        .nth(1)
        .and_then(|body| body.split("fn scan_static_pairs").next())
        .expect("tick_perception source");
    let collector = source
        .split("fn collect_perception_samples")
        .nth(1)
        .and_then(|body| body.split("#[derive(Clone, Copy, Debug)]").next())
        .expect("sample collector source");

    assert!(tick.contains("collect_perception_samples(world)"));
    assert_eq!(collector.matches(".node_records()").count(), 1);
    assert!(!collector.contains("sort_by"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn single_pass_perception_sampling_release_benchmark_evidence() {
    let world = benchmark_world(BENCHMARK_NODE_COUNT);
    let (legacy_receivers, legacy_sources) = legacy_collect_perception_samples(&world);
    let (receivers, sources) = collect_perception_samples(&world);
    assert_eq!(
        sampled_entities(&legacy_receivers),
        sampled_entities(&receivers)
    );
    assert_eq!(
        sampled_entities(&legacy_sources),
        sampled_entities(&sources)
    );

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_collect_perception_samples(&world),
        || collect_perception_samples(&world),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_single_pass_perception_sampling nodes={} samples={} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_world_projections=2 optimized_world_projections=1 legacy_redundant_sample_sorts=2 optimized_redundant_sample_sorts=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
        BENCHMARK_NODE_COUNT,
        BENCHMARK_SAMPLE_COUNT,
        legacy_p50,
        legacy_p95,
        optimized_p50,
        optimized_p95
    );
    assert!(
        optimized_p95 * 4 <= legacy_p95 * 3,
        "optimized P95 {optimized_p95}ns must be no more than 75% of legacy P95 {legacy_p95}ns"
    );
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn benchmark_world(node_count: usize) -> World {
    let mut world = World::empty();
    for index in 0..node_count {
        let entity = world.spawn_node(NodeKind::Empty);
        world
            .update_transform(
                entity,
                Transform::from_translation(Vec3::new(index as f32, 0.0, 0.0)),
            )
            .unwrap();
        if index % 2 == 0 {
            world
                .insert(entity, AiPerceptionReceiver::default())
                .unwrap();
        } else {
            world.insert(entity, AiPerceptionSource::default()).unwrap();
        }
    }
    world
}

fn legacy_collect_perception_samples(world: &World) -> (Vec<ReceiverSample>, Vec<SourceSample>) {
    let mut receivers = world
        .node_records()
        .into_iter()
        .filter_map(|node| {
            let config = perception_receiver(world, node.id)?;
            let transform = world.world_transform(node.id)?;
            Some(ReceiverSample {
                entity: node.id,
                position: transform.translation,
                forward: transform.forward().normalize_or_zero(),
                config,
            })
        })
        .collect::<Vec<_>>();
    receivers.sort_by_key(|receiver| receiver.entity);
    let mut sources = world
        .node_records()
        .into_iter()
        .filter_map(|node| {
            let config = perception_source(world, node.id)?;
            let position = world.world_transform(node.id)?.translation;
            Some(SourceSample {
                entity: node.id,
                position,
                config,
            })
        })
        .collect::<Vec<_>>();
    sources.sort_by_key(|source| source.entity);
    (receivers, sources)
}

fn sampled_entities<T>(samples: &[T]) -> Vec<u64>
where
    T: SampledEntity,
{
    samples.iter().map(SampledEntity::entity).collect()
}

trait SampledEntity {
    fn entity(&self) -> u64;
}

impl SampledEntity for ReceiverSample {
    fn entity(&self) -> u64 {
        self.entity
    }
}

impl SampledEntity for SourceSample {
    fn entity(&self) -> u64 {
        self.entity
    }
}

fn benchmark_paired_samples<L, O>(
    mut legacy: impl FnMut() -> L,
    mut optimized: impl FnMut() -> O,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample<T>(operation: &mut impl FnMut() -> T) -> u128 {
    let started = Instant::now();
    let result = black_box(operation());
    let elapsed = started.elapsed().as_nanos();
    black_box(&result);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (sorted.len() * percentile).div_ceil(100) - 1;
    sorted[index]
}
