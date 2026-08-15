use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use zircon_runtime_interface::reflect::{ReflectError, ReflectFieldInfo};

use super::*;

#[test]
fn public_property_writes_preserve_complete_vm_component_schema() {
    const TYPE_PATH: &str = "gameplay.Component.PublicPropertyState";
    let registration = vm_component_registration(
        TYPE_PATH,
        "PublicPropertyState",
        ReflectScriptVisibility::Public,
        vec![
            ReflectFieldInfo::new("enabled", "Bool", ReflectEditorHint::Bool),
            ReflectFieldInfo::new("label", "String", ReflectEditorHint::String),
        ],
    );
    let mut world = World::empty();
    world
        .register_vm_type(registration, VmTypeBacking::DynamicComponent)
        .expect("VM property state should register");
    let entity = world.spawn_node(NodeKind::Empty);
    let enabled_path = ComponentPropertyPath::new(TYPE_PATH, vec!["enabled".to_string()])
        .expect("VM property path should build");

    let missing_error = world
        .set_property(entity, &enabled_path, ScenePropertyValue::Bool(true))
        .expect_err("a property write must not create a partial VM component");
    assert!(matches!(
        missing_error,
        SceneError::Reflect(ReflectError::MissingComponent {
            entity: missing_entity,
            ref type_path,
        }) if missing_entity == entity && type_path == TYPE_PATH
    ));
    assert!(world.dynamic_component(entity, TYPE_PATH).is_none());

    world
        .set_dynamic_component(
            entity,
            TYPE_PATH,
            serde_json::json!({ "enabled": false, "label": "ready" }),
        )
        .expect("complete VM component should attach");
    let mismatch = world
        .set_property(entity, &enabled_path, ScenePropertyValue::Integer(1))
        .expect_err("public property writes must enforce the VM field type");
    assert!(matches!(
        mismatch,
        SceneError::Reflect(ReflectError::TypeMismatch {
            ref type_path,
            ref field_name,
            ref expected,
            ref actual,
        }) if type_path == TYPE_PATH
            && field_name == "enabled"
            && expected == "Bool"
            && actual == "JsonNumber"
    ));
    assert_eq!(
        world.dynamic_component(entity, TYPE_PATH),
        Some(&serde_json::json!({ "enabled": false, "label": "ready" }))
    );

    assert!(world
        .set_property(entity, &enabled_path, ScenePropertyValue::Bool(true))
        .expect("a correctly typed public property write should succeed"));
    assert_eq!(
        world.dynamic_component(entity, TYPE_PATH),
        Some(&serde_json::json!({ "enabled": true, "label": "ready" }))
    );
}

#[test]
fn zero_field_vm_components_accept_only_empty_objects() {
    const TYPE_PATH: &str = "gameplay.Component.EmptyState";
    let mut world = World::empty();
    world
        .register_vm_type(
            vm_component_registration(
                TYPE_PATH,
                "EmptyState",
                ReflectScriptVisibility::Public,
                Vec::new(),
            ),
            VmTypeBacking::DynamicComponent,
        )
        .expect("empty VM component schema should register");
    let entity = world.spawn_node(NodeKind::Empty);

    assert!(matches!(
        world
            .set_dynamic_component(entity, TYPE_PATH, serde_json::json!(true))
            .expect_err("zero-field VM components still require an object"),
        SceneError::DynamicComponentNotObject { .. }
    ));
    assert!(matches!(
        world
            .set_dynamic_component(entity, TYPE_PATH, serde_json::json!({ "extra": true }))
            .expect_err("zero-field VM components reject undeclared keys"),
        SceneError::Reflect(ReflectError::UnknownField { .. })
    ));
    assert!(world
        .set_dynamic_component(entity, TYPE_PATH, serde_json::json!({}))
        .expect("the exact empty object should attach"));
}

