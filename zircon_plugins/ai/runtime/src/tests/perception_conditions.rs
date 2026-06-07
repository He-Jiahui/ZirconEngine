use zircon_runtime::core::framework::ai::{
    AiAgentTickRequest, AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor,
    AiBlackboardEntry, AiDecisionStatus, AiManager, AiPerceptionSense, AiPerceptionSnapshot,
    AiPerceptionStimulus,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Vec3;

use crate::DefaultAiManager;

#[test]
fn ai_manager_decorator_gates_tree_with_current_perception_stimulus() {
    let manager = DefaultAiManager::default();
    let tree_id = register_perception_tree(&manager);
    let world = WorldHandle::new(19);
    let entity = 14;

    let fallback_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: Vec::new(),
            perception: Some(perception_snapshot(
                entity,
                AiPerceptionSense::Hearing,
                200,
                1.0,
                0.05,
            )),
        })
        .expect("perception tick with non-matching sense");
    assert_eq!(fallback_report.status, AiDecisionStatus::Running);
    assert_eq!(fallback_report.active_node.as_deref(), Some("patrol"));

    let attack_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: Vec::new(),
            perception: Some(perception_snapshot(
                entity,
                AiPerceptionSense::Sight,
                200,
                0.75,
                0.15,
            )),
        })
        .expect("perception tick with matching sight stimulus");
    assert_eq!(attack_report.status, AiDecisionStatus::Succeeded);
    assert_eq!(attack_report.active_node.as_deref(), Some("attack"));
}

#[test]
fn ai_manager_decorator_can_use_stored_perception_snapshot_when_tick_omits_one() {
    let manager = DefaultAiManager::default();
    let tree_id = register_perception_tree(&manager);
    let world = WorldHandle::new(20);
    let entity = 15;
    manager
        .set_perception_snapshot(
            world,
            entity,
            perception_snapshot(entity, AiPerceptionSense::Sight, 200, 0.9, 0.2),
        )
        .expect("valid stored perception snapshot");

    let report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: Vec::new(),
            perception: None,
        })
        .expect("tick should use stored perception when no request snapshot is supplied");
    assert_eq!(report.status, AiDecisionStatus::Succeeded);
    assert_eq!(report.active_node.as_deref(), Some("attack"));
}

#[test]
fn ai_manager_decorator_applies_perception_strength_age_source_and_absence_filters() {
    let manager = DefaultAiManager::default();
    let tree_id = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("perception_filters", "Perception Filters", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Sequence, "Root")
                        .with_child("strong_recent_target")
                        .with_child("no_damage_alert")
                        .with_child("attack"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "strong_recent_target",
                        AiBehaviorNodeKind::Decorator,
                        "Strong Recent Target",
                    )
                    .with_parameter("perception_sense", "sight")
                    .with_parameter("perception_source", 200_u64)
                    .with_parameter("perception_min_strength", 0.5_f32)
                    .with_parameter("perception_max_age_seconds", 0.25_f32)
                    .with_child("target_visible"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "target_visible",
                        AiBehaviorNodeKind::Task,
                        "Target Visible",
                    )
                    .with_parameter("result", "succeeded"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "no_damage_alert",
                        AiBehaviorNodeKind::Decorator,
                        "No Damage Alert",
                    )
                    .with_parameter("perception_sense", "damage")
                    .with_parameter("perception_exists", false)
                    .with_child("damage_clear"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "damage_clear",
                        AiBehaviorNodeKind::Task,
                        "Damage Clear",
                    )
                    .with_parameter("result", "succeeded"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("attack", AiBehaviorNodeKind::Task, "Attack")
                        .with_parameter("result", "succeeded"),
                ),
        )
        .expect("valid perception filter tree");
    let world = WorldHandle::new(21);
    let entity = 16;

    let stale_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: Vec::new(),
            perception: Some(perception_snapshot(
                entity,
                AiPerceptionSense::Sight,
                200,
                0.9,
                0.6,
            )),
        })
        .expect("stale stimulus should fail the sequence");
    assert_eq!(stale_report.status, AiDecisionStatus::Failed);
    assert_eq!(
        stale_report.active_node.as_deref(),
        Some("strong_recent_target")
    );

    let damage_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: Vec::new(),
            perception: Some(AiPerceptionSnapshot {
                agent: entity,
                stimuli: vec![
                    stimulus(AiPerceptionSense::Sight, 200, 0.9, 0.1),
                    stimulus(AiPerceptionSense::Damage, 300, 1.0, 0.0),
                ],
            }),
        })
        .expect("damage stimulus should fail absence decorator");
    assert_eq!(damage_report.status, AiDecisionStatus::Failed);
    assert_eq!(
        damage_report.active_node.as_deref(),
        Some("no_damage_alert")
    );

    let clear_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: Vec::<AiBlackboardEntry>::new(),
            perception: Some(perception_snapshot(
                entity,
                AiPerceptionSense::Sight,
                200,
                0.9,
                0.1,
            )),
        })
        .expect("recent strong sight stimulus without damage should pass");
    assert_eq!(clear_report.status, AiDecisionStatus::Succeeded);
    assert_eq!(clear_report.active_node.as_deref(), Some("attack"));
}

fn register_perception_tree(
    manager: &DefaultAiManager,
) -> zircon_runtime::core::framework::ai::AiBehaviorTreeId {
    manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("perception_selector", "Perception Selector", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                        .with_child("can_attack")
                        .with_child("patrol"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "can_attack",
                        AiBehaviorNodeKind::Decorator,
                        "Can Attack",
                    )
                    .with_parameter("perception_sense", "sight")
                    .with_parameter("perception_source", 200_u64)
                    .with_parameter("perception_min_strength", 0.5_f32)
                    .with_parameter("perception_max_age_seconds", 0.25_f32)
                    .with_child("attack"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("attack", AiBehaviorNodeKind::Task, "Attack")
                        .with_parameter("result", "succeeded"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("patrol", AiBehaviorNodeKind::Task, "Patrol")
                        .with_parameter("result", "running"),
                ),
        )
        .expect("valid perception selector tree")
}

fn perception_snapshot(
    agent: u64,
    sense: AiPerceptionSense,
    source: u64,
    strength: f32,
    age_seconds: f32,
) -> AiPerceptionSnapshot {
    AiPerceptionSnapshot {
        agent,
        stimuli: vec![stimulus(sense, source, strength, age_seconds)],
    }
}

fn stimulus(
    sense: AiPerceptionSense,
    source: u64,
    strength: f32,
    age_seconds: f32,
) -> AiPerceptionStimulus {
    AiPerceptionStimulus {
        source,
        sense,
        position: Vec3::new(1.0, 2.0, 3.0),
        strength,
        age_seconds,
    }
}
