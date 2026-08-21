use zircon_runtime::core::framework::ai::{
    AiHearingStimulusEvent, AiPerceptionSense, AiPerceptionStimulus,
};
use zircon_runtime::core::framework::physics::{
    PhysicsQueryFilter, PhysicsQueryInterface, PhysicsQueryMode, PhysicsRayCastQuery,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::math::{Real, Vec3};
use zircon_runtime::plugin::BridgeImport;
use zircon_runtime::scene::World;
use zircon_runtime::scene::ecs::Resource;

use super::adapter::HearingStimulusAdapter;
use super::components::{
    AiPerceptionChannels, AiPerceptionReceiver, AiPerceptionSource, perception_receiver,
    perception_source,
};
use super::stimuli::PerceivedStimuli;

pub const DEFAULT_AI_PERCEPTION_PAIR_BUDGET: usize = 256;
const OCCLUSION_DISTANCE_EPSILON: Real = 1.0e-4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AiTickBudget {
    pub max_pairs_per_frame: usize,
    consumed_pairs: usize,
    prefer_events: bool,
}

impl AiTickBudget {
    pub const fn new(max_pairs_per_frame: usize) -> Self {
        Self {
            max_pairs_per_frame,
            consumed_pairs: 0,
            prefer_events: false,
        }
    }

    pub const fn consumed_pairs(&self) -> usize {
        self.consumed_pairs
    }

    fn begin_frame(&mut self) {
        self.consumed_pairs = 0;
    }

    pub(super) fn try_consume(&mut self) -> bool {
        if self.consumed_pairs >= self.max_pairs_per_frame {
            return false;
        }
        self.consumed_pairs += 1;
        true
    }

    fn event_quota(&mut self, has_events: bool, has_static_pairs: bool) -> usize {
        if !has_events {
            return 0;
        }
        if !has_static_pairs {
            return self.max_pairs_per_frame;
        }
        let quota = self.max_pairs_per_frame / 2
            + usize::from(self.max_pairs_per_frame % 2 == 1 && self.prefer_events);
        self.prefer_events = !self.prefer_events;
        quota
    }
}

impl Default for AiTickBudget {
    fn default() -> Self {
        Self::new(DEFAULT_AI_PERCEPTION_PAIR_BUDGET)
    }
}

impl Resource for AiTickBudget {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PerceptionTickReport {
    pub scanned_pairs: usize,
    pub refreshed_stimuli: usize,
    pub forgotten_stimuli: usize,
    pub event_stimuli: usize,
    pub event_pairs: usize,
    pub physics_queries: usize,
    pub fallback_sight_pairs: usize,
}

pub(crate) trait SightOcclusionQuery {
    fn is_occluded(&self, query: &PhysicsRayCastQuery, source: EntityId) -> Option<bool>;
}

impl SightOcclusionQuery for BridgeImport<dyn PhysicsQueryInterface> {
    fn is_occluded(&self, query: &PhysicsRayCastQuery, source: EntityId) -> Option<bool> {
        self.call(|physics| {
            let closest = physics
                .ray_cast(query)
                .into_iter()
                .filter(|hit| hit.distance.is_finite() && hit.distance >= 0.0)
                .min_by(|left, right| left.distance.total_cmp(&right.distance));
            closest.is_some_and(|hit| {
                hit.entity != source
                    && hit.distance + OCCLUSION_DISTANCE_EPSILON < query.max_distance
            })
        })
        .ok()
    }
}

pub(crate) fn tick_perception(
    world: &World,
    world_handle: WorldHandle,
    delta_seconds: Real,
    budget: &mut AiTickBudget,
    perceived: &mut PerceivedStimuli,
    event_adapter: &mut HearingStimulusAdapter,
    hearing_events: &[AiHearingStimulusEvent],
    occlusion: Option<&dyn SightOcclusionQuery>,
) -> PerceptionTickReport {
    let (receivers, sources) = collect_perception_samples(world);
    let receiver_ages = receivers
        .iter()
        .map(|receiver| (receiver.entity, receiver.config.forget_seconds))
        .collect::<Vec<_>>();
    let mut report = PerceptionTickReport {
        forgotten_stimuli: perceived.begin_frame(delta_seconds, &receiver_ages),
        ..PerceptionTickReport::default()
    };
    budget.begin_frame();

    let hearing_receivers = receivers
        .iter()
        .map(|receiver| {
            (
                receiver.entity,
                receiver.position,
                receiver.config.hearing_radius,
                receiver.config.forget_seconds,
            )
        })
        .collect::<Vec<_>>();
    event_adapter.advance_time(delta_seconds);
    event_adapter.enqueue(hearing_events.iter().cloned(), &hearing_receivers);
    let has_static_pairs = receivers.iter().any(|receiver| {
        sources
            .iter()
            .any(|source| receiver.entity != source.entity)
    });
    let event_quota = budget.event_quota(event_adapter.pending_event_count() > 0, has_static_pairs);
    let event_report = event_adapter.process_budgeted(
        &hearing_receivers,
        event_quota,
        || budget.try_consume(),
        |receiver, stimulus| {
            perceived.refresh(receiver, stimulus);
        },
    );
    report.event_pairs += event_report.processed_pairs;
    report.scanned_pairs += event_report.processed_pairs;
    report.event_stimuli += event_report.refreshed_stimuli;
    report.refreshed_stimuli += event_report.refreshed_stimuli;

    scan_static_pairs(
        world_handle,
        &receivers,
        &sources,
        budget,
        perceived,
        occlusion,
        &mut report,
    );
    let remaining = budget
        .max_pairs_per_frame
        .saturating_sub(budget.consumed_pairs());
    if remaining > 0 {
        let event_report = event_adapter.process_budgeted(
            &hearing_receivers,
            remaining,
            || budget.try_consume(),
            |receiver, stimulus| perceived.refresh(receiver, stimulus),
        );
        report.event_pairs += event_report.processed_pairs;
        report.scanned_pairs += event_report.processed_pairs;
        report.event_stimuli += event_report.refreshed_stimuli;
        report.refreshed_stimuli += event_report.refreshed_stimuli;
    }
    report
}

fn scan_static_pairs(
    world_handle: WorldHandle,
    receivers: &[ReceiverSample],
    sources: &[SourceSample],
    budget: &mut AiTickBudget,
    perceived: &mut PerceivedStimuli,
    occlusion: Option<&dyn SightOcclusionQuery>,
    report: &mut PerceptionTickReport,
) {
    let pair_slot_count = receivers.len().saturating_mul(sources.len());
    if pair_slot_count == 0 {
        perceived.set_scan_cursor(0);
        return;
    }

    let mut cursor = perceived.scan_cursor(pair_slot_count);
    let mut visited_slots = 0;
    while visited_slots < pair_slot_count {
        let pair_slot = cursor;
        cursor = (cursor + 1) % pair_slot_count;
        visited_slots += 1;
        let receiver = &receivers[pair_slot / sources.len()];
        let source = &sources[pair_slot % sources.len()];
        if receiver.entity == source.entity {
            continue;
        }
        if !budget.try_consume() {
            cursor = pair_slot;
            break;
        }
        report.scanned_pairs += 1;
        scan_pair(world_handle, receiver, source, perceived, occlusion, report);
    }
    perceived.set_scan_cursor(cursor);
}

fn scan_pair(
    world_handle: WorldHandle,
    receiver: &ReceiverSample,
    source: &SourceSample,
    perceived: &mut PerceivedStimuli,
    occlusion: Option<&dyn SightOcclusionQuery>,
    report: &mut PerceptionTickReport,
) {
    if !source.config.strength.is_finite() || source.config.strength <= 0.0 {
        return;
    }
    let offset = source.position - receiver.position;
    let distance = offset.length();
    if source
        .config
        .channels
        .contains(AiPerceptionChannels::HEARING)
        && in_range(distance, receiver.config.hearing_radius)
    {
        perceived.refresh(
            receiver.entity,
            stimulus(source, AiPerceptionSense::Hearing),
        );
        report.refreshed_stimuli += 1;
    }
    if !source.config.channels.contains(AiPerceptionChannels::SIGHT)
        || !in_sight_cone(receiver, offset, distance)
    {
        return;
    }

    let query = PhysicsRayCastQuery {
        world: world_handle,
        origin: receiver.position.to_array(),
        direction: offset.normalize_or_zero().to_array(),
        max_distance: distance,
        mode: PhysicsQueryMode::Closest,
        filter: PhysicsQueryFilter {
            excluded_entities: vec![receiver.entity],
            ..PhysicsQueryFilter::default()
        },
    };
    let occluded = if distance <= OCCLUSION_DISTANCE_EPSILON {
        Some(false)
    } else {
        occlusion.and_then(|query_provider| query_provider.is_occluded(&query, source.entity))
    };
    match occluded {
        Some(true) => {
            report.physics_queries += 1;
        }
        Some(false) => {
            report.physics_queries += 1;
            perceived.refresh(receiver.entity, stimulus(source, AiPerceptionSense::Sight));
            report.refreshed_stimuli += 1;
        }
        None => {
            report.fallback_sight_pairs += 1;
            perceived.refresh(receiver.entity, stimulus(source, AiPerceptionSense::Sight));
            report.refreshed_stimuli += 1;
        }
    }
}

fn in_sight_cone(receiver: &ReceiverSample, offset: Vec3, distance: Real) -> bool {
    if !in_range(distance, receiver.config.sight_range)
        || !receiver.config.sight_fov_degrees.is_finite()
    {
        return false;
    }
    if distance <= OCCLUSION_DISTANCE_EPSILON {
        return true;
    }
    let half_fov = receiver
        .config
        .sight_fov_degrees
        .clamp(0.0, 360.0)
        .to_radians()
        * 0.5;
    receiver.forward.dot(offset / distance) >= half_fov.cos()
}

fn in_range(distance: Real, range: Real) -> bool {
    distance.is_finite() && range.is_finite() && range >= 0.0 && distance <= range
}

fn stimulus(source: &SourceSample, sense: AiPerceptionSense) -> AiPerceptionStimulus {
    AiPerceptionStimulus {
        source: source.entity,
        sense,
        position: source.position,
        strength: source.config.strength,
        age_seconds: 0.0,
    }
}

fn collect_perception_samples(world: &World) -> (Vec<ReceiverSample>, Vec<SourceSample>) {
    let nodes = world.node_records();
    let mut receivers = Vec::new();
    let mut sources = Vec::new();
    for node in nodes {
        let receiver = perception_receiver(world, node.id);
        let source = perception_source(world, node.id);
        if receiver.is_none() && source.is_none() {
            continue;
        }
        let Some(transform) = world.world_transform(node.id) else {
            continue;
        };
        if let Some(config) = receiver {
            receivers.push(ReceiverSample {
                entity: node.id,
                position: transform.translation,
                forward: transform.forward().normalize_or_zero(),
                config,
            });
        }
        if let Some(config) = source {
            sources.push(SourceSample {
                entity: node.id,
                position: transform.translation,
                config,
            });
        }
    }
    (receivers, sources)
}

#[derive(Clone, Copy, Debug)]
struct ReceiverSample {
    entity: EntityId,
    position: Vec3,
    forward: Vec3,
    config: AiPerceptionReceiver,
}

#[derive(Clone, Copy, Debug)]
struct SourceSample {
    entity: EntityId,
    position: Vec3,
    config: AiPerceptionSource,
}

#[cfg(test)]
mod sampling_tests {
    use std::{hint::black_box, time::Instant};

    use zircon_runtime::core::math::{Transform, Vec3};
    use zircon_runtime::scene::{NodeKind, World};

    use super::{
        AiPerceptionReceiver, AiPerceptionSource, ReceiverSample, SourceSample,
        collect_perception_samples, perception_receiver, perception_source,
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
        let source = include_str!("scan.rs");
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

    fn legacy_collect_perception_samples(
        world: &World,
    ) -> (Vec<ReceiverSample>, Vec<SourceSample>) {
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
}