#[test]
fn vm_entity_and_resource_wrappers_reject_lossy_extra_keys() {
    const TYPE_PATH: &str = "gameplay.Component.Links";
    let mut world = World::empty();
    world
        .register_vm_type(
            vm_component_registration(
                TYPE_PATH,
                "Links",
                ReflectScriptVisibility::Public,
                vec![
                    ReflectFieldInfo::new("target", "Entity", ReflectEditorHint::Entity),
                    ReflectFieldInfo::new("asset", "Resource", ReflectEditorHint::Resource),
                ],
            ),
            VmTypeBacking::DynamicComponent,
        )
        .expect("VM link schema should register");
    let entity = world.spawn_node(NodeKind::Empty);

    assert!(matches!(
        world
            .set_dynamic_component(
                entity,
                TYPE_PATH,
                serde_json::json!({
                    "target": { "entity": 7, "extra": true },
                    "asset": { "resource": "asset://hero" }
                }),
            )
            .expect_err("entity wrappers must not silently discard extra data"),
        SceneError::Reflect(ReflectError::TypeMismatch { .. })
    ));
    assert!(matches!(
        world
            .set_dynamic_component(
                entity,
                TYPE_PATH,
                serde_json::json!({
                    "target": { "entity": 7 },
                    "asset": { "resource": "asset://hero", "extra": true }
                }),
            )
            .expect_err("resource wrappers must not silently discard extra data"),
        SceneError::Reflect(ReflectError::TypeMismatch { .. })
    ));
}

#[test]
fn vm_registration_rejects_malformed_declared_value_type_grammar() {
    for malformed in [
        "List",
        "List<",
        "List<>",
        "List<Scalar",
        "List<Scalar>>",
        "List<Scalar>tail>",
        "Map",
        "Map<>",
        "Map<Scalar>",
        "Map<String,>",
        "Map<String, Scalar, String>",
        "Map<Integer, String>",
    ] {
        let registration = vm_component_registration(
            "gameplay.Component.Malformed",
            "Malformed",
            ReflectScriptVisibility::Public,
            vec![ReflectFieldInfo::new(
                "value",
                malformed,
                ReflectEditorHint::None,
            )],
        );

        assert!(
            matches!(
                World::empty()
                    .register_vm_type(registration, VmTypeBacking::DynamicComponent)
                    .expect_err("malformed VM declared value types must fail at registration"),
                SceneError::Reflect(ReflectError::InvalidRegistration { .. })
            ),
            "malformed type `{malformed}` was accepted"
        );
    }

    World::empty()
        .register_vm_type(
            vm_component_registration(
                "gameplay.Component.Nested",
                "Nested",
                ReflectScriptVisibility::Public,
                vec![ReflectFieldInfo::new(
                    "value",
                    "List<Map<String, List<Scalar>>>",
                    ReflectEditorHint::None,
                )],
            ),
            VmTypeBacking::DynamicComponent,
        )
        .expect("fully nested declared value types should parse exactly");
}

#[test]
fn nested_type_mismatches_report_canonical_declared_grammar() {
    const TYPE_PATH: &str = "gameplay.Component.NestedMismatch";
    let mut world = World::empty();
    world
        .register_vm_type(
            vm_component_registration(
                TYPE_PATH,
                "NestedMismatch",
                ReflectScriptVisibility::Public,
                vec![ReflectFieldInfo::new(
                    "values",
                    "List<Map<String, Scalar>>",
                    ReflectEditorHint::None,
                )],
            ),
            VmTypeBacking::DynamicComponent,
        )
        .expect("nested VM schema should register");
    let entity = world.spawn_node(NodeKind::Empty);

    let error = world
        .set_dynamic_component(entity, TYPE_PATH, serde_json::json!({ "values": [1] }))
        .expect_err("a scalar list item must not satisfy a declared map item");
    assert!(matches!(
        error,
        SceneError::Reflect(ReflectError::TypeMismatch { ref expected, .. })
            if expected == "Map<String, Scalar>"
    ));
}

#[test]
fn vm_registration_revalidates_canonical_dto_text_and_plugin_prefix() {
    let mut wrong_prefix = vm_component_registration(
        "other.Component.Health",
        "Health",
        ReflectScriptVisibility::Public,
        vec![scalar_field("current")],
    );
    wrong_prefix.type_path.plugin_id = Some("gameplay".to_string());
    wrong_prefix.plugin_id = Some("gameplay".to_string());
    assert!(matches!(
        World::empty()
            .register_vm_type(wrong_prefix, VmTypeBacking::DynamicComponent)
            .expect_err("VM type paths must live under the declared plugin prefix"),
        SceneError::Reflect(ReflectError::InvalidRegistration { .. })
    ));

    let mut untrimmed = vm_component_registration(
        "gameplay.Component.Health",
        "Health",
        ReflectScriptVisibility::Public,
        vec![scalar_field("current")],
    );
    untrimmed.display_name = " Health ".to_string();
    assert!(matches!(
        World::empty()
            .register_vm_type(untrimmed, VmTypeBacking::DynamicComponent)
            .expect_err("deserialized VM display names must be canonical"),
        SceneError::Reflect(ReflectError::InvalidRegistration { .. })
    ));
}

