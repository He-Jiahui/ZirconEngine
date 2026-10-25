use zircon_runtime::core::framework::ai::{
    AiBlackboardKeyDescriptor as Key, AiBlackboardValueType as ValueType,
};

use crate::{
    AiBlackboardMenuAdapter as Menu, AiBlackboardMenuCandidate as Candidate,
    AiBlackboardMenuCommand as Command, AiBlackboardMenuDisabledReason as Disabled,
    AiBlackboardMenuError as Error, AiBlackboardMenuRequest as Request,
    AiBlackboardProviderSnapshot as Providers, AiBlackboardReferenceKnowledge as Knowledge,
    AiBlackboardReferencePlan as Plan, AiBlackboardSchemaInput as SchemaInput,
};

use super::{tree, Fixture};

fn readonly_plan<'a>(f: &'a Fixture, inputs: &'a [crate::AiBlackboardTreeInput<'a>]) -> Plan<'a> {
    Plan::scan(
        SchemaInput {
            source: &f.schema_source,
            descriptor: &f.schema,
            writable: false,
        },
        inputs,
        Providers {
            catalog: &f.catalog,
            registry_identity: "unit-only-provider",
            revision: 9,
            builtin_provider: f.builtin,
        },
        None,
    )
    .unwrap()
}

#[test]
fn create_key_preserves_order_alias_required_source_and_exact_inverse() {
    let f = Fixture::new(tree(true));
    let inputs = [f.input()];
    let plan = f.scan(&inputs, None);
    let menu = Menu::new(&plan, &[]);
    // Schema-only creation does not claim the missing reference set is complete.
    let key = Key {
        key: "目标位置".to_owned(),
        value_type: "vector3".to_owned(),
        required: true,
    };
    let Some(Candidate::CreateKey(candidate)) = menu
        .prepare(Request::CreateKey {
            index: 1,
            key: &key,
        })
        .unwrap()
    else {
        panic!("expected schema value");
    };
    assert_eq!(candidate.source(), &f.schema_source);
    assert_eq!(candidate.before(), &f.schema);
    assert_eq!(candidate.after().keys[1], key);
    assert_eq!(candidate.after().keys[0], f.schema.keys[0]);
    assert_eq!(candidate.after().keys[2], f.schema.keys[1]);
    assert_eq!(candidate.forward_from(&f.schema), Some(candidate.after()));
    assert_eq!(candidate.inverse_from(candidate.after()), Some(&f.schema));
    assert_eq!(
        menu.key_rows("score").next().unwrap().references,
        Knowledge::Unknown(plan.blockers())
    );
}

#[test]
fn six_runtime_types_and_long_schema_key_use_real_validation_without_effect_limit() {
    let f = Fixture::new(tree(true));
    let inputs = [f.input()];
    let plan = f.scan(&inputs, None);
    let menu = Menu::new(&plan, &[]);
    for kind in [
        ValueType::Bool,
        ValueType::Integer,
        ValueType::Scalar,
        ValueType::String,
        ValueType::Vec3,
        ValueType::Entity,
    ] {
        let key = Key {
            key: "中文长键".repeat(80),
            value_type: kind.as_str().to_owned(),
            required: false,
        };
        assert!(key.key.len() > 256);
        let Some(Candidate::CreateKey(candidate)) = menu
            .prepare(Request::CreateKey {
                index: 2,
                key: &key,
            })
            .unwrap()
        else {
            panic!("expected schema value");
        };
        assert_eq!(candidate.after().keys[2], key);
        assert_eq!(candidate.after().keys[2].expected_value_type(), Some(kind));
    }
    assert_eq!(f.schema.keys.len(), 2);
}

