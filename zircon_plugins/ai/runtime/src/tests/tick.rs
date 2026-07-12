use zircon_runtime::core::framework::ai::{
    AiAgentTickRequest, AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor,
    AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardValue, AiDecisionStatus,
    AiManager, AiManagerError, AiPerceptionSense, AiPerceptionSnapshot, AiPerceptionStimulus,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Vec3;

use crate::DefaultAiManager;

#[test]
fn node_semantics_matrix() {
    let manager = DefaultAiManager::default();
    let world = WorldHandle::new(1);
    for (index, status_name, expected) in [
        (1_u64, "running", AiDecisionStatus::Running),
        (2_u64, "succeeded", AiDecisionStatus::Succeeded),
        (3_u64, "failed", AiDecisionStatus::Failed),
    ] {
        let tree_id = manager
            .register_behavior_tree(
                AiBehaviorTreeDescriptor::new(format!("task_{status_name}"), status_name, "root")
                    .with_node(
                        AiBehaviorNodeDescriptor::new(
                            "root",
                            AiBehaviorNodeKind::Task,
                            status_name,
                        )
                        .with_parameter("result", status_name),
                    ),
            )
            .expect("task semantics tree");
        let report = manager
            .tick_agent(AiAgentTickRequest {
                world,
                entity: index,
                behavior_tree: Some(tree_id),
                blackboard_schema: None,
                delta_seconds: 1.0 / 60.0,
                blackboard: Vec::new(),
                perception: None,
            })
            .expect("compiled task tick");
        assert_eq!(report.status, expected);
    }
}

#[test]
fn ai_manager_stores_blackboard_and_reports_running_task_status() {
    let manager = DefaultAiManager::default();
    let tree_id = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("idle", "Idle", "root").with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Wait"),
            ),
        )
        .expect("valid tree");
    let world = WorldHandle::new(7);
    let entity = 42;
    let blackboard = vec![AiBlackboardEntry {
        key: "can_see_player".to_string(),
        value: AiBlackboardValue::Bool(true),
    }];

    let report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: blackboard.clone(),
            perception: None,
        })
        .expect("AI tick should accept valid blackboard input");

    assert_eq!(report.status, AiDecisionStatus::Running);
    assert_eq!(report.active_node.as_deref(), Some("root"));
    assert_eq!(report.diagnostic, None);
    assert_eq!(manager.blackboard_entries(world, entity), blackboard);
    let snapshot = manager.runtime_snapshot();
    assert_eq!(snapshot.behavior_trees.len(), 1);
    assert_eq!(snapshot.agents.len(), 1);
    assert_eq!(snapshot.agents[0].blackboard.len(), 1);
}

