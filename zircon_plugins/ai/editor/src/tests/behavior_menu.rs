use zircon_plugin_ai_runtime::behavior_tree::{
    standard_node_catalog, BehaviorNodeCatalog, BehaviorNodeCategory, BehaviorNodeDescriptor,
    BehaviorNodeSemantics, FrozenBehaviorNodeCatalog,
};
use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor as Node, AiBehaviorNodeKind as Kind,
};
use zircon_runtime::plugin::PluginModuleId;

use crate::{
    AiBehaviorTreeAsset as Asset, AiBehaviorTreeEdit as Edit, AiBehaviorTreeMenuAdapter as Menu,
    AiBehaviorTreeMenuCommand as Command, AiBehaviorTreeMenuDisabledReason as Disabled,
    AiBehaviorTreeMenuError as Error, AiBehaviorTreeMenuRequest as Request,
    AiBlackboardProviderSnapshot as Providers,
};

// Unit-only source/catalog values. These are not live asset/provider registrations.
fn providers(catalog: &FrozenBehaviorNodeCatalog) -> Providers<'_> {
    Providers {
        catalog,
        registry_identity: "unit-only-provider-lifecycle",
        revision: 7,
        builtin_provider: catalog
            .owner_for_slot(catalog.resolve("wait").unwrap())
            .unwrap(),
    }
}

#[test]
fn selection_and_readonly_gates_do_not_create_partial_candidates() {
    let source = super::asset();
    let catalog = standard_node_catalog().unwrap();
    for (selection, writable, reason) in [
        (&[][..], true, Disabled::NoSelection),
        (
            &["wait", "set_score"][..],
            true,
            Disabled::MultipleSelection,
        ),
        (&["removed_node"][..], true, Disabled::MissingNode),
        (&["wait"][..], false, Disabled::ReadOnly),
    ] {
        let menu = Menu::new(&source, selection, writable, providers(&catalog));
        assert_eq!(menu.eligibility(Command::DeleteSubtree), Err(reason));
        assert!(
            matches!(menu.prepare(Request::DeleteSubtree), Err(Error::Disabled(actual)) if actual == reason)
        );
    }
    assert_eq!(source.descriptor(), &super::effect_tree());
}

#[test]
fn search_borrows_real_catalog_and_preserves_exact_subtree_creation_kind() {
    let source = super::asset();
    let catalog = standard_node_catalog().unwrap();
    let menu = Menu::new(&source, &["root"], true, providers(&catalog));
    assert_eq!(menu.node_rows(Command::CreateChild, "").count(), 11);
    assert_eq!(menu.node_rows(Command::WrapDecorator, "").count(), 6);
    assert_eq!(menu.node_rows(Command::AddService, "").count(), 0);
    let rows: Vec<_> = menu
        .node_rows(Command::CreateChild, "  run\tSUBtree  ")
        .collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].descriptor.id(), "run_subtree");
    assert_eq!(rows[0].kind, Some(Kind::Subtree));
    assert!(rows[0].disabled.is_none());
    assert!(std::ptr::eq(
        rows[0].descriptor,
        catalog
            .get(catalog.resolve("run_subtree").unwrap())
            .unwrap()
    ));
    assert_eq!(
        menu.rows("delete SUB").next().unwrap().command,
        Command::DeleteSubtree
    );
    assert_eq!(menu.rows("no matching command").count(), 0);
}

#[test]
fn create_subtree_uses_supplied_registration_parameter_and_rejects_task_kind() {
    let source = super::asset();
    let catalog = standard_node_catalog().unwrap();
    let menu = Menu::new(&source, &["root"], true, providers(&catalog));
    let subtree = super::subtree_node();
    let candidate = menu
        .prepare(Request::CreateChild {
            index: 1,
            node: &subtree,
        })
        .unwrap()
        .unwrap();
    assert_eq!(candidate.after().descriptor().nodes.last(), Some(&subtree));
    assert_eq!(
        candidate.after().descriptor().nodes[0].children,
        ["set_score", "call_patrol", "emit_guard_event", "wait"]
    );
    assert_eq!(candidate.inverse_from(candidate.after()), Some(&source));
    let mut task = subtree.clone();
    task.kind = Kind::Task;
    assert!(matches!(
        menu.prepare(Request::CreateChild {
            index: 1,
            node: &task
        }),
        Err(Error::Disabled(Disabled::NodeKindMismatch))
    ));
    assert_eq!(source.descriptor(), &super::effect_tree());
}

