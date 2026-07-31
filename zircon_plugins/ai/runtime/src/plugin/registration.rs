use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiBehaviorDebugFrame, AiBehaviorDebugSnapshot, AiHearingStimulusEvent,
    AiPerceptionDebugSnapshot, BtNodeResultEvent,
};
use zircon_runtime::core::framework::animation::AnimationEventRecord;
use zircon_runtime::core::framework::physics::PhysicsQueryInterface;
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::framework::script::ScriptBehaviorBridge;
use zircon_runtime::core::framework::sound::SoundGameplayEmissionRead;
use zircon_runtime::core::manager::{resolve_manager_service, sound_manager_handle};
use zircon_runtime::plugin::{
    PluginEventCatalogManifest, PluginEventManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError,
};
use zircon_runtime::scene::World;
use zircon_runtime::scene::ecs::{EventCursor, EventReadIter, Resource};

use crate::behavior_tree::{
    BehaviorNodeRegistry, BehaviorNodeRegistryService, RuntimeBehaviorIntegrationHost,
};
use crate::perception::{
    AI_HEARING_INGEST_EVENT_LIMIT, AI_HEARING_PENDING_EVENT_CAPACITY, AiTickBudget,
    HearingStimulusAdapter, PerceivedStimuli, ai_perception_component_descriptors,
    hearing_event_from_animation, hearing_event_from_sound, perception_receiver, tick_perception,
};
use crate::{AI_MODULE_NAME, AiBehaviorTickLod, DefaultAiManager};

pub const AI_BEHAVIOR_TICK_SYSTEM: &str = "ai.behavior_tick";
pub const AI_PERCEPTION_TICK_SYSTEM: &str = "ai.perception_tick";
pub const AI_EVENT_NAMESPACE: &str = "ai.events";
pub const AI_BEHAVIOR_DEBUG_SNAPSHOT_EVENT_ID: &str = "ai.events.behavior_debug_snapshot";
pub const AI_BEHAVIOR_DEBUG_SNAPSHOT_PAYLOAD_SCHEMA: &str = "ai.events.behavior_debug_snapshot.v1";
pub const BT_NODE_RESULT_EVENT_ID: &str = "ai.events.bt_node_result";
pub const BT_NODE_RESULT_PAYLOAD_SCHEMA: &str = "ai.events.bt_node_result.v1";

pub(crate) struct PerceptionEventSubscriptions {
    hearing: EventCursor<AiHearingStimulusEvent>,
    animation: EventCursor<AnimationEventRecord>,
    pending_bus_events: VecDeque<AiHearingStimulusEvent>,
    dropped_bus_events: u64,
    sound_sequence: Option<u64>,
    activation_id: Option<u64>,
    last_frame_index: Option<u64>,
}

impl Default for PerceptionEventSubscriptions {
    fn default() -> Self {
        Self {
            hearing: EventCursor::default(),
            animation: EventCursor::default(),
            pending_bus_events: VecDeque::new(),
            dropped_bus_events: 0,
            sound_sequence: None,
            activation_id: None,
            last_frame_index: None,
        }
    }
}

impl Resource for PerceptionEventSubscriptions {}

impl PerceptionEventSubscriptions {
    pub(crate) fn begin_frame(
        &mut self,
        world: &mut World,
        activation_id: u64,
        frame_index: u64,
    ) -> bool {
        world.register_event::<AiHearingStimulusEvent>();
        world.register_event::<AnimationEventRecord>();
        let frame_gap = self
            .last_frame_index
            .is_some_and(|last| frame_index > last.saturating_add(1) || frame_index < last);
        let reset = self.activation_id != Some(activation_id) || frame_gap;
        if reset {
            self.hearing.clear(world.events::<AiHearingStimulusEvent>());
            self.animation.clear(world.events::<AnimationEventRecord>());
            self.pending_bus_events.clear();
            self.dropped_bus_events = 0;
            self.sound_sequence = None;
        }
        self.activation_id = Some(activation_id);
        self.last_frame_index = Some(frame_index);
        reset
    }