#[test]
fn ai_manager_validates_tick_schema_and_perception_boundaries() {
    let manager = DefaultAiManager::default();
    let schema_id = manager
        .register_blackboard_schema(
            AiBlackboardSchemaDescriptor::new("combat_agent", "Combat Agent")
                .with_key("can_see_player", "bool", true)
                .with_key("target", "entity", false),
        )
        .expect("valid blackboard schema");
    let world = WorldHandle::new(9);
    let entity = 77;

    let missing_required = manager.tick_agent(AiAgentTickRequest {
        world,
        entity,
        behavior_tree: None,
        blackboard_schema: Some(schema_id),
        delta_seconds: 1.0 / 30.0,
        blackboard: Vec::new(),
        perception: None,
    });
    assert_eq!(
        missing_required,
        Err(AiManagerError::MissingBlackboardKey {
            schema_id: "combat_agent".to_string(),
            key: "can_see_player".to_string()
        })
    );

    let type_mismatch = manager.tick_agent(AiAgentTickRequest {
        world,
        entity,
        behavior_tree: None,
        blackboard_schema: Some(schema_id),
        delta_seconds: 1.0 / 30.0,
        blackboard: vec![AiBlackboardEntry::new(
            "can_see_player",
            AiBlackboardValue::Scalar(1.0),
        )],
        perception: None,
    });
    assert_eq!(
        type_mismatch,
        Err(AiManagerError::BlackboardValueTypeMismatch {
            schema_id: "combat_agent".to_string(),
            key: "can_see_player".to_string(),
            expected: "bool".to_string(),
            actual: "scalar".to_string()
        })
    );

    let perception_mismatch = manager.tick_agent(AiAgentTickRequest {
        world,
        entity,
        behavior_tree: None,
        blackboard_schema: Some(schema_id),
        delta_seconds: 1.0 / 30.0,
        blackboard: vec![AiBlackboardEntry::new(
            "can_see_player",
            AiBlackboardValue::Bool(true),
        )],
        perception: Some(AiPerceptionSnapshot {
            agent: entity + 1,
            stimuli: Vec::new(),
        }),
    });
    assert_eq!(
        perception_mismatch,
        Err(AiManagerError::PerceptionAgentMismatch {
            expected: entity,
            actual: entity + 1
        })
    );

    let perception = AiPerceptionSnapshot {
        agent: entity,
        stimuli: vec![AiPerceptionStimulus {
            source: 100,
            sense: AiPerceptionSense::Sight,
            position: Vec3::new(1.0, 2.0, 3.0),
            strength: 0.9,
            age_seconds: 0.1,
        }],
    };
    let report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: None,
            blackboard_schema: Some(schema_id),
            delta_seconds: 1.0 / 30.0,
            blackboard: vec![AiBlackboardEntry::new(
                "can_see_player",
                AiBlackboardValue::Bool(true),
            )],
            perception: Some(perception.clone()),
        })
        .expect("valid schema and perception tick");
    assert_eq!(report.status, AiDecisionStatus::Idle);
    assert_eq!(manager.perception_snapshot(world, entity), Some(perception));
    assert_eq!(
        manager.runtime_snapshot().agents[0]
            .perception
            .as_ref()
            .expect("perception snapshot in runtime snapshot")
            .stimuli
            .len(),
        1
    );
}

#[test]
fn ai_manager_executes_selector_with_blackboard_condition_parameters() {
    let manager = DefaultAiManager::default();
    let tree_id = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("combat_selector", "Combat Selector", "root")
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
                    .with_parameter("blackboard_key", "can_see_player")
                    .with_parameter("equals_bool", true)
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
        .expect("valid selector tree");
    let world = WorldHandle::new(10);
    let entity = 5;

    let fallback_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: vec![AiBlackboardEntry::new(
                "can_see_player",
                AiBlackboardValue::Bool(false),
            )],
            perception: None,
        })
        .expect("selector tick");
    assert_eq!(fallback_report.status, AiDecisionStatus::Running);
    assert_eq!(fallback_report.active_node.as_deref(), Some("patrol"));

    let attack_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: vec![AiBlackboardEntry::new(
                "can_see_player",
                AiBlackboardValue::Bool(true),
            )],
            perception: None,
        })
        .expect("selector tick after blackboard update");
    assert_eq!(attack_report.status, AiDecisionStatus::Succeeded);
    assert_eq!(attack_report.active_node.as_deref(), Some("attack"));
    assert_eq!(
        manager.runtime_snapshot().agents[0]
            .behavior_tree
            .as_deref(),
        Some("combat_selector")
    );
}