#[test]
fn root_leaf_and_required_decorator_child_are_distinct_disabled_reasons() {
    let source = super::asset();
    let catalog = standard_node_catalog().unwrap();
    assert_eq!(
        Menu::new(&source, &["root"], true, providers(&catalog))
            .eligibility(Command::DeleteSubtree),
        Err(Disabled::RootDeletion)
    );
    assert_eq!(
        Menu::new(&source, &["wait"], true, providers(&catalog)).eligibility(Command::CreateChild),
        Err(Disabled::LeafHasNoChildren)
    );
    assert_eq!(
        Menu::new(&source, &["wait"], true, providers(&catalog))
            .eligibility(Command::UnwrapDecorator),
        Err(Disabled::NotDecorator)
    );
    let wrapper = Node::new("guard", Kind::Decorator, "Guard").with_implementation("inverter");
    let wrapped = source
        .edited(&Edit::WrapDecorator {
            child_id: "wait".to_owned(),
            decorator: wrapper,
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        Menu::new(&wrapped, &["guard"], true, providers(&catalog))
            .eligibility(Command::CreateChild),
        Err(Disabled::RequiredDecoratorChild)
    );
    assert_eq!(
        Menu::new(&wrapped, &["wait"], true, providers(&catalog))
            .eligibility(Command::DeleteSubtree),
        Err(Disabled::RequiredDecoratorChild)
    );
}

#[test]
fn wrapper_root_and_subtree_deletion_return_one_exact_inverse_value() {
    let source = super::asset();
    let catalog = standard_node_catalog().unwrap();
    let wrapper =
        Node::new("root_guard", Kind::Decorator, "Root Guard").with_implementation("inverter");
    let wrapped = Menu::new(&source, &["root"], true, providers(&catalog))
        .prepare(Request::WrapDecorator {
            decorator: &wrapper,
        })
        .unwrap()
        .unwrap();
    assert_eq!(wrapped.after().descriptor().root_node, "root_guard");
    assert_eq!(wrapped.inverse_from(wrapped.after()), Some(&source));
    let unwrapped = Menu::new(wrapped.after(), &["root_guard"], true, providers(&catalog))
        .prepare(Request::UnwrapDecorator)
        .unwrap()
        .unwrap();
    assert_eq!(unwrapped.after(), &source);
    assert_eq!(
        unwrapped.inverse_from(unwrapped.after()),
        Some(wrapped.after())
    );
    let deleted = Menu::new(&source, &["set_score"], true, providers(&catalog))
        .prepare(Request::DeleteSubtree)
        .unwrap()
        .unwrap();
    assert_eq!(
        deleted.after().descriptor().nodes.len(),
        source.descriptor().nodes.len() - 1
    );
    assert_eq!(
        deleted.inverse_from(deleted.after()).unwrap().to_toml(),
        source.to_toml()
    );
    let intervening = source
        .edited(&Edit::SetTreeName {
            display_name: "intervening".to_owned(),
        })
        .unwrap()
        .unwrap();
    assert!(deleted.forward_from(&intervening).is_none());
}

#[test]
fn full_compiler_rejects_bad_parameter_index_duplicate_and_prepopulated_children() {
    let source = super::asset();
    let catalog = standard_node_catalog().unwrap();
    let menu = Menu::new(&source, &["root"], true, providers(&catalog));
    let bad_parameter = Node::new("new_wait", Kind::Task, "Wait")
        .with_implementation("wait")
        .with_parameter("duration_seconds", "invalid");
    let duplicate = Node::new("wait", Kind::Task, "Wait").with_implementation("wait");
    let children = Node::new("child_wait", Kind::Task, "Wait")
        .with_implementation("wait")
        .with_child("wait");
    for node in [&bad_parameter, &duplicate, &children] {
        assert!(matches!(
            menu.prepare(Request::CreateChild { index: 1, node }),
            Err(Error::Candidate(_))
        ));
    }
    let valid = Node::new("new_wait", Kind::Task, "Wait").with_implementation("wait");
    assert!(matches!(
        menu.prepare(Request::CreateChild {
            index: 99,
            node: &valid
        }),
        Err(Error::Candidate(_))
    ));
    assert_eq!(source.descriptor(), &super::effect_tree());
}

#[test]
fn weighted_topology_reuses_edit_guard_for_create_delete_wrap_and_unwrap() {
    let mut descriptor = super::effect_tree();
    descriptor.nodes[0].kind = Kind::Selector;
    descriptor.nodes[0].implementation = "random_selector".to_owned();
    descriptor.nodes[0].parameters.push(
        zircon_runtime::core::framework::ai::AiBehaviorNodeParameter::new(
            "weight.set_score",
            2.0_f32,
        ),
    );
    let source = Asset::from_descriptor(descriptor).unwrap();
    let catalog = standard_node_catalog().unwrap();
    for (selected, command) in [
        ("root", Command::CreateChild),
        ("set_score", Command::DeleteSubtree),
        ("set_score", Command::WrapDecorator),
    ] {
        assert_eq!(
            Menu::new(&source, &[selected], true, providers(&catalog)).eligibility(command),
            Err(Disabled::WeightedTopology)
        );
    }
    // Root wrapping preserves every weighted child ID and order.
    assert!(Menu::new(&source, &["root"], true, providers(&catalog))
        .eligibility(Command::WrapDecorator)
        .is_ok());

    let mut wrapped = source.descriptor().clone();
    wrapped.nodes[0].children[0] = "guard".to_owned();
    wrapped.nodes[0].parameters[0].key = "weight.guard".to_owned();
    wrapped.nodes.push(
        Node::new("guard", Kind::Decorator, "Guard")
            .with_implementation("inverter")
            .with_child("set_score"),
    );
    let wrapped = Asset::from_descriptor(wrapped).unwrap();
    assert_eq!(
        Menu::new(&wrapped, &["guard"], true, providers(&catalog))
            .eligibility(Command::UnwrapDecorator),
        Err(Disabled::WeightedTopology)
    );
}

#[test]
fn changed_builtin_semantics_or_category_cannot_reuse_the_creation_contract() {
    let source = super::asset();
    for (category, semantics) in [
        (
            BehaviorNodeCategory::Task,
            BehaviorNodeSemantics::RunSubtree,
        ),
        (BehaviorNodeCategory::Decorator, BehaviorNodeSemantics::Wait),
    ] {
        let mut catalog = BehaviorNodeCatalog::default();
        catalog
            .add_node(
                PluginModuleId::from_raw(123),
                BehaviorNodeDescriptor::new("wait", "Changed Wait", category, semantics),
            )
            .unwrap();
        let catalog = catalog.freeze();
        let menu = Menu::new(&source, &["root"], true, providers(&catalog));
        let rows: Vec<_> = menu
            .node_rows(
                if category == BehaviorNodeCategory::Task {
                    Command::CreateChild
                } else {
                    Command::WrapDecorator
                },
                "wait",
            )
            .collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, None);
        assert_eq!(rows[0].disabled, Some(Disabled::UnsupportedProvider));
        let node = Node::new("new_wait", Kind::Task, "Wait").with_implementation("wait");
        assert!(matches!(
            menu.prepare(Request::CreateChild {
                index: 1,
                node: &node
            }),
            Err(Error::Disabled(Disabled::UnsupportedProvider))
        ));
    }
}

