use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor,
};

use crate::behavior_tree::{
    compile_behavior_tree, compile_behavior_tree_toml, BehaviorNodeCategory,
    BehaviorTreeAssetError, BehaviorTreeCompileError,
};

fn node(id: &str, kind: AiBehaviorNodeKind) -> AiBehaviorNodeDescriptor {
    AiBehaviorNodeDescriptor::new(id, kind, id)
}

#[test]
fn compiled_tree_preorder_ranges_are_consistent() {
    let descriptor = AiBehaviorTreeDescriptor::new("patrol", "Patrol", "root")
        .with_node(
            node("root", AiBehaviorNodeKind::Selector)
                .with_child("attack")
                .with_child("patrol_sequence"),
        )
        .with_node(node("attack", AiBehaviorNodeKind::Task))
        .with_node(
            node("patrol_sequence", AiBehaviorNodeKind::Sequence)
                .with_child("move")
                .with_child("wait"),
        )
        .with_node(node("move", AiBehaviorNodeKind::Task))
        .with_node(node("wait", AiBehaviorNodeKind::Task));

    let compiled = compile_behavior_tree(&descriptor).expect("valid tree compiles");

    assert_eq!(
        compiled.node_ids(),
        ["root", "attack", "patrol_sequence", "move", "wait"]
    );
    assert_eq!(compiled.child_indices(compiled.root()), &[1, 2]);
    assert!(compiled.child_indices(compiled.node(1)).is_empty());
    assert_eq!(compiled.child_indices(compiled.node(2)), &[3, 4]);
    assert!(compiled.child_indices(compiled.node(3)).is_empty());
    assert!(compiled.child_indices(compiled.node(4)).is_empty());
}

#[test]
fn compile_rejects_duplicate_child_ownership_instead_of_duplicating_nodes() {
    let descriptor = AiBehaviorTreeDescriptor::new("shared", "Shared", "root")
        .with_node(
            node("root", AiBehaviorNodeKind::Selector)
                .with_child("left")
                .with_child("right"),
        )
        .with_node(node("left", AiBehaviorNodeKind::Sequence).with_child("leaf"))
        .with_node(node("right", AiBehaviorNodeKind::Sequence).with_child("leaf"))
        .with_node(node("leaf", AiBehaviorNodeKind::Task));

    assert_eq!(
        compile_behavior_tree(&descriptor),
        Err(BehaviorTreeCompileError::MultipleParents {
            node_id: "leaf".to_string(),
        })
    );
}

#[test]
fn compile_rejects_cycles_with_a_typed_error() {
    let descriptor = AiBehaviorTreeDescriptor::new("cycle", "Cycle", "root")
        .with_node(node("root", AiBehaviorNodeKind::Sequence).with_child("child"))
        .with_node(node("child", AiBehaviorNodeKind::Sequence).with_child("root"));

    assert_eq!(
        compile_behavior_tree(&descriptor),
        Err(BehaviorTreeCompileError::Cycle {
            node_id: "root".to_string(),
        })
    );
}

#[test]
fn btree_toml_asset_loads_and_compiles_to_dense_preorder() {
    let source = r#"
format_version = 1
id = "patrol"
display_name = "Patrol"
root_node = "root"

[[nodes]]
id = "root"
kind = "sequence"
implementation = "sequence"
display_name = "Root"
children = ["wait"]

[[nodes]]
id = "wait"
kind = "task"
implementation = "wait"
display_name = "Wait"
"#;

    let compiled = compile_behavior_tree_toml(source).expect("valid .btree.toml asset");

    assert_eq!(compiled.id(), "patrol");
    assert_eq!(compiled.node_ids(), ["root", "wait"]);
    assert_eq!(compiled.child_indices(compiled.root()), &[1]);
}

#[test]
fn btree_toml_asset_runs_full_descriptor_validation_before_compile() {
    let source = r#"
format_version = 1
id = "invalid"
display_name = "Invalid"
root_node = "root"

[[nodes]]
id = "root"
kind = "decorator"
implementation = "blackboard_condition"
display_name = "Missing Child"
"#;

    assert!(matches!(
        compile_behavior_tree_toml(source),
        Err(BehaviorTreeAssetError::Validation(
            zircon_runtime::core::framework::ai::AiManagerError::InvalidBehaviorNodeChildCount {
                actual: 0,
                ..
            }
        ))
    ));
}

#[test]
fn compile_rejects_dto_kind_and_implementation_category_mismatch() {
    let descriptor = AiBehaviorTreeDescriptor::new("mismatch", "Mismatch", "root").with_node(
        AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Wrong")
            .with_implementation("selector"),
    );

    assert_eq!(
        compile_behavior_tree(&descriptor),
        Err(BehaviorTreeCompileError::ImplementationCategoryMismatch {
            node_id: "root".to_string(),
            implementation: "selector".to_string(),
            expected: BehaviorNodeCategory::Task,
            actual: BehaviorNodeCategory::Composite,
        })
    );
}

#[test]
fn btree_toml_requires_an_explicit_format_version() {
    let source = r#"
id = "legacy"
display_name = "Legacy"
root_node = "root"
nodes = []
"#;

    assert!(matches!(
        compile_behavior_tree_toml(source),
        Err(BehaviorTreeAssetError::Parse(_))
    ));
}