#[test]
fn ai_manager_sequence_fails_on_blackboard_condition_failure() {
    let manager = DefaultAiManager::default();
    let tree_id = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("guard_sequence", "Guard Sequence", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Sequence, "Root")
                        .with_child("has_target")
                        .with_child("engage"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "has_target",
                        AiBehaviorNodeKind::Decorator,
                        "Has Target",
                    )
                    .with_parameter("blackboard_key", "target")
                    .with_parameter("exists", true)
                    .with_child("target_ready"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "target_ready",
                        AiBehaviorNodeKind::Task,
                        "Target Ready",
                    )
                    .with_parameter("result", "succeeded"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("engage", AiBehaviorNodeKind::Task, "Engage")
                        .with_parameter("result", "succeeded"),
                ),
        )
        .expect("valid sequence tree");
    let world = WorldHandle::new(11);
    let entity = 6;

    let failed_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: Vec::new(),
            perception: None,
        })
        .expect("sequence tick");
    assert_eq!(failed_report.status, AiDecisionStatus::Failed);
    assert_eq!(failed_report.active_node.as_deref(), Some("has_target"));

    let succeeded_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: vec![AiBlackboardEntry::new(
                "target",
                AiBlackboardValue::Entity(99),
            )],
            perception: None,
        })
        .expect("sequence tick after target appears");
    assert_eq!(succeeded_report.status, AiDecisionStatus::Succeeded);
    assert_eq!(succeeded_report.active_node.as_deref(), Some("engage"));
}

#[test]
fn ai_manager_decorator_passes_when_absent_blackboard_entry_is_expected() {
    let manager = DefaultAiManager::default();
    let tree_id = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("target_absence", "Target Absence", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                        .with_child("target_missing")
                        .with_child("engage"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "target_missing",
                        AiBehaviorNodeKind::Decorator,
                        "Target Missing",
                    )
                    .with_parameter("blackboard_key", "target")
                    .with_parameter("exists", false)
                    .with_child("search"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("search", AiBehaviorNodeKind::Task, "Search")
                        .with_parameter("result", "succeeded"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("engage", AiBehaviorNodeKind::Task, "Engage")
                        .with_parameter("result", "running"),
                ),
        )
        .expect("valid absence condition tree");
    let world = WorldHandle::new(15);
    let entity = 10;

    let search_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: Vec::new(),
            perception: None,
        })
        .expect("selector tick without target");
    assert_eq!(search_report.status, AiDecisionStatus::Succeeded);
    assert_eq!(search_report.active_node.as_deref(), Some("search"));

    let engage_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: vec![AiBlackboardEntry::new(
                "target",
                AiBlackboardValue::Entity(99),
            )],
            perception: None,
        })
        .expect("selector tick with target");
    assert_eq!(engage_report.status, AiDecisionStatus::Running);
    assert_eq!(engage_report.active_node.as_deref(), Some("engage"));
}

#[test]
fn ai_manager_decorator_inverts_condition_gate_without_inverting_child_result() {
    let manager = DefaultAiManager::default();
    let tree_id = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("inverted_visibility", "Inverted Visibility", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                        .with_parameter("blackboard_key", "can_see_player")
                        .with_parameter("equals_bool", true)
                        .with_parameter("invert", true)
                        .with_child("retreat"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("retreat", AiBehaviorNodeKind::Task, "Retreat")
                        .with_parameter("result", "failed"),
                ),
        )
        .expect("valid inverted condition tree");
    let world = WorldHandle::new(16);
    let entity = 11;

    let child_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: vec![AiBlackboardEntry::new(
                "can_see_player",
                AiBlackboardValue::Bool(false),
            )],
            perception: None,
        })
        .expect("inverted decorator tick when raw condition is false");
    assert_eq!(child_report.status, AiDecisionStatus::Failed);
    assert_eq!(child_report.active_node.as_deref(), Some("retreat"));

    let decorator_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: vec![AiBlackboardEntry::new(
                "can_see_player",
                AiBlackboardValue::Bool(true),
            )],
            perception: None,
        })
        .expect("inverted decorator tick when raw condition is true");
    assert_eq!(decorator_report.status, AiDecisionStatus::Failed);
    assert_eq!(decorator_report.active_node.as_deref(), Some("root"));
}

