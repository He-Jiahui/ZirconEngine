use zircon_runtime::core::framework::ai::{
    AiAgentTickRequest, AiBehaviorEffectOutcome, AiBehaviorEffectReceipt, AiBehaviorNodeDescriptor,
    AiBehaviorNodeKind, AiBehaviorNodeParameterValue, AiBehaviorTreeDescriptor, AiBlackboardValue,
    AiDecisionStatus, AiGameplayEvent, AiManager,
};
use zircon_runtime::core::framework::scene::WorldHandle;

use crate::DefaultAiManager;

use super::{RuntimeBehaviorIntegrationHost, World};

const WORLD: WorldHandle = WorldHandle::new(77);
const AGENT: u64 = 9001;

#[test]
fn set_blackboard_overlay_and_emit_event_produce_world_effects_once_per_tick() {
    let manager = DefaultAiManager::default();
    let tree = manager
        .register_behavior_tree(effect_tree())
        .expect("typed effect tree");
    let mut world = World::new();
    world.register_event::<AiGameplayEvent>();
    world.register_event::<AiBehaviorEffectReceipt>();

    let first = tick_with_world_host(&manager, tree, &mut world);
    world.update_events::<AiGameplayEvent>();
    world.update_events::<AiBehaviorEffectReceipt>();

    assert_eq!(first.status, AiDecisionStatus::Succeeded);
    assert_eq!(
        manager.blackboard_entries(WORLD, AGENT),
        vec![zircon_runtime::core::framework::ai::AiBlackboardEntry::new(
            "score",
            AiBlackboardValue::Integer(42),
        )]
    );
    let first_event = world
        .events::<AiGameplayEvent>()
        .and_then(|events| events.iter().next().cloned())
        .expect("typed gameplay event is observable in World");
    assert_eq!(first_event.name, "ai.guard_passed");
    assert_eq!(
        first_event.payload,
        Some(AiBlackboardValue::String("confirmed".to_string()))
    );
    assert_eq!(first_event.effect_id.tick, 0);
    assert_eq!(first_event.effect_id.ordinal, 1);

    let first_receipts = world
        .events::<AiBehaviorEffectReceipt>()
        .expect("receipt channel registered")
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(first_receipts.len(), 2);
    assert_eq!(
        first_receipts[0].outcome,
        AiBehaviorEffectOutcome::BlackboardWrite { changed: true }
    );
    assert_eq!(
        first_receipts[1].outcome,
        AiBehaviorEffectOutcome::GameplayEventQueued
    );
    assert_eq!(first_receipts[0].effect_id.node_id, "set_score");
    assert_eq!(first_receipts[1].effect_id.node_id, "emit_guard_event");

    let second = tick_with_world_host(&manager, tree, &mut world);
    world.update_events::<AiGameplayEvent>();
    world.update_events::<AiBehaviorEffectReceipt>();

    assert_eq!(second.status, AiDecisionStatus::Succeeded);
    let second_event = world
        .events::<AiGameplayEvent>()
        .and_then(|events| events.iter().next().cloned())
        .expect("second tick emits its own event");
    assert_eq!(second_event.effect_id.tick, 1);
    assert_ne!(
        first_event.effect_id.effect_generation,
        second_event.effect_id.effect_generation
    );
    assert_ne!(first_event.effect_id, second_event.effect_id);

    let idle = tick_without_tree_with_world_host(&manager, &mut world);
    world.update_events::<AiGameplayEvent>();
    world.update_events::<AiBehaviorEffectReceipt>();
    assert_eq!(idle.status, AiDecisionStatus::Idle);
    assert!(world
        .events::<AiGameplayEvent>()
        .is_some_and(|events| events.is_empty()));
    assert!(world
        .events::<AiBehaviorEffectReceipt>()
        .is_some_and(|events| events.is_empty()));
}

#[test]
fn missing_gameplay_event_sink_blocks_the_whole_effect_batch() {
    let manager = DefaultAiManager::default();
    let tree = manager
        .register_behavior_tree(effect_tree())
        .expect("typed effect tree");
    let mut world = World::new();
    world.register_event::<AiBehaviorEffectReceipt>();

    let report = tick_with_world_host(&manager, tree, &mut world);

    assert_eq!(report.status, AiDecisionStatus::Blocked);
    assert!(report
        .diagnostic
        .as_deref()
        .is_some_and(|diagnostic| diagnostic.contains("gameplay event sink")));
    assert!(manager.blackboard_entries(WORLD, AGENT).is_empty());
    assert!(world.events::<AiGameplayEvent>().is_none());
    assert!(world
        .events::<AiBehaviorEffectReceipt>()
        .is_some_and(|events| events.is_empty()));
}

