use zircon_plugin_ai_runtime::behavior_tree::{
    standard_node_catalog, BehaviorNodeCatalog, BehaviorNodeDescriptor, BehaviorNodeSemantics,
    FrozenBehaviorNodeCatalog,
};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::ai::{
    AiBehaviorAbortPolicy, AiBehaviorNodeDescriptor as Node, AiBehaviorNodeKind as Kind,
    AiBehaviorTreeDescriptor as Tree, AiBlackboardSchemaDescriptor as Schema,
};
use zircon_runtime::plugin::PluginModuleId;

#[path = "blackboard_menu.rs"]
mod blackboard_menu;

use crate::{
    AiBehaviorTreeAsset as Asset, AiBlackboardDependencies as Dependencies,
    AiBlackboardKeyUsage as Usage, AiBlackboardProviderSnapshot as Providers,
    AiBlackboardReferenceBlocker as Blocker, AiBlackboardReferencePlan as Plan,
    AiBlackboardRenameError as Error, AiBlackboardSchemaInput as SchemaInput,
    AiBlackboardSourceVersion as Source, AiBlackboardTreeAssociation as Association,
    AiBlackboardTreeInput as TreeInput,
};

// Test-only identities exercise version checks; they are never catalog registrations.
fn source(uuid: &str, uri: &str, revision: u64, digest: u8) -> Source {
    Source {
        asset_uuid: uuid.parse().unwrap(),
        uri: AssetUri::parse(uri).unwrap(),
        revision,
        source_sha256: [digest; 32],
    }
}

fn tree(effect: bool) -> Asset {
    let leaf = if effect {
        Node::new("leaf", Kind::Task, "score")
            .with_implementation("set_blackboard")
            .with_parameter("blackboard_key", "score")
            .with_parameter("value", "score")
    } else {
        Node::new("leaf", Kind::Task, "score").with_implementation("wait")
    };
    // Authored order deliberately differs from compiled traversal order.
    Asset::from_descriptor(
        Tree::new("tree_business_id", "score", "root")
            .with_node(
                Node::new("event", Kind::Task, "score")
                    .with_implementation("emit_event")
                    .with_parameter("event_name", "score")
                    .with_parameter("payload", "score"),
            )
            .with_node(
                Node::new("root", Kind::Sequence, "score")
                    .with_child("guard")
                    .with_child("event"),
            )
            .with_node(
                Node::new("guard", Kind::Decorator, "score")
                    .with_implementation("blackboard_condition")
                    .with_child("leaf")
                    .with_parameter("blackboard_key", "score")
                    .with_parameter("equals_string", "score"),
            )
            .with_node(leaf),
    )
    .unwrap()
}

struct Fixture {
    schema: Schema,
    schema_source: Source,
    tree_source: Source,
    association: Association,
    asset: Asset,
    catalog: FrozenBehaviorNodeCatalog,
    builtin: PluginModuleId,
}

impl Fixture {
    fn new(asset: Asset) -> Self {
        let schema_source = source(
            "a02518f3-2e27-480d-81e1-13ca98d665c1",
            "res://ai/schema",
            7,
            31,
        );
        let tree_source = source(
            "a02518f3-2e27-480d-81e1-13ca98d665c2",
            "res://ai/tree",
            11,
            63,
        );
        let association = Association {
            tree: tree_source.clone(),
            tree_id: asset.descriptor().id.clone(),
            schema: schema_source.clone(),
            schema_id: "schema_business_id".to_owned(),
            owner: source(
                "a02518f3-2e27-480d-81e1-13ca98d665c3",
                "res://ai/association-fixture",
                3,
                127,
            ),
            field_path: "test_only.explicit_association".to_owned(),
        };
        let catalog = standard_node_catalog().unwrap();
        let builtin = catalog
            .owner_for_slot(catalog.resolve("wait").unwrap())
            .unwrap();
        Self {
            schema: Schema::new("schema_business_id", "score")
                .with_key("score", "string", true)
                .with_key("other", "bool", false),
            schema_source,
            tree_source,
            association,
            asset,
            catalog,
            builtin,
        }
    }