#[test]
fn ai_manager_decorator_compares_numeric_blackboard_attributes() {
    let manager = DefaultAiManager::default();
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
        .expect("valid numeric condition tree");
    let world = WorldHandle::new(14);
    let entity = 9;

    let low_ammo_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: vec![
                AiBlackboardEntry::new("ammo", AiBlackboardValue::Integer(2)),
                AiBlackboardEntry::new("health", AiBlackboardValue::Scalar(0.8_f32)),
            ],
            perception: None,
        })
        .expect("selector tick with low ammo");
    assert_eq!(low_ammo_report.status, AiDecisionStatus::Running);
    assert_eq!(low_ammo_report.active_node.as_deref(), Some("recover"));

    let low_health_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: vec![
                AiBlackboardEntry::new("ammo", AiBlackboardValue::Integer(3)),
                AiBlackboardEntry::new("health", AiBlackboardValue::Scalar(0.35_f32)),
            ],
            perception: None,
        })
        .expect("selector tick with low health");
    assert_eq!(low_health_report.status, AiDecisionStatus::Running);
    assert_eq!(low_health_report.active_node.as_deref(), Some("recover"));

    let attack_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: vec![
                AiBlackboardEntry::new("ammo", AiBlackboardValue::Integer(5)),
                AiBlackboardEntry::new("health", AiBlackboardValue::Scalar(0.75_f32)),
            ],
            perception: None,
        })
        .expect("selector tick with valid numeric attributes");
    assert_eq!(attack_report.status, AiDecisionStatus::Succeeded);
    assert_eq!(attack_report.active_node.as_deref(), Some("attack"));
}

#[test]
fn ai_manager_decorator_compares_vec3_blackboard_attributes() {
    let manager = DefaultAiManager::default();
    let target_point = Vec3::new(4.0, 0.0, -2.0);
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
        .expect("valid vec3 condition tree");
    let world = WorldHandle::new(17);
    let entity = 12;

    let fallback_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: vec![AiBlackboardEntry::new(
                "target_point",
                AiBlackboardValue::Vec3(Vec3::new(4.0, 0.0, -1.5)),
            )],
            perception: None,
        })
        .expect("selector tick with different target point");
    assert_eq!(fallback_report.status, AiDecisionStatus::Running);
    assert_eq!(fallback_report.active_node.as_deref(), Some("reposition"));

    let hold_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: vec![AiBlackboardEntry::new(
                "target_point",
                AiBlackboardValue::Vec3(target_point),
            )],
            perception: None,
        })
        .expect("selector tick with matching target point");
    assert_eq!(hold_report.status, AiDecisionStatus::Succeeded);
    assert_eq!(hold_report.active_node.as_deref(), Some("hold_position"));
}

#[test]
fn ai_manager_parallel_requires_all_success_and_fails_on_any_failed_by_default() {
    let manager = DefaultAiManager::default();
    let tree_id = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("parallel_patrol", "Parallel Patrol", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Parallel, "Root")
                        .with_child("move")
                        .with_child("scan"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("move", AiBehaviorNodeKind::Task, "Move")
                        .with_parameter("result", "succeeded"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("scan", AiBehaviorNodeKind::Task, "Scan")
                        .with_parameter("result", "failed"),
                ),
        )
        .expect("valid parallel tree");
    let world = WorldHandle::new(12);
    let entity = 7;

    let failed_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: Vec::new(),
            perception: None,
        })
        .expect("parallel tick with default policies");
    assert_eq!(failed_report.status, AiDecisionStatus::Failed);
    assert_eq!(failed_report.active_node.as_deref(), Some("scan"));

    let tree_id = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("parallel_success", "Parallel Success", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Parallel, "Root")
                        .with_child("move")
                        .with_child("scan"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("move", AiBehaviorNodeKind::Task, "Move")
                        .with_parameter("result", "succeeded"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("scan", AiBehaviorNodeKind::Task, "Scan")
                        .with_parameter("result", "succeeded"),
                ),
        )
        .expect("valid all-success parallel tree");
    let succeeded_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: Vec::new(),
            perception: None,
        })
        .expect("parallel tick after all children succeed");
    assert_eq!(succeeded_report.status, AiDecisionStatus::Succeeded);
    assert_eq!(succeeded_report.active_node.as_deref(), Some("scan"));
}

