use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorNodeParameterValue,
    AiBehaviorTreeDescriptor, AiBlackboardSchemaDescriptor, AiManager, AiManagerError,
};
use zircon_runtime::core::math::Vec3;

use crate::DefaultAiManager;

#[test]
fn ai_manager_validates_behavior_tree_and_blackboard_contracts() {
    let manager = DefaultAiManager::default();
    let tree = AiBehaviorTreeDescriptor::new("patrol", "Patrol", "root").with_node(
        AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Sequence, "Root"),
    );
    let tree_id = manager
        .register_behavior_tree(tree)
        .expect("valid behavior tree");
    assert_eq!(tree_id.raw(), 1);

    let missing_root = AiBehaviorTreeDescriptor::new("broken", "Broken", "missing");
    assert_eq!(
        manager.register_behavior_tree(missing_root),
        Err(AiManagerError::MissingRootNode {
            tree_id: "broken".to_string(),
            root_node: "missing".to_string()
        })
    );

    let missing_child = AiBehaviorTreeDescriptor::new("missing_child", "Missing Child", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Sequence, "Root")
                .with_child("task"),
        );
    assert_eq!(
        manager.register_behavior_tree(missing_child),
        Err(AiManagerError::MissingChildNode {
            tree_id: "missing_child".to_string(),
            node_id: "root".to_string(),
            child_id: "task".to_string()
        })
    );

    let cyclic_tree = AiBehaviorTreeDescriptor::new("cyclic_tree", "Broken", "root").with_node(
        AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Sequence, "Root")
            .with_child("root"),
    );
    assert_eq!(
        manager.register_behavior_tree(cyclic_tree),
        Err(AiManagerError::InvalidBehaviorTreeTopology {
            tree_id: "cyclic_tree".to_string(),
            node_id: "root".to_string(),
            reason: "node participates in a cycle"
        })
    );

    let duplicate_child_edge =
        AiBehaviorTreeDescriptor::new("duplicate_child_edge", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                    .with_child("task")
                    .with_child("task"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "task",
                AiBehaviorNodeKind::Task,
                "Task",
            ));
    assert_eq!(
        manager.register_behavior_tree(duplicate_child_edge),
        Err(AiManagerError::InvalidBehaviorTreeTopology {
            tree_id: "duplicate_child_edge".to_string(),
            node_id: "task".to_string(),
            reason: "node must have exactly one incoming edge"
        })
    );

    let shared_child = AiBehaviorTreeDescriptor::new("shared_child", "Broken", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("branch_a")
                .with_child("branch_b"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("branch_a", AiBehaviorNodeKind::Sequence, "Branch A")
                .with_child("shared"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("branch_b", AiBehaviorNodeKind::Sequence, "Branch B")
                .with_child("shared"),
        )
        .with_node(AiBehaviorNodeDescriptor::new(
            "shared",
            AiBehaviorNodeKind::Task,
            "Shared",
        ));
    assert_eq!(
        manager.register_behavior_tree(shared_child),
        Err(AiManagerError::InvalidBehaviorTreeTopology {
            tree_id: "shared_child".to_string(),
            node_id: "shared".to_string(),
            reason: "node must have exactly one incoming edge"
        })
    );

    let unreachable_node = AiBehaviorTreeDescriptor::new("unreachable_node", "Broken", "root")
        .with_node(AiBehaviorNodeDescriptor::new(
            "root",
            AiBehaviorNodeKind::Sequence,
            "Root",
        ))
        .with_node(AiBehaviorNodeDescriptor::new(
            "orphan",
            AiBehaviorNodeKind::Task,
            "Orphan",
        ));
    assert_eq!(
        manager.register_behavior_tree(unreachable_node),
        Err(AiManagerError::InvalidBehaviorTreeTopology {
            tree_id: "unreachable_node".to_string(),
            node_id: "orphan".to_string(),
            reason: "node is not reachable from root"
        })
    );

    let decorator_without_child =
        AiBehaviorTreeDescriptor::new("decorator_without_child", "Broken", "root").with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                .with_parameter("blackboard_key", "can_see_player")
                .with_parameter("exists", true),
        );
    assert_eq!(
        manager.register_behavior_tree(decorator_without_child),
        Err(AiManagerError::InvalidBehaviorNodeChildCount {
            tree_id: "decorator_without_child".to_string(),
            node_id: "root".to_string(),
            expected: "exactly one",
            actual: 0
        })
    );

    let decorator_with_extra_child =
        AiBehaviorTreeDescriptor::new("decorator_with_extra_child", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                    .with_parameter("blackboard_key", "can_see_player")
                    .with_parameter("exists", true)
                    .with_child("first")
                    .with_child("second"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "first",
                AiBehaviorNodeKind::Task,
                "First",
            ))
            .with_node(AiBehaviorNodeDescriptor::new(
                "second",
                AiBehaviorNodeKind::Task,
                "Second",
            ));
    assert_eq!(
        manager.register_behavior_tree(decorator_with_extra_child),
        Err(AiManagerError::InvalidBehaviorNodeChildCount {
            tree_id: "decorator_with_extra_child".to_string(),
            node_id: "root".to_string(),
            expected: "exactly one",
            actual: 2
        })
    );

    let task_with_child = AiBehaviorTreeDescriptor::new("task_with_child", "Broken", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root")
                .with_child("ignored_child"),
        )
        .with_node(AiBehaviorNodeDescriptor::new(
            "ignored_child",
            AiBehaviorNodeKind::Task,
            "Ignored Child",
        ));
    assert_eq!(
        manager.register_behavior_tree(task_with_child),
        Err(AiManagerError::InvalidBehaviorNodeChildCount {
            tree_id: "task_with_child".to_string(),
            node_id: "root".to_string(),
            expected: "zero",
            actual: 1
        })
    );

    let subtree_with_child = AiBehaviorTreeDescriptor::new("subtree_with_child", "Broken", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Subtree, "Root")
                .with_child("inline_child"),
        )
        .with_node(AiBehaviorNodeDescriptor::new(
            "inline_child",
            AiBehaviorNodeKind::Task,
            "Inline Child",
        ));
    assert_eq!(
        manager.register_behavior_tree(subtree_with_child),
        Err(AiManagerError::InvalidBehaviorNodeChildCount {
            tree_id: "subtree_with_child".to_string(),
            node_id: "root".to_string(),
            expected: "zero",
            actual: 1
        })
    );

    let duplicate_parameter =
        AiBehaviorTreeDescriptor::new("duplicate_parameter", "Broken", "root").with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root")
                .with_parameter("result", "succeeded")
                .with_parameter("result", "failed"),
        );
    assert_eq!(
        manager.register_behavior_tree(duplicate_parameter),
        Err(AiManagerError::DuplicateBehaviorNodeParameter {
            tree_id: "duplicate_parameter".to_string(),
            node_id: "root".to_string(),
            key: "result".to_string()
        })
    );

    let non_finite_parameter =
        AiBehaviorTreeDescriptor::new("non_finite_parameter", "Broken", "root").with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root").with_parameter(
                "equals_scalar",
                AiBehaviorNodeParameterValue::Scalar(f32::NAN),
            ),
        );
    assert_eq!(
        manager.register_behavior_tree(non_finite_parameter),
        Err(AiManagerError::NonFiniteBehaviorNodeParameter {
            tree_id: "non_finite_parameter".to_string(),
            node_id: "root".to_string(),
            key: "equals_scalar".to_string()
        })
    );

    let missing_blackboard_key =
        AiBehaviorTreeDescriptor::new("missing_blackboard_key_parameter", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                    .with_parameter("equals_bool", true)
                    .with_child("child"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "child",
                AiBehaviorNodeKind::Task,
                "Child",
            ));
    assert_eq!(
        manager.register_behavior_tree(missing_blackboard_key),
        Err(AiManagerError::InvalidBehaviorNodeParameter {
            tree_id: "missing_blackboard_key_parameter".to_string(),
            node_id: "root".to_string(),
            key: "blackboard_key".to_string(),
            expected: "string",
            actual: "missing"
        })
    );

    let missing_blackboard_key_for_invert =
        AiBehaviorTreeDescriptor::new("missing_blackboard_key_for_invert", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                    .with_parameter("invert", true)
                    .with_child("child"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "child",
                AiBehaviorNodeKind::Task,
                "Child",
            ));
    assert_eq!(
        manager.register_behavior_tree(missing_blackboard_key_for_invert),
        Err(AiManagerError::InvalidBehaviorNodeParameter {
            tree_id: "missing_blackboard_key_for_invert".to_string(),
            node_id: "root".to_string(),
            key: "blackboard_key".to_string(),
            expected: "string",
            actual: "missing"
        })
    );

    let wrong_parameter_type =
        AiBehaviorTreeDescriptor::new("wrong_parameter_type", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                    .with_parameter("blackboard_key", "can_see_player")
                    .with_parameter("equals_bool", "true")
                    .with_child("child"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "child",
                AiBehaviorNodeKind::Task,
                "Child",
            ));
    assert_eq!(
        manager.register_behavior_tree(wrong_parameter_type),
        Err(AiManagerError::InvalidBehaviorNodeParameter {
            tree_id: "wrong_parameter_type".to_string(),
            node_id: "root".to_string(),
            key: "equals_bool".to_string(),
            expected: "bool",
            actual: "string"
        })
    );

    let wrong_invert_parameter_type =
        AiBehaviorTreeDescriptor::new("wrong_invert_parameter_type", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                    .with_parameter("blackboard_key", "can_see_player")
                    .with_parameter("invert", "true")
                    .with_child("child"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "child",
                AiBehaviorNodeKind::Task,
                "Child",
            ));
    assert_eq!(
        manager.register_behavior_tree(wrong_invert_parameter_type),
        Err(AiManagerError::InvalidBehaviorNodeParameter {
            tree_id: "wrong_invert_parameter_type".to_string(),
            node_id: "root".to_string(),
            key: "invert".to_string(),
            expected: "bool",
            actual: "string"
        })
    );

    let wrong_numeric_parameter_type =
        AiBehaviorTreeDescriptor::new("wrong_numeric_parameter_type", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                    .with_parameter("blackboard_key", "ammo")
                    .with_parameter("greater_than_integer", 3.0_f32)
                    .with_child("child"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "child",
                AiBehaviorNodeKind::Task,
                "Child",
            ));
    assert_eq!(
        manager.register_behavior_tree(wrong_numeric_parameter_type),
        Err(AiManagerError::InvalidBehaviorNodeParameter {
            tree_id: "wrong_numeric_parameter_type".to_string(),
            node_id: "root".to_string(),
            key: "greater_than_integer".to_string(),
            expected: "integer",
            actual: "scalar"
        })
    );

    let task_with_perception_condition =
        AiBehaviorTreeDescriptor::new("task_with_perception_condition", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root")
                    .with_parameter("perception_sense", "sight"),
            );
    assert_eq!(
        manager.register_behavior_tree(task_with_perception_condition),
        Err(AiManagerError::InvalidBehaviorNodeParameterOwner {
            tree_id: "task_with_perception_condition".to_string(),
            node_id: "root".to_string(),
            key: "perception_sense".to_string(),
            expected: "`decorator` nodes"
        })
    );

    let unknown_perception_sense =
        AiBehaviorTreeDescriptor::new("unknown_perception_sense", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                    .with_parameter("perception_sense", "smell")
                    .with_child("child"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "child",
                AiBehaviorNodeKind::Task,
                "Child",
            ));
    assert_eq!(
        manager.register_behavior_tree(unknown_perception_sense),
        Err(AiManagerError::InvalidBehaviorNodeParameterValue {
            tree_id: "unknown_perception_sense".to_string(),
            node_id: "root".to_string(),
            key: "perception_sense".to_string(),
            expected: "`sight`, `hearing`, `damage`, `touch`, or `custom`",
            actual: "smell".to_string()
        })
    );

    let wrong_perception_strength_type =
        AiBehaviorTreeDescriptor::new("wrong_perception_strength_type", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                    .with_parameter("perception_min_strength", 1_i64)
                    .with_child("child"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "child",
                AiBehaviorNodeKind::Task,
                "Child",
            ));
    assert_eq!(
        manager.register_behavior_tree(wrong_perception_strength_type),
        Err(AiManagerError::InvalidBehaviorNodeParameter {
            tree_id: "wrong_perception_strength_type".to_string(),
            node_id: "root".to_string(),
            key: "perception_min_strength".to_string(),
            expected: "scalar",
            actual: "integer"
        })
    );

    let negative_perception_age =
        AiBehaviorTreeDescriptor::new("negative_perception_age", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                    .with_parameter("perception_max_age_seconds", -0.1_f32)
                    .with_child("child"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "child",
                AiBehaviorNodeKind::Task,
                "Child",
            ));
    assert_eq!(
        manager.register_behavior_tree(negative_perception_age),
        Err(AiManagerError::InvalidBehaviorNodeParameterValue {
            tree_id: "negative_perception_age".to_string(),
            node_id: "root".to_string(),
            key: "perception_max_age_seconds".to_string(),
            expected: "a non-negative scalar",
            actual: "-0.1".to_string()
        })
    );

    let wrong_vec3_parameter_type =
        AiBehaviorTreeDescriptor::new("wrong_vec3_parameter_type", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                    .with_parameter("blackboard_key", "target_point")
                    .with_parameter("equals_vec3", 3.0_f32)
                    .with_child("child"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "child",
                AiBehaviorNodeKind::Task,
                "Child",
            ));
    assert_eq!(
        manager.register_behavior_tree(wrong_vec3_parameter_type),
        Err(AiManagerError::InvalidBehaviorNodeParameter {
            tree_id: "wrong_vec3_parameter_type".to_string(),
            node_id: "root".to_string(),
            key: "equals_vec3".to_string(),
            expected: "vec3",
            actual: "scalar"
        })
    );

    let non_finite_vec3_parameter =
        AiBehaviorTreeDescriptor::new("non_finite_vec3_parameter", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                    .with_parameter("blackboard_key", "target_point")
                    .with_parameter("equals_vec3", Vec3::new(0.0, f32::NAN, 0.0))
                    .with_child("child"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "child",
                AiBehaviorNodeKind::Task,
                "Child",
            ));
    assert_eq!(
        manager.register_behavior_tree(non_finite_vec3_parameter),
        Err(AiManagerError::NonFiniteBehaviorNodeParameter {
            tree_id: "non_finite_vec3_parameter".to_string(),
            node_id: "root".to_string(),
            key: "equals_vec3".to_string()
        })
    );

    let invalid_parallel_policy =
        AiBehaviorTreeDescriptor::new("invalid_parallel_policy", "Broken", "root").with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Parallel, "Root")
                .with_parameter("success_policy", "majority"),
        );
    assert_eq!(
        manager.register_behavior_tree(invalid_parallel_policy),
        Err(AiManagerError::InvalidBehaviorNodeParameterValue {
            tree_id: "invalid_parallel_policy".to_string(),
            node_id: "root".to_string(),
            key: "success_policy".to_string(),
            expected: "`all` or `any`",
            actual: "majority".to_string()
        })
    );

    let invalid_task_result =
        AiBehaviorTreeDescriptor::new("invalid_task_result", "Broken", "root").with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root")
                .with_parameter("result", "complete"),
        );
    assert_eq!(
        manager.register_behavior_tree(invalid_task_result),
        Err(AiManagerError::InvalidBehaviorNodeParameterValue {
            tree_id: "invalid_task_result".to_string(),
            node_id: "root".to_string(),
            key: "result".to_string(),
            expected: "`idle`, `running`, `succeeded`, `failed`, or `blocked`",
            actual: "complete".to_string()
        })
    );

    let task_with_parallel_policy =
        AiBehaviorTreeDescriptor::new("task_with_parallel_policy", "Broken", "root").with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root")
                .with_parameter("success_policy", "any"),
        );
    assert_eq!(
        manager.register_behavior_tree(task_with_parallel_policy),
        Err(AiManagerError::InvalidBehaviorNodeParameterOwner {
            tree_id: "task_with_parallel_policy".to_string(),
            node_id: "root".to_string(),
            key: "success_policy".to_string(),
            expected: "`parallel` nodes"
        })
    );

    let selector_with_blackboard_condition =
        AiBehaviorTreeDescriptor::new("selector_with_blackboard_condition", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                    .with_parameter("blackboard_key", "target")
                    .with_parameter("exists", true),
            );
    assert_eq!(
        manager.register_behavior_tree(selector_with_blackboard_condition),
        Err(AiManagerError::InvalidBehaviorNodeParameterOwner {
            tree_id: "selector_with_blackboard_condition".to_string(),
            node_id: "root".to_string(),
            key: "blackboard_key".to_string(),
            expected: "`decorator` nodes"
        })
    );

    let decorator_with_task_result =
        AiBehaviorTreeDescriptor::new("decorator_with_task_result", "Broken", "root")
            .with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Root")
                    .with_parameter("result", "succeeded")
                    .with_child("child"),
            )
            .with_node(AiBehaviorNodeDescriptor::new(
                "child",
                AiBehaviorNodeKind::Task,
                "Child",
            ));
    assert_eq!(
        manager.register_behavior_tree(decorator_with_task_result),
        Err(AiManagerError::InvalidBehaviorNodeParameterOwner {
            tree_id: "decorator_with_task_result".to_string(),
            node_id: "root".to_string(),
            key: "result".to_string(),
            expected: "`task` nodes"
        })
    );

    let subtree_without_target =
        AiBehaviorTreeDescriptor::new("subtree_without_target", "Broken", "root").with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Subtree, "Root"),
        );
    assert_eq!(
        manager.register_behavior_tree(subtree_without_target),
        Err(AiManagerError::InvalidBehaviorNodeParameter {
            tree_id: "subtree_without_target".to_string(),
            node_id: "root".to_string(),
            key: "behavior_tree".to_string(),
            expected: "string",
            actual: "missing"
        })
    );

    let subtree_with_unknown_target =
        AiBehaviorTreeDescriptor::new("subtree_with_unknown_target", "Broken", "root").with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Subtree, "Root")
                .with_parameter("behavior_tree", "missing_tree"),
        );
    assert_eq!(
        manager.register_behavior_tree(subtree_with_unknown_target),
        Err(AiManagerError::InvalidBehaviorSubtreeTarget {
            tree_id: "subtree_with_unknown_target".to_string(),
            node_id: "root".to_string(),
            target_tree: "missing_tree".to_string(),
            reason: "target behavior tree is not registered"
        })
    );

    let subtree_with_self_target =
        AiBehaviorTreeDescriptor::new("subtree_with_self_target", "Broken", "root").with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Subtree, "Root")
                .with_parameter("behavior_tree", "subtree_with_self_target"),
        );
    assert_eq!(
        manager.register_behavior_tree(subtree_with_self_target),
        Err(AiManagerError::InvalidBehaviorSubtreeTarget {
            tree_id: "subtree_with_self_target".to_string(),
            node_id: "root".to_string(),
            target_tree: "subtree_with_self_target".to_string(),
            reason: "subtree cannot target itself"
        })
    );

    let task_with_subtree_target =
        AiBehaviorTreeDescriptor::new("task_with_subtree_target", "Broken", "root").with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root")
                .with_parameter("behavior_tree", "patrol"),
        );
    assert_eq!(
        manager.register_behavior_tree(task_with_subtree_target),
        Err(AiManagerError::InvalidBehaviorNodeParameterOwner {
            tree_id: "task_with_subtree_target".to_string(),
            node_id: "root".to_string(),
            key: "behavior_tree".to_string(),
            expected: "`subtree` nodes"
        })
    );

    let valid_subtree_target =
        AiBehaviorTreeDescriptor::new("valid_subtree_target", "Valid", "root").with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Subtree, "Root")
                .with_parameter("behavior_tree", "patrol"),
        );
    assert!(manager.register_behavior_tree(valid_subtree_target).is_ok());

    let task_with_custom_parameter =
        AiBehaviorTreeDescriptor::new("task_with_custom_parameter", "Custom", "root").with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root")
                .with_parameter("plugin.custom_probe", "preserved"),
        );
    assert!(manager
        .register_behavior_tree(task_with_custom_parameter)
        .is_ok());

    let duplicate_key_schema = AiBlackboardSchemaDescriptor::new("agent", "Agent")
        .with_key("target", "entity", true)
        .with_key("target", "entity", false);
    assert_eq!(
        manager.register_blackboard_schema(duplicate_key_schema),
        Err(AiManagerError::DuplicateBlackboardKey {
            schema_id: "agent".to_string(),
            key: "target".to_string()
        })
    );

    let unknown_key_type = AiBlackboardSchemaDescriptor::new("agent_unknown_type", "Agent")
        .with_key("target", "object_ref", true);
    assert_eq!(
        manager.register_blackboard_schema(unknown_key_type),
        Err(AiManagerError::UnknownBlackboardValueType {
            schema_id: "agent_unknown_type".to_string(),
            key: "target".to_string(),
            value_type: "object_ref".to_string()
        })
    );
}