    fn input(&self) -> TreeInput<'_> {
        TreeInput {
            source: &self.tree_source,
            asset: &self.asset,
            association: Some(&self.association),
            writable: true,
        }
    }

    fn dependencies<'a>(&self, ids: &'a [zircon_runtime::asset::AssetUuid]) -> Dependencies<'a> {
        Dependencies {
            catalog_identity: "test-only-catalog-generation",
            revision: 19,
            tree_assets: ids,
            unresolved: &[],
        }
    }

    fn scan<'a>(
        &'a self,
        trees: &'a [TreeInput<'a>],
        dependencies: Option<Dependencies<'a>>,
    ) -> Plan<'a> {
        Plan::scan(
            SchemaInput {
                source: &self.schema_source,
                descriptor: &self.schema,
                writable: true,
            },
            trees,
            Providers {
                catalog: &self.catalog,
                registry_identity: "test-only-provider-lifecycle",
                revision: 23,
                builtin_provider: self.builtin,
            },
            dependencies,
        )
        .unwrap()
    }
}

#[test]
fn rename_changes_semantic_slots_and_inverse_restores_exact_typed_values() {
    let f = Fixture::new(tree(true));
    let inputs = [f.input()];
    let ids = [f.tree_source.asset_uuid];
    let plan = f.scan(&inputs, Some(f.dependencies(&ids)));
    assert!(plan.blockers().is_empty());
    assert_eq!(plan.references().len(), 2);
    assert_eq!(plan.references()[0].usage, Usage::ConditionAndObserverSlot);
    assert_eq!(plan.references()[1].usage, Usage::EffectWrite);
    let candidate = plan.rename_key("score", "目标分数").unwrap().unwrap();
    let rewrite = &candidate.trees[0];
    let mut expected = f.asset.descriptor().clone();
    for index in [2, 3] {
        expected.nodes[index].parameters[0].value = "目标分数".into();
    }
    assert_eq!(rewrite.candidate.descriptor(), &expected);
    let mut restored = rewrite.candidate.clone();
    for inverse in &rewrite.inverse {
        restored = restored.edited(inverse).unwrap().unwrap();
    }
    assert_eq!(restored, f.asset);
    let mut forwarded = f.asset.clone();
    for forward in &rewrite.forward {
        forwarded = forwarded.edited(forward).unwrap().unwrap();
    }
    assert_eq!(forwarded, rewrite.candidate);
    let mut schema_after = f.schema.clone();
    schema_after.keys[0].key = "目标分数".to_owned();
    assert_eq!(candidate.schema_before, f.schema);
    assert_eq!(candidate.schema_after, schema_after);
    assert_eq!(candidate.schema_source, f.schema_source);
    assert_eq!(candidate.source_preconditions, [f.tree_source.clone()]);
    assert_eq!(candidate.association_preconditions, [f.association.clone()]);
    assert_eq!(candidate.provider_registry, "test-only-provider-lifecycle");
    assert_eq!(candidate.provider_revision, 23);
    assert_eq!(candidate.dependency_revision, 19);
    assert_eq!(candidate.dependency_tree_assets, ids);
}

#[test]
fn another_schema_uuid_with_the_same_business_id_and_key_is_not_rewritten() {
    let mut f = Fixture::new(tree(true));
    f.association.schema.asset_uuid = "a02518f3-2e27-480d-81e1-13ca98d665c4".parse().unwrap();
    let inputs = [f.input()];
    let ids = [f.tree_source.asset_uuid];
    let plan = f.scan(&inputs, Some(f.dependencies(&ids)));
    assert!(plan.blockers().is_empty());
    assert!(plan.references().is_empty());
    assert!(plan
        .rename_key("score", "renamed")
        .unwrap()
        .unwrap()
        .trees
        .is_empty());
    assert_eq!(f.asset, tree(true));
}

#[test]
fn non_condition_decorator_keeps_its_wrapper_and_observer_slot_semantics() {
    let mut descriptor = tree(false).descriptor().clone();
    descriptor.nodes[2].implementation = "cooldown".to_owned();
    descriptor.nodes[2].abort_policy = AiBehaviorAbortPolicy::Self_;
    let f = Fixture::new(Asset::from_descriptor(descriptor).unwrap());
    let inputs = [f.input()];
    let ids = [f.tree_source.asset_uuid];
    let plan = f.scan(&inputs, Some(f.dependencies(&ids)));
    assert!(plan.blockers().is_empty());
    assert_eq!(plan.references()[0].usage, Usage::ObserverSlot);
    let candidate = plan.rename_key("score", "renamed").unwrap().unwrap();
    let guard = &candidate.trees[0].candidate.descriptor().nodes[2];
    assert_eq!(guard.implementation, "cooldown");
    assert_eq!(guard.children, ["leaf"]);
    assert_eq!(guard.abort_policy, AiBehaviorAbortPolicy::Self_);
    assert_eq!(
        guard.parameters[1],
        f.asset.descriptor().nodes[2].parameters[1]
    );
}