#[test]
fn ai_manager_parallel_any_success_policy_keeps_running_until_any_child_succeeds() {
    let manager = DefaultAiManager::default();
    let tree_id = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("parallel_alert", "Parallel Alert", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Parallel, "Root")
                        .with_parameter("success_policy", "any")
                        .with_parameter("failure_policy", "all")
                        .with_child("patrol")
                        .with_child("alert"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("patrol", AiBehaviorNodeKind::Task, "Patrol")
                        .with_parameter("result", "running"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("alert", AiBehaviorNodeKind::Task, "Alert")
                        .with_parameter("result", "failed"),
                ),
        )
        .expect("valid any-success parallel tree");
    let world = WorldHandle::new(13);
    let entity = 8;

    let running_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: Vec::new(),
            perception: None,
        })
        .expect("parallel tick before any child succeeds");
    assert_eq!(running_report.status, AiDecisionStatus::Running);
    assert_eq!(running_report.active_node.as_deref(), Some("patrol"));

    let tree_id = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("parallel_alert_success", "Parallel Alert", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Parallel, "Root")
                        .with_parameter("success_policy", "any")
                        .with_parameter("failure_policy", "all")
                        .with_child("patrol")
                        .with_child("alert"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("patrol", AiBehaviorNodeKind::Task, "Patrol")
                        .with_parameter("result", "running"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("alert", AiBehaviorNodeKind::Task, "Alert")
                        .with_parameter("result", "succeeded"),
                ),
        )
        .expect("valid any-success parallel tree after alert");
    let succeeded_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: Vec::new(),
            perception: None,
        })
        .expect("parallel tick after one child succeeds");
    assert_eq!(succeeded_report.status, AiDecisionStatus::Succeeded);
    assert_eq!(succeeded_report.active_node.as_deref(), Some("alert"));
}

#[test]
fn ai_manager_subtree_node_executes_registered_behavior_tree_by_id() {
    let manager = DefaultAiManager::default();
    manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("combat_subtree", "Combat Subtree", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Can See")
                        .with_parameter("blackboard_key", "can_see_player")
                        .with_parameter("equals_bool", true)
                        .with_child("attack"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("attack", AiBehaviorNodeKind::Task, "Attack")
                        .with_parameter("result", "succeeded"),
                ),
        )
        .expect("valid registered subtree target");
    let parent_tree_id = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("combat_parent", "Combat Parent", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                        .with_child("run_combat")
                        .with_child("patrol"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "run_combat",
                        AiBehaviorNodeKind::Subtree,
                        "Run Combat",
                    )
                    .with_parameter("behavior_tree", "combat_subtree"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new("patrol", AiBehaviorNodeKind::Task, "Patrol")
                        .with_parameter("result", "running"),
                ),
        )
        .expect("valid parent tree with registered subtree");

    let world = WorldHandle::new(18);
    let entity = 13;
    let fallback_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(parent_tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: vec![AiBlackboardEntry::new(
                "can_see_player",
                AiBlackboardValue::Bool(false),
            )],
            perception: None,
        })
        .expect("subtree condition failure should let parent selector continue");
    assert_eq!(fallback_report.status, AiDecisionStatus::Running);
    assert_eq!(fallback_report.active_node.as_deref(), Some("patrol"));

    let attack_report = manager
        .tick_agent(AiAgentTickRequest {
            world,
            entity,
            behavior_tree: Some(parent_tree_id),
            blackboard_schema: None,
            delta_seconds: 1.0 / 60.0,
            blackboard: vec![AiBlackboardEntry::new(
                "can_see_player",
                AiBlackboardValue::Bool(true),
            )],
            perception: None,
        })
        .expect("subtree condition success should report subtree active node");
    assert_eq!(attack_report.status, AiDecisionStatus::Succeeded);
    assert_eq!(
        attack_report.active_node.as_deref(),
        Some("combat_subtree::attack")
    );
}
