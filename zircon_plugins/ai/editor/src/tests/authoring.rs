use zircon_plugin_ai_runtime::behavior_tree::{BehaviorNodeSemantics, BehaviorTreeAssetError};
use zircon_plugin_ai_runtime::DefaultAiManager;
#[path = "behavior_graph.rs"]
mod behavior_graph;
#[path = "behavior_menu.rs"]
mod behavior_menu;
#[path = "blackboard_references.rs"]
mod blackboard_references;
use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor as Node, AiBehaviorNodeKind as Kind,
    AiBehaviorNodeParameter as Parameter, AiBehaviorNodeParameterValue as Value,
    AiBehaviorTreeDescriptor as Tree, AiManager, AiManagerError,
};
use zircon_runtime::core::math::Vec3;

use crate::{
    AiBehaviorTreeAsset as Asset, AiBehaviorTreeEdit as Edit, AiBehaviorTreeEditError as Error,
};

fn effect_tree() -> Tree {
    Tree::new("typed_effects_wait", "Typed Effects Wait", "root")
        .with_node(
            Node::new("root", Kind::Sequence, "Sequence")
                .with_child("set_score")
                .with_child("emit_guard_event")
                .with_child("wait"),
        )
        .with_node(
            Node::new("set_score", Kind::Task, "Set Blackboard")
                .with_implementation("set_blackboard")
                .with_parameter("blackboard_key", "score")
                .with_parameter("value", 42_i64),
        )
        .with_node(
            Node::new("emit_guard_event", Kind::Task, "Emit Event")
                .with_implementation("emit_event")
                .with_parameter("event_name", "ai.guard_passed"),
        )
        .with_node(
            Node::new("wait", Kind::Task, "Wait").with_parameter("duration_seconds", 10.0_f32),
        )
}

fn asset() -> Asset {
    Asset::from_descriptor(effect_tree()).expect("real runtime validation and compilation")
}

fn compiled_children(asset: &Asset, id: &str) -> Vec<String> {
    let compiled = asset.compiled();
    let parent = compiled
        .nodes()
        .iter()
        .find(|node| node.id() == id)
        .unwrap();
    compiled
        .child_indices(parent)
        .iter()
        .map(|index| compiled.node(*index as usize).id().to_owned())
        .collect()
}

fn subtree_node() -> Node {
    Node::new("call_patrol", Kind::Subtree, "Run Patrol")
        .with_implementation("run_subtree")
        .with_parameter("behavior_tree", "patrol_tree")
}

#[test]
fn typed_values_and_chinese_names_roundtrip_through_real_asset_compiler() {
    for value in [
        Value::Bool(true),
        Value::Integer(i64::MIN),
        Value::Scalar(1.25),
        Value::String("中文长键和值必须完整保留".to_owned()),
        Value::Vec3(Vec3::new(1.0, 2.0, -8.0)),
        Value::Entity(77),
    ] {
        let source = asset();
        let changed = source
            .edited(&Edit::SetParameter {
                node_id: "set_score".to_owned(),
                parameter: Parameter::new("value", value.clone()),
            })
            .unwrap()
            .unwrap();
        let changed = changed
            .edited(&Edit::SetTreeName {
                display_name: "行为树 — 完整保存中文长名称".to_owned(),
            })
            .unwrap()
            .unwrap();
        let reopened = Asset::from_toml(changed.to_toml()).unwrap();
        assert_eq!(reopened.descriptor(), changed.descriptor());
        assert_eq!(
            compiled_children(&reopened, "root"),
            ["set_score", "emit_guard_event", "wait"]
        );
        let parameter = reopened.descriptor().nodes[1]
            .parameters
            .iter()
            .find(|parameter| parameter.key == "value")
            .unwrap();
        assert_eq!(parameter.value, value);
        assert_eq!(source.descriptor(), &effect_tree());
    }
}

#[test]
fn full_runtime_validation_rejects_child_count_format_and_parameter_errors() {
    let mut bad_child_count = effect_tree();
    bad_child_count.nodes[1].children.push("wait".to_owned());
    let mut bad_version = effect_tree();
    bad_version.format_version += 1;
    let mut bad_parameter = effect_tree();
    bad_parameter.nodes[3].parameters[0].value = Value::String("ten".to_owned());
    let mut duplicate_parameter = effect_tree();
    duplicate_parameter.nodes[3]
        .parameters
        .push(Parameter::new("duration_seconds", 1.0_f32));
    for descriptor in [
        bad_child_count,
        bad_version,
        bad_parameter,
        duplicate_parameter,
    ] {
        assert!(matches!(
            Asset::from_descriptor(descriptor),
            Err(Error::Runtime(BehaviorTreeAssetError::Validation(_)))
        ));
    }
}