#[test]
fn missing_stale_and_unresolved_inputs_are_not_zero_references() {
    let f = Fixture::new(tree(true));
    let ids = [f.tree_source.asset_uuid];
    let inputs = [f.input()];
    let plan = f.scan(&inputs, None);
    assert!(plan.blockers().contains(&Blocker::MissingDependencySet));
    assert!(matches!(
        plan.rename_key("score", "score"),
        Err(Error::Blocked(_))
    ));
    let missing = [TreeInput {
        association: None,
        ..f.input()
    }];
    assert!(f
        .scan(&missing, Some(f.dependencies(&ids)))
        .blockers()
        .contains(&Blocker::MissingAssociation {
            tree: f.tree_source.asset_uuid
        }));
    for stale in [0, 1, 2, 3] {
        let mut association = f.association.clone();
        match stale {
            0 => association.tree.revision += 1,
            1 => association.schema.source_sha256[0] ^= 1,
            2 => association.tree_id.push_str("_stale"),
            _ => association.field_path.clear(),
        }
        let inputs = [TreeInput {
            association: Some(&association),
            ..f.input()
        }];
        assert!(f
            .scan(&inputs, Some(f.dependencies(&ids)))
            .blockers()
            .contains(&Blocker::StaleAssociation {
                tree: f.tree_source.asset_uuid
            }));
    }
    let absent = f.scan(&[], Some(f.dependencies(&ids)));
    assert!(absent.blockers().contains(&Blocker::MissingTree {
        tree: f.tree_source.asset_uuid
    }));
    let duplicates = [f.input(), f.input()];
    assert!(f
        .scan(&duplicates, Some(f.dependencies(&ids)))
        .blockers()
        .contains(&Blocker::DuplicateTree {
            tree: f.tree_source.asset_uuid
        }));
    let unresolved = [AssetUri::parse("res://ai/unresolved").unwrap()];
    let dependencies = Dependencies {
        unresolved: &unresolved,
        ..f.dependencies(&ids)
    };
    assert!(matches!(
        f.scan(&inputs, Some(dependencies))
            .rename_key("score", "new"),
        Err(Error::Blocked(_))
    ));
}

#[test]
fn provider_ownership_and_exact_node_semantics_are_required() {
    for replace_semantics in [false, true] {
        let mut f = Fixture::new(tree(true));
        let mut catalog = BehaviorNodeCatalog::default();
        for descriptor in f.catalog.descriptors() {
            let descriptor = if replace_semantics && descriptor.id() == "set_blackboard" {
                BehaviorNodeDescriptor::new(
                    descriptor.id(),
                    descriptor.display_name(),
                    descriptor.category(),
                    BehaviorNodeSemantics::ScriptTask,
                )
            } else {
                descriptor.clone()
            };
            catalog
                .add_node(
                    if replace_semantics {
                        f.builtin
                    } else {
                        PluginModuleId::from_raw(77)
                    },
                    descriptor,
                )
                .unwrap();
        }
        f.catalog = catalog.freeze();
        let inputs = [f.input()];
        let ids = [f.tree_source.asset_uuid];
        let plan = f.scan(&inputs, Some(f.dependencies(&ids)));
        assert!(plan
            .blockers()
            .iter()
            .any(|blocker| matches!(blocker, Blocker::UnknownProvider { .. })));
        assert!(matches!(
            plan.rename_key("score", "new"),
            Err(Error::Blocked(_))
        ));
    }
}