#[test]
fn invalid_duplicate_empty_unknown_type_and_index_never_yield_a_schema_candidate() {
    let f = Fixture::new(tree(true));
    let inputs = [f.input()];
    let plan = f.scan(&inputs, None);
    let menu = Menu::new(&plan, &[]);
    for key in [
        Key {
            key: "score".to_owned(),
            value_type: "string".to_owned(),
            required: false,
        },
        Key {
            key: "".to_owned(),
            value_type: "bool".to_owned(),
            required: false,
        },
        Key {
            key: "new_key".to_owned(),
            value_type: "unknown".to_owned(),
            required: false,
        },
    ] {
        assert!(matches!(
            menu.prepare(Request::CreateKey {
                index: 1,
                key: &key
            }),
            Err(Error::InvalidSchema(_))
        ));
    }
    let key = Key {
        key: "new_key".to_owned(),
        value_type: "bool".to_owned(),
        required: false,
    };
    assert!(matches!(
        menu.prepare(Request::CreateKey {
            index: 99,
            key: &key
        }),
        Err(Error::KeyIndex {
            index: 99,
            count: 2
        })
    ));
    let readonly = readonly_plan(&f, &inputs);
    assert!(matches!(
        Menu::new(&readonly, &[]).prepare(Request::CreateKey {
            index: 0,
            key: &key
        }),
        Err(Error::Disabled(Disabled::ReadOnly))
    ));
    assert_eq!(f.schema.keys.len(), 2);

    let mut invalid = Fixture::new(tree(true));
    invalid.schema.keys[0].value_type = "unknown".to_owned();
    let invalid_inputs = [invalid.input()];
    let invalid_plan = invalid.scan(&invalid_inputs, None);
    assert!(matches!(
        Menu::new(&invalid_plan, &[]).prepare(Request::CreateKey {
            index: 0,
            key: &key
        }),
        Err(Error::InvalidSchema(_))
    ));
}

#[test]
fn rename_reuses_whole_semantic_plan_and_preserves_noop_and_inverse() {
    let f = Fixture::new(tree(true));
    let inputs = [f.input()];
    let ids = [f.tree_source.asset_uuid];
    let plan = f.scan(&inputs, Some(f.dependencies(&ids)));
    let menu = Menu::new(&plan, &["score"]);
    assert!(menu
        .prepare(Request::RenameKey { new_name: "score" })
        .unwrap()
        .is_none());
    let Some(Candidate::RenameKey(candidate)) = menu
        .prepare(Request::RenameKey {
            new_name: "目标分数",
        })
        .unwrap()
    else {
        panic!("expected original rename plan");
    };
    assert_eq!(candidate.schema_before, f.schema);
    assert_eq!(candidate.schema_after.keys[0].key, "目标分数");
    assert_eq!(candidate.source_preconditions, [f.tree_source.clone()]);
    assert_eq!(candidate.association_preconditions, [f.association.clone()]);
    assert_eq!(candidate.trees.len(), 1);
    let mut restored = candidate.trees[0].candidate.clone();
    for inverse in &candidate.trees[0].inverse {
        restored = restored.edited(inverse).unwrap().unwrap();
    }
    assert_eq!(restored, f.asset);
    assert_eq!(
        candidate.trees[0].candidate.descriptor().nodes[0].display_name,
        "score"
    );
}

#[test]
fn affected_readonly_tree_blocks_rename_without_blocking_an_unreferenced_key() {
    let f = Fixture::new(tree(true));
    let mut input = f.input();
    input.writable = false;
    let inputs = [input];
    let ids = [f.tree_source.asset_uuid];
    let plan = f.scan(&inputs, Some(f.dependencies(&ids)));
    assert_eq!(
        Menu::new(&plan, &["score"]).eligibility(Command::RenameKey),
        Err(Disabled::ReferencedTreeReadOnly)
    );
    let Some(Candidate::RenameKey(candidate)) = Menu::new(&plan, &["other"])
        .prepare(Request::RenameKey {
            new_name: "unused_other",
        })
        .unwrap()
    else {
        panic!("expected schema-only rename");
    };
    assert!(candidate.trees.is_empty());
    assert_eq!(candidate.source_preconditions, [f.tree_source.clone()]);
}

