use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor,
    AiBlackboardSchemaDescriptor, AiManager, AiManagerError,
};

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
