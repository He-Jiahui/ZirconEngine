use zircon_runtime::core::framework::ai::{
    AiAgentTickRequest, AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor,
    AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardValue, AiDecisionStatus,
    AiManager, AiManagerError, AiPerceptionSense, AiPerceptionSnapshot, AiPerceptionStimulus,
};
use zircon_runtime::core::framework::scene::WorldHandle;

use crate::DefaultAiManager;

#[test]
fn ai_manager_stores_blackboard_and_reports_staged_tick_status() {
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
        .expect("staged AI tick should accept valid blackboard input");

    assert_eq!(report.status, AiDecisionStatus::Blocked);
    assert_eq!(report.active_node.as_deref(), Some("root"));
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
            position: zircon_runtime::core::math::Vec3::new(1.0, 2.0, 3.0),
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