#[test]
fn unavailable_effect_sink_does_not_advance_a_latent_behavior_tree() {
    let manager = DefaultAiManager::default();
    let tree = manager
        .register_behavior_tree(effect_then_running_tree())
        .expect("typed effect tree");
    let mut world = World::new();
    world.register_event::<AiBehaviorEffectReceipt>();

    let blocked = tick_with_world_host(&manager, tree, &mut world);
    assert_eq!(blocked.status, AiDecisionStatus::Blocked);
    assert!(manager.blackboard_entries(WORLD, AGENT).is_empty());

    world.register_event::<AiGameplayEvent>();
    let running = tick_with_world_host(&manager, tree, &mut world);
    assert_eq!(running.status, AiDecisionStatus::Running);
    assert_eq!(
        manager.blackboard_entries(WORLD, AGENT),
        vec![zircon_runtime::core::framework::ai::AiBlackboardEntry::new(
            "score",
            AiBlackboardValue::Integer(42),
        )]
    );
    assert_eq!(
        world.events::<AiGameplayEvent>().unwrap().len(),
        1,
        "the deferred behavior tick reaches and commits its typed event"
    );

    world.update_events::<AiGameplayEvent>();
    world.update_events::<AiBehaviorEffectReceipt>();
    let resumed = tick_with_world_host(&manager, tree, &mut world);
    assert_eq!(resumed.status, AiDecisionStatus::Running);
    assert!(world.events::<AiGameplayEvent>().unwrap().is_empty());
    assert!(world
        .events::<AiBehaviorEffectReceipt>()
        .unwrap()
        .is_empty());
}

fn tick_with_world_host(
    manager: &DefaultAiManager,
    tree: zircon_runtime::core::framework::ai::AiBehaviorTreeId,
    world: &mut World,
) -> zircon_runtime::core::framework::ai::AiAgentTickReport {
    let mut host = RuntimeBehaviorIntegrationHost::new(world, None);
    manager
        .tick_agent_with_integration_host(
            AiAgentTickRequest {
                world: WORLD,
                entity: AGENT,
                behavior_tree: Some(tree),
                blackboard_schema: None,
                delta_seconds: 0.1,
                blackboard: Vec::new(),
                perception: None,
            },
            &mut host,
        )
        .expect("agent effect tick")
}

fn tick_without_tree_with_world_host(
    manager: &DefaultAiManager,
    world: &mut World,
) -> zircon_runtime::core::framework::ai::AiAgentTickReport {
    let mut host = RuntimeBehaviorIntegrationHost::new(world, None);
    manager
        .tick_agent_with_integration_host(
            AiAgentTickRequest {
                world: WORLD,
                entity: AGENT,
                behavior_tree: None,
                blackboard_schema: None,
                delta_seconds: 0.1,
                blackboard: Vec::new(),
                perception: None,
            },
            &mut host,
        )
        .expect("agent cancellation tick")
}

fn effect_tree() -> AiBehaviorTreeDescriptor {
    let root = AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Sequence, "Sequence")
        .with_child("set_score")
        .with_child("check_score")
        .with_child("emit_guard_event");
    let set_score =
        AiBehaviorNodeDescriptor::new("set_score", AiBehaviorNodeKind::Task, "Set Blackboard")
            .with_implementation("set_blackboard")
            .with_parameter("blackboard_key", "score")
            .with_parameter("value", AiBehaviorNodeParameterValue::Integer(42));
    let check_score = AiBehaviorNodeDescriptor::new(
        "check_score",
        AiBehaviorNodeKind::Decorator,
        "Blackboard Condition",
    )
    .with_implementation("blackboard_condition")
    .with_child("check_score_leaf")
    .with_parameter("blackboard_key", "score")
    .with_parameter("equals_integer", 42_i64);
    let check_score_leaf =
        AiBehaviorNodeDescriptor::new("check_score_leaf", AiBehaviorNodeKind::Task, "Wait")
            .with_implementation("wait")
            .with_parameter("result", "succeeded");
    let emit =
        AiBehaviorNodeDescriptor::new("emit_guard_event", AiBehaviorNodeKind::Task, "Emit Event")
            .with_implementation("emit_event")
            .with_parameter("event_name", "ai.guard_passed")
            .with_parameter(
                "payload",
                AiBehaviorNodeParameterValue::String("confirmed".to_string()),
            );
    AiBehaviorTreeDescriptor::new("typed_effects", "Typed Effects", "root")
        .with_node(root)
        .with_node(set_score)
        .with_node(check_score)
        .with_node(check_score_leaf)
        .with_node(emit)
}

fn effect_then_running_tree() -> AiBehaviorTreeDescriptor {
    let root = AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Sequence, "Sequence")
        .with_child("set_score")
        .with_child("emit_guard_event")
        .with_child("wait");
    let set_score =
        AiBehaviorNodeDescriptor::new("set_score", AiBehaviorNodeKind::Task, "Set Blackboard")
            .with_implementation("set_blackboard")
            .with_parameter("blackboard_key", "score")
            .with_parameter("value", AiBehaviorNodeParameterValue::Integer(42));
    let emit =
        AiBehaviorNodeDescriptor::new("emit_guard_event", AiBehaviorNodeKind::Task, "Emit Event")
            .with_implementation("emit_event")
            .with_parameter("event_name", "ai.guard_passed");
    let wait = AiBehaviorNodeDescriptor::new("wait", AiBehaviorNodeKind::Task, "Wait")
        .with_implementation("wait")
        .with_parameter("duration_seconds", 10.0_f32);
    AiBehaviorTreeDescriptor::new("typed_effects_wait", "Typed Effects Wait", "root")
        .with_node(root)
        .with_node(set_score)
        .with_node(emit)
        .with_node(wait)
}
