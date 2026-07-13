use zircon_runtime::core::framework::ai::{
    AiAgentTickRequest, AiBehaviorAbortPolicy, AiBehaviorNodeDescriptor, AiBehaviorNodeKind,
    AiBehaviorTreeDescriptor, AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardValue,
    AiDecisionStatus, AiManager,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Vec3;

use crate::DefaultAiManager;

#[test]
fn numeric_blackboard_conditions_preempt_with_explicit_policy() {
    let manager = DefaultAiManager::default();
    let schema_id = manager
        .register_blackboard_schema(
            AiBlackboardSchemaDescriptor::new("combat_thresholds", "Combat Thresholds")
                .with_key("ammo", "integer", true)
                .with_key("health", "scalar", true),
        )
        .expect("combat threshold schema");
    let tree_id = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("combat_thresholds", "Combat Thresholds", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                        .with_child("has_resources")
                        .with_child("recover"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "has_resources",
                        AiBehaviorNodeKind::Decorator,
                        "Has Resources",
                    )
                    .with_parameter("blackboard_key", "ammo")
                    .with_parameter("greater_or_equal_integer", 3_i64)
                    .with_abort_policy(AiBehaviorAbortPolicy::LowerPriority)
                    .with_child("healthy_enough"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "healthy_enough",
                        AiBehaviorNodeKind::Decorator,
                        "Healthy Enough",
                    )
                    .with_parameter("blackboard_key", "health")
                    .with_parameter("greater_than_scalar", 0.35_f32)
                    .with_parameter("less_or_equal_scalar", 1.0_f32)
                    .with_abort_policy(AiBehaviorAbortPolicy::LowerPriority)
                    .with_child("attack"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("attack", AiBehaviorNodeKind::Task, "Attack")
                        .with_parameter("result", "succeeded"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("recover", AiBehaviorNodeKind::Task, "Recover")
                        .with_parameter("result", "running"),
                ),
        )
        .expect("numeric condition tree");
    let tick = |ammo, health| {
        manager
            .tick_agent(AiAgentTickRequest {
                world: WorldHandle::new(14),
                entity: 9,
                behavior_tree: Some(tree_id),
                blackboard_schema: Some(schema_id),
                delta_seconds: 1.0 / 60.0,
                blackboard: vec![
                    AiBlackboardEntry::new("ammo", AiBlackboardValue::Integer(ammo)),
                    AiBlackboardEntry::new("health", AiBlackboardValue::Scalar(health)),
                ],
                perception: None,
            })
            .expect("numeric condition tick")
    };
    assert_eq!(tick(2, 0.8).status, AiDecisionStatus::Running);
    assert_eq!(tick(3, 0.35).status, AiDecisionStatus::Running);
    assert_eq!(tick(5, 0.75).status, AiDecisionStatus::Succeeded);
}

#[test]
fn vec3_blackboard_condition_preempts_with_explicit_policy() {
    let manager = DefaultAiManager::default();
    let target_point = Vec3::new(4.0, 0.0, -2.0);
    let schema_id = manager
        .register_blackboard_schema(
            AiBlackboardSchemaDescriptor::new("target_point", "Target Point").with_key(
                "target_point",
                "vec3",
                true,
            ),
        )
        .expect("target point schema");
    let tree_id = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("target_point_selector", "Target Point Selector", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                        .with_child("at_target_point")
                        .with_child("reposition"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "at_target_point",
                        AiBehaviorNodeKind::Decorator,
                        "At Target Point",
                    )
                    .with_parameter("blackboard_key", "target_point")
                    .with_parameter("equals_vec3", target_point)
                    .with_abort_policy(AiBehaviorAbortPolicy::LowerPriority)
                    .with_child("hold_position"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "hold_position",
                        AiBehaviorNodeKind::Task,
                        "Hold Position",
                    )
                    .with_parameter("result", "succeeded"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "reposition",
                        AiBehaviorNodeKind::Task,
                        "Reposition",
                    )
                    .with_parameter("result", "running"),
                ),
        )
        .expect("vec3 condition tree");
    let tick = |value| {
        manager
            .tick_agent(AiAgentTickRequest {
                world: WorldHandle::new(17),
                entity: 12,
                behavior_tree: Some(tree_id),
                blackboard_schema: Some(schema_id),
                delta_seconds: 1.0 / 60.0,
                blackboard: vec![AiBlackboardEntry::new(
                    "target_point",
                    AiBlackboardValue::Vec3(value),
                )],
                perception: None,
            })
            .expect("vec3 condition tick")
    };
    assert_eq!(
        tick(Vec3::new(4.0, 0.0, -1.5)).status,
        AiDecisionStatus::Running
    );
    assert_eq!(tick(target_point).status, AiDecisionStatus::Succeeded);
}
