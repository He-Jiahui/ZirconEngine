use zircon_runtime::core::framework::ai::{
    AiHearingStimulusEvent, AiPerceptionSense, AiPerceptionStimulus,
};
use zircon_runtime::core::framework::physics::{
    PhysicsQueryFilter, PhysicsQueryInterface, PhysicsQueryMode, PhysicsRayCastQuery,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::math::{Real, Vec3};
use zircon_runtime::plugin::BridgeImport;
use zircon_runtime::scene::ecs::Resource;
use zircon_runtime::scene::World;

use super::adapter::HearingStimulusAdapter;
use super::components::{
    perception_receiver, perception_source, AiPerceptionChannels, AiPerceptionReceiver,
    AiPerceptionSource,
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
    let receivers = collect_receivers(world);
    let sources = collect_sources(world);
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

fn collect_receivers(world: &World) -> Vec<ReceiverSample> {
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
    receivers
}

fn collect_sources(world: &World) -> Vec<SourceSample> {
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
    sources
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
