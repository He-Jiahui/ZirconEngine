use std::sync::atomic::Ordering;

use zircon_runtime::core::framework::ai::{
    AiAgentTickRequest, AiBehaviorAbortPolicy, AiBehaviorNodeDescriptor, AiBehaviorNodeKind,
    AiBehaviorTreeDescriptor, AiBlackboardEntry, AiBlackboardValue, AiDecisionStatus, AiManager,
    AiManagerError,
};
use zircon_runtime::core::framework::scene::WorldHandle;

use super::observer_abort::{
    guarded_fallback_tree, manager_with_schema, single_external_tree, tick_without_schema,
    DISABLE_ABORTS, SWITCH_ABORTS,
};

#[test]
fn observer_binding_rejects_missing_and_unknown_schema_keys() {
    for (key, expected_missing) in [(None, true), (Some("missing"), false)] {
        let (manager, schema) = manager_with_schema();
        let mut guard =
            AiBehaviorNodeDescriptor::new("guard", AiBehaviorNodeKind::Decorator, "Guard")
                .with_abort_policy(AiBehaviorAbortPolicy::LowerPriority)
                .with_child("task");
        if let Some(key) = key {
            guard = guard.with_parameter("blackboard_key", key);
        }
        let tree = manager
            .register_behavior_tree(
                AiBehaviorTreeDescriptor::new("invalid_observer", "Invalid Observer", "guard")
                    .with_node(guard)
                    .with_node(
                        AiBehaviorNodeDescriptor::new("task", AiBehaviorNodeKind::Task, "Task")
                            .with_parameter("result", "running"),
                    ),
            )
            .expect("descriptor validation remains schema-independent");
        let error = manager
            .tick_agent(AiAgentTickRequest {
                world: WorldHandle::new(902),
                entity: 18,
                behavior_tree: Some(tree),
                blackboard_schema: Some(schema),
                delta_seconds: 0.1,
                blackboard: vec![AiBlackboardEntry::new(
                    "alert",
                    AiBlackboardValue::Bool(false),
                )],
                perception: None,
            })
            .expect_err("observer binding must reject an invalid schema key");
        assert!(
            matches!(
                &error,
                AiManagerError::BehaviorObserverMissingBlackboardKey { .. }
                    if expected_missing
            ) || matches!(
                &error,
                AiManagerError::BehaviorObserverUnknownBlackboardKey { .. }
                    if !expected_missing
            )
        );
    }
}

#[test]
fn root_or_reachable_subtree_observer_requires_a_bound_schema() {
    let (manager, _) = manager_with_schema();
    let child = manager
        .register_behavior_tree(guarded_fallback_tree(false))
        .expect("observer child tree");
    for tree in [
        child,
        manager
            .register_behavior_tree(
                AiBehaviorTreeDescriptor::new("observer_parent", "Observer Parent", "root")
                    .with_node(
                        AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Subtree, "Root")
                            .with_implementation("run_subtree")
                            .with_parameter("behavior_tree", "observer_abort"),
                    ),
            )
            .expect("observer parent tree"),
    ] {
        let error = manager
            .tick_agent(AiAgentTickRequest {
                world: WorldHandle::new(903),
                entity: tree.raw(),
                behavior_tree: Some(tree),
                blackboard_schema: None,
                delta_seconds: 0.1,
                blackboard: Vec::new(),
                perception: None,
            })
            .expect_err("observer tree without schema must be rejected");
        assert!(matches!(
            error,
            AiManagerError::BehaviorObserverRequiresBlackboardSchema { .. }
        ));
    }
}

#[test]
fn switching_or_disabling_a_tree_aborts_active_tasks() {
    SWITCH_ABORTS.store(0, Ordering::SeqCst);
    let (manager, _) = manager_with_schema();
    let first = manager
        .register_behavior_tree(single_external_tree("switch_a", "test.switch_abort_probe"))
        .expect("switch source tree");
    let second = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("switch_b", "Switch B", "root").with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root")
                    .with_parameter("result", "running"),
            ),
        )
        .expect("switch target tree");
    assert_eq!(
        tick_without_schema(&manager, Some(first), 31).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick_without_schema(&manager, Some(second), 31).status,
        AiDecisionStatus::Running
    );
    assert_eq!(SWITCH_ABORTS.load(Ordering::SeqCst), 1);

    DISABLE_ABORTS.store(0, Ordering::SeqCst);
    let (manager, _) = manager_with_schema();
    let tree = manager
        .register_behavior_tree(single_external_tree("disable", "test.disable_abort_probe"))
        .expect("disable source tree");
    assert_eq!(
        tick_without_schema(&manager, Some(tree), 32).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick_without_schema(&manager, None, 32).status,
        AiDecisionStatus::Idle
    );
    assert_eq!(DISABLE_ABORTS.load(Ordering::SeqCst), 1);
}