    pub(crate) fn sound_sequence(&self) -> Option<u64> {
        self.sound_sequence
    }

    pub(crate) fn advance_sound_sequence(&mut self, sequence: u64) {
        self.sound_sequence = Some(sequence);
    }

    pub(crate) fn advance_pending_bus_time(&mut self, delta_seconds: f32) {
        let delta_seconds = if delta_seconds.is_finite() {
            delta_seconds.max(0.0)
        } else {
            0.0
        };
        for event in &mut self.pending_bus_events {
            event.age_seconds += delta_seconds;
        }
    }

    pub(crate) fn collect_bus_events(&mut self, world: &World) {
        let hearing_count = self
            .hearing
            .unread_count(world.events::<AiHearingStimulusEvent>());
        let hearing_limit = hearing_count.min(AI_HEARING_INGEST_EVENT_LIMIT);
        self.record_dropped_bus_events(hearing_count.saturating_sub(hearing_limit));
        for event in self.read_hearing(world).take(hearing_limit) {
            self.push_pending_bus_event(event.clone());
        }
        let remaining_limit = AI_HEARING_INGEST_EVENT_LIMIT.saturating_sub(hearing_limit);
        let animation_count = self
            .animation
            .unread_count(world.events::<AnimationEventRecord>());
        let animation_limit = animation_count.min(remaining_limit);
        self.record_dropped_bus_events(animation_count.saturating_sub(animation_limit));
        for event in self.read_animation(world).take(animation_limit) {
            if let Some(event) = hearing_event_from_animation(world, event) {
                self.push_pending_bus_event(event);
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn pending_bus_event_count(&self) -> usize {
        self.pending_bus_events.len()
    }

    #[cfg(test)]
    pub(crate) fn dropped_bus_event_count(&self) -> u64 {
        self.dropped_bus_events
    }

    fn push_pending_bus_event(&mut self, event: AiHearingStimulusEvent) {
        if self.pending_bus_events.len() == AI_HEARING_PENDING_EVENT_CAPACITY {
            self.pending_bus_events.pop_front();
            self.record_dropped_bus_events(1);
        }
        self.pending_bus_events.push_back(event);
    }

    fn record_dropped_bus_events(&mut self, count: usize) {
        self.dropped_bus_events = self.dropped_bus_events.saturating_add(count as u64);
    }

    fn take_pending_bus_events(&mut self) -> Vec<AiHearingStimulusEvent> {
        self.pending_bus_events.drain(..).collect()
    }

    fn take_dropped_bus_events(&mut self) -> u64 {
        std::mem::take(&mut self.dropped_bus_events)
    }

    pub(crate) fn read_hearing<'events>(
        &mut self,
        world: &'events World,
    ) -> EventReadIter<'events, AiHearingStimulusEvent> {
        self.hearing.read(world.events::<AiHearingStimulusEvent>())
    }

    fn read_animation<'events>(
        &mut self,
        world: &'events World,
    ) -> EventReadIter<'events, AnimationEventRecord> {
        self.animation.read(world.events::<AnimationEventRecord>())
    }
}