#[test]
fn legacy_effect_key_parameter_is_rejected_by_full_runtime_validation() {
    let mut descriptor = effect_tree();
    descriptor.nodes[1].parameters[0].key = "key".to_owned();
    assert!(matches!(
        Asset::from_descriptor(descriptor),
        Err(Error::Runtime(BehaviorTreeAssetError::Validation(_)))
    ));
}

#[test]
fn missing_implementation_and_nonfinite_value_never_replace_source() {
    let source = asset();
    let mut unknown = effect_tree();
    unknown.nodes[3].implementation = "missing_provider".to_owned();
    assert!(matches!(
        Asset::from_descriptor(unknown),
        Err(Error::Runtime(BehaviorTreeAssetError::Compile(_)))
    ));
    assert!(source
        .edited(&Edit::SetParameter {
            node_id: "wait".to_owned(),
            parameter: Parameter::new("duration_seconds", Value::Scalar(f32::NAN)),
        })
        .is_err());
    assert_eq!(source.descriptor(), &effect_tree());
}

#[test]
fn unknown_source_fields_are_rejected_instead_of_dropped_on_save() {
    let source = asset();
    assert!(
        matches!(Asset::from_toml(&format!("future = true\n{}", source.to_toml())),
        Err(Error::UnknownField { path }) if path == "future")
    );
    let mut value: toml::Value = toml::from_str(source.to_toml()).unwrap();
    let nodes = value.get_mut("nodes").unwrap().as_array_mut().unwrap();
    nodes[1]
        .as_table_mut()
        .unwrap()
        .insert("future".to_owned(), toml::Value::Boolean(true));
    assert!(
        matches!(Asset::from_toml(&toml::to_string(&value).unwrap()),
        Err(Error::UnknownField { path }) if path == "nodes[1].future")
    );
}

#[test]
fn subtree_creation_and_target_edit_preserve_kind_identity_and_order() {
    let source = asset();
    let inserted = source
        .edited(&Edit::InsertChild {
            parent_id: "root".to_owned(),
            index: 1,
            node: subtree_node(),
        })
        .unwrap()
        .unwrap();
    let changed = inserted
        .edited(&Edit::SetParameter {
            node_id: "call_patrol".to_owned(),
            parameter: Parameter::new("behavior_tree", "chase_tree"),
        })
        .unwrap()
        .unwrap();
    let reopened = Asset::from_toml(changed.to_toml()).unwrap();
    assert_eq!(reopened, changed);
    assert_eq!(
        compiled_children(&reopened, "root"),
        ["set_score", "call_patrol", "emit_guard_event", "wait"]
    );
    let compiled = reopened.compiled();
    let call = compiled
        .nodes()
        .iter()
        .find(|node| node.id() == "call_patrol")
        .unwrap();
    assert_eq!(call.kind(), Kind::Subtree);
    assert_eq!(call.semantics(), BehaviorNodeSemantics::RunSubtree);
    assert!(compiled.child_indices(call).is_empty());
    assert_eq!(
        call.parameters(),
        [Parameter::new("behavior_tree", "chase_tree")]
    );
    assert_eq!(inserted.descriptor().nodes.last().unwrap(), &subtree_node());
    assert_eq!(source.descriptor(), &effect_tree());
}