#[test]
fn public_non_component_state_types_are_not_projected_as_vm_components() {
    let component = vm_component_registration(
        "gameplay.Component.Health",
        "Health",
        ReflectScriptVisibility::Public,
        vec![scalar_field("current")],
    );
    let mut resource = vm_component_registration(
        "gameplay.Resource.Settings",
        "Settings",
        ReflectScriptVisibility::Public,
        vec![scalar_field("volume")],
    );
    resource.is_component = false;
    resource.is_resource = true;

    let projected = VmReflectionSchema::from_state_schema(&state_schema(vec![component, resource]))
        .expect("public non-component VM state types should be ignored by component projection");

    assert_eq!(projected.registrations().len(), 1);
    assert_eq!(
        projected.registrations()[0].type_path.type_path,
        "gameplay.Component.Health"
    );
}

#[test]
fn candidate_schema_revalidates_retained_live_vm_payloads_before_publish() {
    const TYPE_PATH: &str = "gameplay.Component.Health";
    let catalog = VmReflectionCatalog::default();
    let (_runtime, level) = create_managed_level(&catalog);
    let slot = PluginSlotId::new(31);
    catalog
        .publish_generation(
            slot,
            1,
            "gameplay",
            &state_schema(vec![vm_component_registration(
                TYPE_PATH,
                "Health",
                ReflectScriptVisibility::Public,
                vec![scalar_field("current")],
            )]),
        )
        .expect("initial scalar schema should publish");
    level.with_world_mut(|world| {
        let entity = world.spawn_node(NodeKind::Empty);
        world
            .set_dynamic_component(entity, TYPE_PATH, serde_json::json!({ "current": 5.0 }))
            .expect("live scalar payload should attach");
    });

    let error = catalog
        .publish_generation(
            slot,
            2,
            "gameplay",
            &state_schema(vec![vm_component_registration(
                TYPE_PATH,
                "Health",
                ReflectScriptVisibility::Public,
                vec![ReflectFieldInfo::new(
                    "current",
                    "String",
                    ReflectEditorHint::String,
                )],
            )]),
        )
        .expect_err("a candidate cannot invalidate retained live payloads");

    assert!(matches!(
        error,
        VmReflectionError::Scene(SceneError::Reflect(_))
    ));
    level.with_world(|world| {
        assert_eq!(
            world
                .type_registry()
                .registration(TYPE_PATH)
                .expect("previous registration should remain")
                .type_info
                .fields[0]
                .value_type_path,
            "Scalar"
        );
    });
}

#[test]
fn equal_generation_cannot_replace_a_different_schema() {
    let catalog = VmReflectionCatalog::default();
    let slot = PluginSlotId::new(41);
    catalog
        .publish_generation(
            slot,
            1,
            "gameplay",
            &state_schema(vec![vm_component_registration(
                "gameplay.Component.Health",
                "Health",
                ReflectScriptVisibility::Public,
                vec![scalar_field("current")],
            )]),
        )
        .expect("first generation should publish");

    let error = catalog
        .publish_generation(
            slot,
            1,
            "gameplay",
            &state_schema(vec![vm_component_registration(
                "gameplay.Component.Health",
                "Health",
                ReflectScriptVisibility::Public,
                vec![ReflectFieldInfo::new(
                    "current",
                    "String",
                    ReflectEditorHint::String,
                )],
            )]),
        )
        .expect_err("equal generations must be immutable");
    assert!(
        error.to_string().contains("generation"),
        "equal-generation conflicts must remain a typed generation error: {error}"
    );
}