fn next_perception_activation_id() -> u64 {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

fn read_sound_emissions(
    core: &zircon_runtime::core::CoreHandle,
    world: zircon_runtime::core::framework::scene::WorldHandle,
    after_sequence: Option<u64>,
) -> Result<Option<SoundGameplayEmissionRead>, zircon_runtime::core::CoreError> {
    let handle = match sound_manager_handle(core) {
        Ok(handle) => handle,
        Err(zircon_runtime::core::CoreError::MissingService(_)) => {
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    let sound = resolve_manager_service(core, handle)?;
    sound
        .read_gameplay_emissions(world, after_sequence)
        .map(Some)
        .map_err(|error| {
            zircon_runtime::core::CoreError::Initialization(
                AI_PERCEPTION_TICK_SYSTEM.to_string(),
                format!("sound gameplay emission read failed: {error}"),
            )
        })
}

pub(crate) fn collect_perception_hearing_events(
    subscriptions: &mut PerceptionEventSubscriptions,
    sound_read: Result<Option<SoundGameplayEmissionRead>, zircon_runtime::core::CoreError>,
    now_seconds: f64,
) -> Result<
    (
        Vec<AiHearingStimulusEvent>,
        Option<SoundGameplayEmissionRead>,
        u64,
    ),
    zircon_runtime::core::CoreError,
> {
    let sound_read = sound_read?;
    let mut hearing_events = subscriptions.take_pending_bus_events();
    let dropped_bus_events = subscriptions.take_dropped_bus_events();
    if let Some(sound_read) = &sound_read {
        hearing_events.extend(
            sound_read
                .events
                .iter()
                .filter_map(|event| hearing_event_from_sound(event, now_seconds)),
        );
    }
    Ok((hearing_events, sound_read, dropped_bus_events))
}

pub(super) fn ai_event_catalog() -> PluginEventCatalogManifest {
    PluginEventCatalogManifest {
        namespace: AI_EVENT_NAMESPACE.to_string(),
        version: 1,
        events: vec![
            ai_tick_report_event(),
            bt_node_result_event(),
            ai_behavior_debug_snapshot_event(),
            hearing_stimulus_event(),
        ],
    }
}

fn hearing_stimulus_event() -> PluginEventManifest {
    PluginEventManifest {
        id: "ai.events.hearing_stimulus".to_string(),
        display_name: "AI Hearing Stimulus".to_string(),
        payload_schema: "ai.events.hearing_stimulus.v1".to_string(),
    }
}

fn ai_tick_report_event() -> PluginEventManifest {
    PluginEventManifest {
        id: "ai.events.agent_tick_completed".to_string(),
        display_name: "AI Agent Tick Completed".to_string(),
        payload_schema: "ai.events.agent_tick_report.v1".to_string(),
    }
}

fn bt_node_result_event() -> PluginEventManifest {
    PluginEventManifest {
        id: BT_NODE_RESULT_EVENT_ID.to_string(),
        display_name: "Behavior Tree Node Result".to_string(),
        payload_schema: BT_NODE_RESULT_PAYLOAD_SCHEMA.to_string(),
    }
}

fn ai_behavior_debug_snapshot_event() -> PluginEventManifest {
    PluginEventManifest {
        id: AI_BEHAVIOR_DEBUG_SNAPSHOT_EVENT_ID.to_string(),
        display_name: "AI Behavior Debug Snapshot".to_string(),
        payload_schema: AI_BEHAVIOR_DEBUG_SNAPSHOT_PAYLOAD_SCHEMA.to_string(),
    }
}

pub(super) fn register_runtime_extensions(
    registry: &mut RuntimeExtensionRegistry,
    manager: Arc<DefaultAiManager>,
) -> Result<(), RuntimeExtensionRegistryError> {
    let mut module = zircon_plugin_sdk::RuntimePluginRegistrationBuilder::new(registry)
        .module(AI_MODULE_NAME)?;
    let owner = module.owner();
    for descriptor in ai_perception_component_descriptors() {
        module.component(descriptor)?;
    }
    module.resource(AiTickBudget::default)?;
    module.resource(HearingStimulusAdapter::default)?;
    module.resource(PerceptionEventSubscriptions::default)?;
    module.resource(PerceivedStimuli::default)?;
    manager
        .bind_standard_behavior_nodes_to_owner(owner)
        .map_err(|error| {
            RuntimeExtensionRegistryError::InvalidPluginModule(format!(
                "{AI_MODULE_NAME}: behavior node catalog: {error}"
            ))
        })?;
    let node_registry: Arc<dyn BehaviorNodeRegistry> =
        Arc::new(BehaviorNodeRegistryService::new(manager.clone()));
    module.export_interface::<dyn BehaviorNodeRegistry>(node_registry)?;
    let revocation_manager = manager.clone();
    module.owner_revocation_listener(move |revoked_owner| {
        revocation_manager.revoke_behavior_node_owner(revoked_owner);
    });
    module.event::<AiAgentTickReport>(ai_tick_report_event())?;
    module.event::<BtNodeResultEvent>(bt_node_result_event())?;
    module.event::<AiBehaviorDebugSnapshot>(ai_behavior_debug_snapshot_event())?;
    module.event::<AiHearingStimulusEvent>(hearing_stimulus_event())?;
    let physics_query = module.import_interface::<dyn PhysicsQueryInterface>()?;
    let perception_manager = manager.clone();
    let perception_activation_id = next_perception_activation_id();
    module
        .runtime_scene_system(
            AI_PERCEPTION_TICK_SYSTEM,
            zircon_runtime::scene::SystemStage::Update,
            move |context| {
                let world_handle = context.level.world_handle();
                let now_seconds = context.core.real_time().elapsed_secs_f64();
                let (sound_sequence, reset) = context.level.with_world_mut(|world| {
                    let mut subscriptions = world
                        .remove_resource::<PerceptionEventSubscriptions>()
                        .unwrap_or_default();
                    let reset = subscriptions.begin_frame(
                        world,
                        perception_activation_id,
                        context.core.real_time().frame_index(),
                    );
                    subscriptions.advance_pending_bus_time(context.delta_seconds);
                    subscriptions.collect_bus_events(world);
                    let sound_sequence = subscriptions.sound_sequence();
                    world.insert_resource(subscriptions);
                    (sound_sequence, reset)
                });
                let sound_read = read_sound_emissions(context.core, world_handle, sound_sequence);
                let snapshots = context.level.with_world_mut(|world| {
                    let mut subscriptions = world
                        .remove_resource::<PerceptionEventSubscriptions>()
                        .unwrap_or_default();
                    let inputs = collect_perception_hearing_events(
                        &mut subscriptions,
                        sound_read,
                        now_seconds,
                    );
                    let (hearing_events, sound_read, dropped_bus_events) = match inputs {
                        Ok(inputs) => inputs,
                        Err(error) => {
                            world.insert_resource(subscriptions);
                            return Err(error);
                        }
                    };
                    if let Some(sound_read) = &sound_read {
                        subscriptions.advance_sound_sequence(sound_read.next_sequence);
                    }
                    let mut budget = world.remove_resource::<AiTickBudget>().unwrap_or_default();
                    let mut event_adapter = world
                        .remove_resource::<HearingStimulusAdapter>()
                        .unwrap_or_default();
                    if reset {
                        event_adapter.clear_pending();
                    }
                    event_adapter.record_dropped_events(dropped_bus_events);
                    if let Some(sound_read) = &sound_read {
                        event_adapter.record_dropped_events(sound_read.missed_events);
                    }
                    let mut perceived = world
                        .remove_resource::<PerceivedStimuli>()
                        .unwrap_or_default();
                    tick_perception(
                        world,
                        world_handle,
                        context.delta_seconds,
                        &mut budget,
                        &mut perceived,
                        &mut event_adapter,
                        &hearing_events,
                        Some(&physics_query),
                    );
                    let snapshots = perceived.snapshots();
                    world.insert_resource(subscriptions);
                    world.insert_resource(budget);
                    world.insert_resource(event_adapter);
                    world.insert_resource(perceived);
                    Ok(snapshots)
                })?;
                perception_manager
                    .replace_world_perception_snapshots(world_handle, snapshots)
                    .map_err(|error| {
                        zircon_runtime::core::CoreError::Initialization(
                            AI_PERCEPTION_TICK_SYSTEM.to_string(),
                            error.to_string(),
                        )
                    })
            },
        )
        .before(zircon_runtime::scene::ecs::SystemRef::System(
            AI_BEHAVIOR_TICK_SYSTEM.to_string(),
        ))
        .register()?;
    let mut debug_reports_by_entity = BTreeMap::new();
    let script_bridge = module.import_interface::<dyn ScriptBehaviorBridge>()?;
    module
        .runtime_scene_system(
            AI_BEHAVIOR_TICK_SYSTEM,
            zircon_runtime::scene::SystemStage::Update,
            move |context| {
                let world_handle = context.level.world_handle();
                let active_entities_before_tick = manager.active_agent_entities(world_handle);
                context.level.with_world_mut(|world| {
                    let camera_position = world
                        .world_transform(world.active_camera())
                        .map(|transform| transform.translation);
                    let lod_by_entity = active_entities_before_tick
                        .iter()
                        .copied()
                        .map(|entity| {
                            let lod = camera_position
                                .zip(
                                    world
                                        .world_transform(entity)
                                        .map(|transform| transform.translation),
                                )
                                .map(|(camera, agent)| {
                                    AiBehaviorTickLod::from_distance((agent - camera).length())
                                })
                                .unwrap_or(AiBehaviorTickLod::Full);
                            (entity, lod)
                        })
                        .collect::<std::collections::BTreeMap<_, _>>();
                    let mut integration_host =
                        RuntimeBehaviorIntegrationHost::new(world, Some(script_bridge.clone()));
                    let reports = manager
                        .tick_active_agents_with_lod_and_integration_host(
                            world_handle,
                            context.delta_seconds,
                            context.core.real_time().frame_index(),
                            |entity| lod_by_entity.get(&entity).copied().unwrap_or_default(),
                            &mut integration_host,
                        )
                        .map_err(|error| {
                            zircon_runtime::core::CoreError::Initialization(
                                AI_BEHAVIOR_TICK_SYSTEM.to_string(),
                                error.to_string(),
                            )
                        })?;
                    drop(integration_host);
                    let active_entities = manager
                        .active_agent_entities(world_handle)
                        .into_iter()
                        .collect::<BTreeSet<_>>();
                    let snapshots_by_entity = manager
                        .runtime_snapshot()
                        .agents
                        .into_iter()
                        .filter(|snapshot| snapshot.world == world_handle)
                        .filter(|snapshot| active_entities.contains(&snapshot.entity))
                        .map(|snapshot| (snapshot.entity, snapshot))
                        .collect::<BTreeMap<_, _>>();
                    let world_id = world_handle.get();
                    debug_reports_by_entity.retain(|(report_world, entity), _| {
                        *report_world != world_id || active_entities.contains(entity)
                    });
                    for report in &reports {
                        if let Some(node_result) = report.node_result_event() {
                            world.send_event(node_result);
                        }
                        debug_reports_by_entity.insert((world_id, report.entity), report.clone());
                    }
                    let frames = debug_reports_by_entity
                        .iter()
                        .filter(|((report_world, entity), _)| {
                            *report_world == world_id && active_entities.contains(entity)
                        })
                        .filter_map(|((_, entity), report)| {
                            snapshots_by_entity
                                .get(entity)
                                .map(|snapshot| AiBehaviorDebugFrame {
                                    report: report.clone(),
                                    behavior_tree: snapshot.behavior_tree.clone(),
                                    blackboard: snapshot.blackboard.clone(),
                                    perception: snapshot.perception.clone(),
                                    perception_debug: perception_debug_snapshot(world, *entity),
                                })
                        })
                        .collect();
                    for report in reports {
                        world.send_event(report);
                    }
                    world.send_event(AiBehaviorDebugSnapshot {
                        world: world_handle,
                        frames,
                    });
                    Ok(())
                })?;
                Ok(())
            },
        )
        .register()
}

fn perception_debug_snapshot(world: &World, entity: EntityId) -> Option<AiPerceptionDebugSnapshot> {
    let receiver = perception_receiver(world, entity)?;
    let transform = world.world_transform(entity)?;
    Some(AiPerceptionDebugSnapshot {
        position: transform.translation,
        forward: transform.forward().normalize_or_zero(),
        sight_fov_degrees: receiver.sight_fov_degrees,
        sight_range: receiver.sight_range,
        hearing_radius: receiver.hearing_radius,
    })
}