#[test]
fn invalid_subtree_creation_and_parameter_edits_leave_sources_unchanged() {
    let source = asset();
    let mut wrong_kind = subtree_node();
    wrong_kind.kind = Kind::Task;
    let mut missing_target = subtree_node();
    missing_target.parameters.clear();
    for node in [wrong_kind, missing_target] {
        assert!(matches!(
            source.edited(&Edit::InsertChild {
                parent_id: "root".to_owned(),
                index: 0,
                node,
            }),
            Err(Error::Runtime(BehaviorTreeAssetError::Validation(_)))
        ));
    }
    let inserted = source
        .edited(&Edit::InsertChild {
            parent_id: "root".to_owned(),
            index: 1,
            node: subtree_node(),
        })
        .unwrap()
        .unwrap();
    for value in [
        Value::Integer(1),
        Value::String(" \t".to_owned()),
        Value::String(source.descriptor().id.clone()),
    ] {
        assert!(matches!(
            inserted.edited(&Edit::SetParameter {
                node_id: "call_patrol".to_owned(),
                parameter: Parameter::new("behavior_tree", value),
            }),
            Err(Error::Runtime(BehaviorTreeAssetError::Validation(_)))
        ));
    }
    for edit in [
        Edit::RemoveParameter {
            node_id: "call_patrol".to_owned(),
            key: "behavior_tree".to_owned(),
        },
        Edit::InsertChild {
            parent_id: "call_patrol".to_owned(),
            index: 0,
            node: Node::new("invalid_child", Kind::Task, "Wait"),
        },
    ] {
        assert!(matches!(
            inserted.edited(&edit),
            Err(Error::Runtime(BehaviorTreeAssetError::Validation(_)))
        ));
    }
    assert_eq!(inserted.descriptor().nodes.last().unwrap(), &subtree_node());
    assert_eq!(Asset::from_toml(inserted.to_toml()).unwrap(), inserted);
    assert_eq!(source.descriptor(), &effect_tree());
}

#[test]
fn a_compiled_subtree_requires_a_registered_target_before_manager_admission() {
    let source = asset();
    let candidate = source
        .edited(&Edit::InsertChild {
            parent_id: "root".to_owned(),
            index: 1,
            node: subtree_node(),
        })
        .unwrap()
        .unwrap();
    let manager = DefaultAiManager::default();
    assert_eq!(
        manager.register_behavior_tree(candidate.descriptor().clone()),
        Err(AiManagerError::InvalidBehaviorSubtreeTarget {
            tree_id: source.descriptor().id.clone(),
            node_id: "call_patrol".to_owned(),
            target_tree: "patrol_tree".to_owned(),
            reason: "target behavior tree is not registered",
        })
    );
    let mut target = effect_tree();
    target.id = "patrol_tree".to_owned();
    manager.register_behavior_tree(target).unwrap();
    manager
        .register_behavior_tree(candidate.descriptor().clone())
        .unwrap();
    assert_eq!(
        candidate.descriptor().nodes.last().unwrap(),
        &subtree_node()
    );
    assert_eq!(source.descriptor(), &effect_tree());
}