#[test]
fn committed_catalog_mutations_invalidate_older_registry_snapshots() {
    let catalog = VmReflectionCatalog::default();
    let original = catalog
        .current_snapshot()
        .expect("initial canonical snapshot should build");
    let repeated = catalog
        .current_snapshot()
        .expect("stable canonical snapshot should remain available");
    assert!(original.is_current());
    assert_eq!(original.revision(), 0);
    assert!(std::ptr::eq(original.registry(), repeated.registry()));

    catalog
        .publish_generation(
            PluginSlotId::new(51),
            1,
            "gameplay",
            &state_schema(vec![vm_component_registration(
                "gameplay.Component.Health",
                "Health",
                ReflectScriptVisibility::Public,
                vec![scalar_field("current")],
            )]),
        )
        .expect("catalog mutation should publish");

    assert!(!original.is_current());
    let current = catalog
        .current_snapshot()
        .expect("updated canonical snapshot should build");
    assert!(current.is_current());
    assert_eq!(current.revision(), 1);
}

#[test]
fn trusted_package_owner_rejects_self_consistent_foreign_namespaces() {
    let catalog = VmReflectionCatalog::default();
    let error = catalog
        .publish_generation(
            PluginSlotId::new(61),
            1,
            "trusted_package",
            &state_schema(vec![vm_component_registration(
                "gameplay.Component.Health",
                "Health",
                ReflectScriptVisibility::Public,
                vec![scalar_field("current")],
            )]),
        )
        .expect_err("a schema cannot self-report another package namespace");

    assert!(matches!(
        error,
        VmReflectionError::PackageOwnerMismatch {
            ref expected_owner,
            ref declared_owner,
            ..
        } if expected_owner == "trusted_package" && declared_owner == "gameplay"
    ));
    assert_eq!(catalog.revision(), 0);
}

#[test]
fn same_revision_candidates_have_distinct_commit_identity() {
    let catalog = VmReflectionCatalog::default();
    let prepared_a = catalog
        .prepare_optional_generation(
            PluginSlotId::new(71),
            1,
            "gameplay",
            Some(&state_schema(vec![vm_component_registration(
                "gameplay.Component.Health",
                "Health",
                ReflectScriptVisibility::Public,
                vec![scalar_field("current")],
            )])),
        )
        .expect("candidate A should prepare");
    let snapshot_a = prepared_a.snapshot().clone();
    let prepared_b = catalog
        .prepare_optional_generation(
            PluginSlotId::new(72),
            1,
            "gameplay",
            Some(&state_schema(vec![vm_component_registration(
                "gameplay.Component.Mana",
                "Mana",
                ReflectScriptVisibility::Public,
                vec![scalar_field("current")],
            )])),
        )
        .expect("candidate B should prepare from the same base");
    let snapshot_b = prepared_b.snapshot().clone();

    assert_eq!(snapshot_a.revision(), snapshot_b.revision());
    assert!(snapshot_a.can_resolve_names());
    assert!(!snapshot_a.is_current());
    catalog
        .commit_prepared(prepared_b)
        .expect("candidate B should commit");
    let committed = catalog
        .current_snapshot()
        .expect("committed candidate snapshot should remain available");
    assert!(snapshot_b.is_current());
    assert!(std::ptr::eq(snapshot_b.registry(), committed.registry()));
    assert!(!snapshot_a.is_current());
    assert!(!snapshot_a.can_resolve_names());
    assert!(matches!(
        catalog
            .commit_prepared(prepared_a)
            .expect_err("candidate A must be stale after B commits"),
        VmReflectionError::PreparedGenerationStale { .. }
    ));
}

#[test]
fn prepared_generation_cannot_cross_catalog_boundaries() {
    let source = VmReflectionCatalog::default();
    let destination = VmReflectionCatalog::default();
    let prepared = source
        .prepare_optional_generation(
            PluginSlotId::new(81),
            1,
            "gameplay",
            Some(&state_schema(vec![vm_component_registration(
                "gameplay.Component.Health",
                "Health",
                ReflectScriptVisibility::Public,
                vec![scalar_field("current")],
            )])),
        )
        .expect("source catalog candidate should prepare");

    let error = destination
        .commit_prepared(prepared)
        .expect_err("a catalog must reject another catalog's prepared capability");

    assert!(matches!(
        error,
        VmReflectionError::ForeignPreparedGeneration
    ));
    assert_eq!(destination.revision(), 0);
    let snapshot = destination
        .current_snapshot()
        .expect("destination snapshot should remain available");
    assert!(snapshot.is_current());
    assert_eq!(snapshot.revision(), 0);
    assert!(!snapshot
        .registry()
        .contains_type_path("gameplay.Component.Health"));
}