#[test]
fn unknown_references_never_become_zero_and_key_search_borrows_original_rows() {
    let mut f = Fixture::new(tree(true));
    f.schema.keys.push(Key {
        key: "中文长名称".to_owned(),
        value_type: "string".to_owned(),
        required: false,
    });
    let inputs = [f.input()];
    let plan = f.scan(&inputs, None);
    let menu = Menu::new(&plan, &["score"]);
    assert!(matches!(
        menu.key_rows(" SCORE  STR ").next().unwrap().references,
        Knowledge::Unknown(_)
    ));
    assert!(menu.key_rows("score").next().unwrap().selected);
    assert!(std::ptr::eq(
        menu.key_rows("score").next().unwrap().key,
        &f.schema.keys[0]
    ));
    assert_eq!(menu.key_rows("中文 STRING").count(), 1);
    assert_eq!(menu.key_rows("no matching key").count(), 0);
    assert_eq!(
        menu.rows("RENAME key").next().unwrap().command,
        Command::RenameKey
    );
    assert!(matches!(
        menu.prepare(Request::RenameKey {
            new_name: "new_score"
        }),
        Err(Error::Disabled(Disabled::UnknownReferences))
    ));
}

#[test]
fn known_tree_scope_zero_does_not_authorize_type_delete_or_runtime_value_changes() {
    let f = Fixture::new(tree(true));
    let inputs = [f.input()];
    let ids = [f.tree_source.asset_uuid];
    let plan = f.scan(&inputs, Some(f.dependencies(&ids)));
    for (selected, count, reason) in [
        ("score", 2, Disabled::KeyReferenced),
        ("other", 0, Disabled::SchemaEvolutionUnsupported),
    ] {
        let selection = [selected];
        let menu = Menu::new(&plan, &selection);
        assert_eq!(
            menu.key_rows(selected).next().unwrap().references,
            Knowledge::InSuppliedScope(count)
        );
        for request in [
            Request::ChangeType {
                value_type: ValueType::Integer,
            },
            Request::DeleteKey,
        ] {
            assert!(
                matches!(menu.prepare(request), Err(Error::Disabled(actual)) if actual == reason)
            );
        }
        assert!(matches!(
            menu.prepare(Request::EditRuntimeValue),
            Err(Error::Disabled(Disabled::RuntimeValuesReadOnly))
        ));
    }
}

#[test]
fn absent_multiple_stale_and_readonly_selection_never_choose_another_key() {
    let f = Fixture::new(tree(true));
    let inputs = [f.input()];
    let ids = [f.tree_source.asset_uuid];
    let plan = f.scan(&inputs, Some(f.dependencies(&ids)));
    for (selection, reason) in [
        (&[][..], Disabled::NoSelection),
        (&["score", "other"][..], Disabled::MultipleSelection),
        (&["old_score"][..], Disabled::MissingKey),
    ] {
        assert!(
            matches!(Menu::new(&plan, selection).prepare(Request::RenameKey { new_name: "new_score" }), Err(Error::Disabled(actual)) if actual == reason)
        );
    }
    let readonly = readonly_plan(&f, &inputs);
    assert_eq!(
        Menu::new(&readonly, &["score"]).eligibility(Command::RenameKey),
        Err(Disabled::ReadOnly)
    );
}

#[test]
fn schema_candidate_checks_intervening_values_without_claiming_revision_or_aba_safety() {
    let f = Fixture::new(tree(true));
    let inputs = [f.input()];
    let plan = f.scan(&inputs, None);
    let key = Key {
        key: "new_key".to_owned(),
        value_type: "entity_id".to_owned(),
        required: false,
    };
    let Some(Candidate::CreateKey(candidate)) = Menu::new(&plan, &[])
        .prepare(Request::CreateKey {
            index: 2,
            key: &key,
        })
        .unwrap()
    else {
        panic!("expected schema value");
    };
    let mut intervening = f.schema.clone();
    intervening.keys[0].required = !intervening.keys[0].required;
    assert!(candidate.forward_from(&intervening).is_none());
    let mut changed_after = candidate.after().clone();
    changed_after.keys[2].value_type = "entity".to_owned();
    assert!(candidate.inverse_from(&changed_after).is_none());
    // Equal values after ABA still match: original host lease/revision CAS is mandatory.
    assert!(candidate.forward_from(&f.schema.clone()).is_some());
    assert_eq!(candidate.inverse_from(candidate.after()), Some(&f.schema));
}