#[test]
fn reordered_children_keep_ids_and_use_destination_index_after_removal() {
    let source = asset();
    let changed = source
        .edited(&Edit::Reparent {
            node_id: "set_score".to_owned(),
            parent_id: "root".to_owned(),
            index: 2,
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        compiled_children(&changed, "root"),
        ["emit_guard_event", "wait", "set_score"]
    );
    assert_eq!(
        compiled_children(&source, "root"),
        ["set_score", "emit_guard_event", "wait"]
    );
    assert_eq!(
        changed.descriptor().nodes,
        source
            .descriptor()
            .nodes
            .iter()
            .cloned()
            .map(|mut node| {
                if node.id == "root" {
                    node.children = vec![
                        "emit_guard_event".to_owned(),
                        "wait".to_owned(),
                        "set_score".to_owned(),
                    ];
                }
                node
            })
            .collect::<Vec<_>>()
    );
    let roundtrip = Asset::from_toml(changed.to_toml()).unwrap();
    assert_eq!(roundtrip.descriptor(), changed.descriptor());
}

#[test]
fn wrapper_creation_and_removal_keep_child_identity_and_sibling_order() {
    let source = asset();
    let wrapped = source
        .edited(&Edit::WrapDecorator {
            child_id: "set_score".to_owned(),
            decorator: Node::new("guard", Kind::Decorator, "Invert")
                .with_implementation("inverter"),
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        compiled_children(&wrapped, "root"),
        ["guard", "emit_guard_event", "wait"]
    );
    assert_eq!(compiled_children(&wrapped, "guard"), ["set_score"]);
    assert_eq!(wrapped.descriptor().nodes[1], source.descriptor().nodes[1]);
    let unwrapped = wrapped
        .edited(&Edit::UnwrapDecorator {
            node_id: "guard".to_owned(),
        })
        .unwrap()
        .unwrap();
    assert_eq!(unwrapped, source);
    assert!(wrapped
        .edited(&Edit::Reparent {
            node_id: "set_score".to_owned(),
            parent_id: "root".to_owned(),
            index: 0,
        })
        .is_err());
    assert_eq!(compiled_children(&wrapped, "guard"), ["set_score"]);
}

#[test]
fn root_wrapper_can_be_removed_without_reidentifying_original_nodes() {
    let source = asset();
    let wrapped = source
        .edited(&Edit::WrapDecorator {
            child_id: "root".to_owned(),
            decorator: Node::new("guard", Kind::Decorator, "Invert")
                .with_implementation("inverter"),
        })
        .unwrap()
        .unwrap();
    assert_eq!(wrapped.descriptor().root_node, "guard");
    let restored = wrapped
        .edited(&Edit::UnwrapDecorator {
            node_id: "guard".to_owned(),
        })
        .unwrap()
        .unwrap();
    assert_eq!(restored, source);
}

#[test]
fn subtree_delete_and_failed_move_are_atomic_source_candidates() {
    let source = asset();
    let branch = source
        .edited(&Edit::WrapDecorator {
            child_id: "set_score".to_owned(),
            decorator: Node::new("guard", Kind::Decorator, "Invert")
                .with_implementation("inverter"),
        })
        .unwrap()
        .unwrap();
    let deleted = branch
        .edited(&Edit::DeleteSubtree {
            node_id: "guard".to_owned(),
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        compiled_children(&deleted, "root"),
        ["emit_guard_event", "wait"]
    );
    assert!(!deleted
        .descriptor()
        .nodes
        .iter()
        .any(|node| node.id == "guard" || node.id == "set_score"));
    assert!(matches!(
        source.edited(&Edit::DeleteSubtree {
            node_id: "root".to_owned()
        }),
        Err(Error::RootEdit { .. })
    ));
    assert!(matches!(
        source.edited(&Edit::Reparent {
            node_id: "wait".to_owned(),
            parent_id: "root".to_owned(),
            index: 99,
        }),
        Err(Error::ChildIndex { .. })
    ));
    assert_eq!(source.descriptor(), &effect_tree());
}

#[test]
fn duplicate_id_invalid_parent_and_cycle_leave_source_unchanged() {
    let source = asset();
    assert!(matches!(
        source.edited(&Edit::InsertChild {
            parent_id: "root".to_owned(),
            index: 0,
            node: Node::new("wait", Kind::Task, "Duplicate"),
        }),
        Err(Error::DuplicateNode { .. })
    ));
    assert!(source
        .edited(&Edit::InsertChild {
            parent_id: "wait".to_owned(),
            index: 0,
            node: Node::new("new", Kind::Task, "Wait"),
        })
        .is_err());
    let branch = source
        .edited(&Edit::InsertChild {
            parent_id: "root".to_owned(),
            index: 0,
            node: Node::new("branch", Kind::Sequence, "Branch"),
        })
        .unwrap()
        .unwrap();
    assert!(branch
        .edited(&Edit::Reparent {
            node_id: "branch".to_owned(),
            parent_id: "branch".to_owned(),
            index: 0,
        })
        .is_err());
    assert_eq!(source.descriptor(), &effect_tree());
}

#[test]
fn weighted_topology_requires_explicit_weight_mapping() {
    for key in ["weight_0", "weight.set_score"] {
        let mut tree = effect_tree();
        tree.nodes[0].implementation = "random_selector".to_owned();
        tree.nodes[0].parameters.push(Parameter::new(key, 3.0_f32));
        let weighted = Asset::from_descriptor(tree).unwrap();
        assert!(matches!(
            weighted.edited(&Edit::Reparent {
                node_id: "set_score".to_owned(),
                parent_id: "root".to_owned(),
                index: 2,
            }),
            Err(Error::WeightedTopology { .. })
        ));
        assert_eq!(
            compiled_children(&weighted, "root"),
            ["set_score", "emit_guard_event", "wait"]
        );
    }
}

#[test]
fn unchanged_edits_return_none_without_creating_a_dirty_candidate() {
    let source = asset();
    for edit in [
        Edit::SetTreeName {
            display_name: source.descriptor().display_name.clone(),
        },
        Edit::SetNodeName {
            node_id: "wait".to_owned(),
            display_name: "Wait".to_owned(),
        },
        Edit::SetParameter {
            node_id: "wait".to_owned(),
            parameter: Parameter::new("duration_seconds", 10.0_f32),
        },
        Edit::Reparent {
            node_id: "wait".to_owned(),
            parent_id: "root".to_owned(),
            index: 2,
        },
        Edit::RemoveParameter {
            node_id: "wait".to_owned(),
            key: "absent".to_owned(),
        },
    ] {
        assert!(source.edited(&edit).unwrap().is_none());
    }
}

#[test]
fn prepared_subtree_delete_has_an_exact_inverse_with_original_storage_order() {
    let source = asset();
    let wrapped = source
        .edited(&Edit::WrapDecorator {
            child_id: "set_score".to_owned(),
            decorator: Node::new("guard", Kind::Decorator, "Invert")
                .with_implementation("inverter"),
        })
        .unwrap()
        .unwrap();
    let candidate = wrapped
        .prepare_edit(&Edit::DeleteSubtree {
            node_id: "guard".to_owned(),
        })
        .unwrap()
        .unwrap();
    assert_eq!(candidate.before(), &wrapped);
    let forward = candidate.forward_from(&wrapped).unwrap();
    assert_eq!(
        compiled_children(forward, "root"),
        ["emit_guard_event", "wait"]
    );
    let inverse = candidate.inverse_from(forward).unwrap();
    assert_eq!(inverse, &wrapped);
    assert_eq!(inverse.to_toml(), wrapped.to_toml());
    assert_eq!(inverse.descriptor().nodes, wrapped.descriptor().nodes);
    assert_eq!(compiled_children(inverse, "guard"), ["set_score"]);
    assert_eq!(source.descriptor(), &effect_tree());
}

#[test]
fn prepared_root_wrapper_and_reorder_keep_identity_through_both_directions() {
    let source = asset();
    for edit in [
        Edit::WrapDecorator {
            child_id: "root".to_owned(),
            decorator: Node::new("root_guard", Kind::Decorator, "Invert")
                .with_implementation("inverter"),
        },
        Edit::Reparent {
            node_id: "set_score".to_owned(),
            parent_id: "root".to_owned(),
            index: 2,
        },
    ] {
        let candidate = source.prepare_edit(&edit).unwrap().unwrap();
        let reopened = Asset::from_toml(candidate.after().to_toml()).unwrap();
        assert_eq!(candidate.inverse_from(&reopened), Some(&source));
        assert_eq!(candidate.forward_from(&source), Some(candidate.after()));
        assert_eq!(candidate.before().descriptor().id, reopened.descriptor().id);
        assert_eq!(
            candidate.before().descriptor().nodes.len()
                + usize::from(matches!(edit, Edit::WrapDecorator { .. })),
            reopened.descriptor().nodes.len()
        );
    }
}

#[test]
fn prepared_edit_rejects_intervening_source_changes_in_either_direction() {
    let source = asset();
    let candidate = source
        .prepare_edit(&Edit::SetParameter {
            node_id: "wait".to_owned(),
            parameter: Parameter::new("duration_seconds", 2.0_f32),
        })
        .unwrap()
        .unwrap();
    let concurrent = source
        .edited(&Edit::SetNodeName {
            node_id: "emit_guard_event".to_owned(),
            display_name: "另一处编辑".to_owned(),
        })
        .unwrap()
        .unwrap();
    assert!(candidate.forward_from(&concurrent).is_none());
    assert!(candidate.inverse_from(&concurrent).is_none());
    assert!(candidate.inverse_from(&source).is_none());
    assert!(candidate.forward_from(candidate.after()).is_none());
    let changed_after = candidate
        .after()
        .edited(&Edit::SetTreeName {
            display_name: "edited after forward".to_owned(),
        })
        .unwrap()
        .unwrap();
    assert!(candidate.inverse_from(&changed_after).is_none());
    assert_eq!(candidate.before(), &source);
}

#[test]
fn prepared_noop_and_invalid_edit_produce_no_forward_or_inverse_value() {
    let source = asset();
    assert!(source
        .prepare_edit(&Edit::SetTreeName {
            display_name: source.descriptor().display_name.clone(),
        })
        .unwrap()
        .is_none());
    assert!(source
        .prepare_edit(&Edit::DeleteSubtree {
            node_id: "root".to_owned(),
        })
        .is_err());
    assert!(source
        .prepare_edit(&Edit::SetParameter {
            node_id: "wait".to_owned(),
            parameter: Parameter::new("duration_seconds", Value::String("invalid".to_owned())),
        })
        .is_err());
    assert_eq!(source.descriptor(), &effect_tree());
}