#[test]
fn scripts_subtrees_and_incomplete_services_remain_opaque_without_literal_keys() {
    for node in [
        Node::new("opaque", Kind::Task, "Script")
            .with_implementation("script_task")
            .with_parameter("callback", "test::read_all"),
        Node::new("opaque", Kind::Subtree, "Subtree")
            .with_implementation("run_subtree")
            .with_parameter("behavior_tree", "other_tree"),
        Node::new("opaque", Kind::Service, "Service")
            .with_implementation("update_blackboard_distance"),
    ] {
        let asset =
            Asset::from_descriptor(Tree::new("opaque_tree", "Opaque", "opaque").with_node(node))
                .unwrap();
        let f = Fixture::new(asset);
        let inputs = [f.input()];
        let ids = [f.tree_source.asset_uuid];
        let plan = f.scan(&inputs, Some(f.dependencies(&ids)));
        assert!(plan.references().is_empty());
        assert!(plan
            .blockers()
            .iter()
            .any(|blocker| matches!(blocker, Blocker::OpaqueNode { .. })));
        assert!(matches!(
            plan.rename_key("score", "new"),
            Err(Error::Blocked(_))
        ));
    }
}

#[test]
fn schema_and_referenced_tree_write_protection_are_checked_before_publication() {
    let f = Fixture::new(tree(true));
    let inputs = [TreeInput {
        writable: false,
        ..f.input()
    }];
    let ids = [f.tree_source.asset_uuid];
    let plan = f.scan(&inputs, Some(f.dependencies(&ids)));
    assert!(
        matches!(plan.rename_key("score", "new"), Err(Error::ReadOnly { asset }) if asset == f.tree_source.asset_uuid)
    );
    let plan = Plan::scan(
        SchemaInput {
            source: &f.schema_source,
            descriptor: &f.schema,
            writable: false,
        },
        &inputs,
        Providers {
            catalog: &f.catalog,
            registry_identity: "test-only-registry",
            revision: 1,
            builtin_provider: f.builtin,
        },
        Some(f.dependencies(&ids)),
    )
    .unwrap();
    assert!(
        matches!(plan.rename_key("score", "new"), Err(Error::ReadOnly { asset }) if asset == f.schema_source.asset_uuid)
    );
    assert_eq!(f.asset, tree(true));
}

#[test]
fn long_chinese_keys_follow_schema_and_effect_specific_runtime_validation() {
    let long_key = "中".repeat(86);
    assert_eq!(long_key.len(), 258);
    for effect in [false, true] {
        let f = Fixture::new(tree(effect));
        let inputs = [f.input()];
        let ids = [f.tree_source.asset_uuid];
        let plan = f.scan(&inputs, Some(f.dependencies(&ids)));
        let renamed = plan.rename_key("score", &long_key);
        if effect {
            assert!(matches!(renamed, Err(Error::InvalidTree { .. })));
        } else {
            assert_eq!(renamed.unwrap().unwrap().schema_after.keys[0].key, long_key);
        }
        assert_eq!(f.asset, tree(effect));
    }
}

#[test]
fn collisions_invalid_schema_unbound_keys_and_known_noop_remain_distinct() {
    let mut f = Fixture::new(tree(false));
    {
        let inputs = [f.input()];
        let ids = [f.tree_source.asset_uuid];
        let plan = f.scan(&inputs, Some(f.dependencies(&ids)));
        assert!(plan.rename_key("score", "score").unwrap().is_none());
        assert!(matches!(
            plan.rename_key("score", "other"),
            Err(Error::DuplicateKey(_))
        ));
        assert!(matches!(
            plan.rename_key("missing", "new"),
            Err(Error::MissingKey(_))
        ));
        assert!(matches!(
            plan.rename_key("score", " "),
            Err(Error::InvalidSchema(_))
        ));
    }
    f.schema.keys[0].value_type = "unsupported".to_owned();
    {
        let inputs = [f.input()];
        let ids = [f.tree_source.asset_uuid];
        assert!(matches!(
            f.scan(&inputs, Some(f.dependencies(&ids)))
                .rename_key("score", "new"),
            Err(Error::InvalidSchema(_))
        ));
    }
    f.schema.keys.remove(0);
    let inputs = [f.input()];
    let ids = [f.tree_source.asset_uuid];
    let plan = f.scan(&inputs, Some(f.dependencies(&ids)));
    assert!(matches!(plan.blockers(), [Blocker::UnboundKey { .. }]));
    assert!(matches!(
        plan.rename_key("other", "new"),
        Err(Error::Blocked(_))
    ));
}