#[test]
fn missing_lifecycle_and_external_chinese_rows_never_invent_a_creation_schema() {
    let source = super::asset();
    let catalog = standard_node_catalog().unwrap();
    let mut missing = providers(&catalog);
    missing.registry_identity = " ";
    let menu = Menu::new(&source, &["root"], true, missing);
    assert!(menu
        .node_rows(Command::CreateChild, "wait")
        .all(|row| row.disabled == Some(Disabled::MissingProviderIdentity)));
    let node = Node::new("new_wait", Kind::Task, "Wait").with_implementation("wait");
    assert!(matches!(
        menu.prepare(Request::CreateChild {
            index: 1,
            node: &node
        }),
        Err(Error::Disabled(Disabled::MissingProviderIdentity))
    ));
    let mut custom = BehaviorNodeCatalog::with_standard_nodes().unwrap();
    custom
        .add_node(
            PluginModuleId::from_raw(123),
            BehaviorNodeDescriptor::new(
                "external_chase",
                "中文追踪任务",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::External,
            ),
        )
        .unwrap();
    let custom = custom.freeze();
    let external = Menu::new(&source, &["root"], true, providers(&custom));
    let rows: Vec<_> = external
        .node_rows(Command::CreateChild, "中文 任务")
        .collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].kind, None);
    assert_eq!(rows[0].disabled, Some(Disabled::UnsupportedProvider));
    assert_eq!(
        external
            .node_rows(Command::CreateChild, "unknown implementation")
            .count(),
        0
    );
}

#[test]
fn service_and_trace_actions_stay_disabled_without_a_fake_success_path() {
    let source = super::asset();
    let catalog = standard_node_catalog().unwrap();
    let menu = Menu::new(&source, &["wait"], true, providers(&catalog));
    for (request, reason) in [
        (Request::AddService, Disabled::ServiceAttachmentUnsupported),
        (
            Request::ToggleBreakpoint,
            Disabled::RuntimeTraceControlUnsupported,
        ),
        (Request::StepInto, Disabled::RuntimeTraceControlUnsupported),
        (Request::StepOver, Disabled::RuntimeTraceControlUnsupported),
    ] {
        assert_eq!(menu.eligibility(request.command()), Err(reason));
        assert!(matches!(menu.prepare(request), Err(Error::Disabled(actual)) if actual == reason));
    }
    assert_eq!(source.descriptor(), &super::effect_tree());
}
