use super::*;

fn test_field(
    name: impl Into<String>,
    value_type_path: impl Into<String>,
    editor_hint: ReflectEditorHint,
) -> ReflectFieldInfo {
    let name = name.into();
    ReflectFieldInfo::from_stable_keys(
        "tests.ecs-reflect.field-probe",
        &name,
        name.clone(),
        value_type_path,
        editor_hint,
    )
}

#[test]
fn empty_world_builds_runtime_only_type_registry() {
    let world = World::empty();

    assert!(!world.type_registry().is_empty());
}

#[test]
fn type_registry_rejects_duplicate_full_type_paths() {
    let mut registry = TypeRegistry::default();
    let registration = metadata_registration("plugin_a::Short", "Short");

    registry
        .register(RuntimeTypeRegistration::metadata(registration.clone()))
        .expect("first registration should be accepted");
    assert!(registry.contains_type_path("plugin_a::Short"));
    let error = registry
        .register(RuntimeTypeRegistration::metadata(registration))
        .expect_err("duplicate full type path should be rejected");

    assert_eq!(
        error,
        ReflectError::DuplicateTypePath {
            type_path: "plugin_a::Short".to_string(),
        }
    );
}

#[test]
fn type_registry_rejects_invalid_field_metadata_atomically() {
    let mut over_depth = ReflectedValue::Null;
    for _ in 0..128 {
        over_depth = ReflectedValue::List(vec![over_depth]);
    }
    let cases = vec![
        test_field("malformed", "List<", ReflectEditorHint::None),
        test_field("bad key", "String", ReflectEditorHint::String),
        test_field("duplicate_enum", "Enum", ReflectEditorHint::Enum).with_enum_options(vec![
            ReflectEnumOption::new("active", "Active"),
            ReflectEnumOption::new("active", "Active duplicate"),
        ]),
        test_field("unknown_default", "Enum", ReflectEditorHint::Enum)
            .with_enum_options(vec![ReflectEnumOption::new("active", "Active")])
            .with_default_value(ReflectedValue::Enum("missing".to_string())),
        test_field("text_range", "String", ReflectEditorHint::String).with_numeric_range(
            ReflectNumericRange::new(Some(0.0), Some(1.0), None, None)
                .expect("test range should be valid"),
        ),
        test_field("wrong_default", "Bool", ReflectEditorHint::Bool)
            .with_default_value(ReflectedValue::String("true".to_string())),
        test_field("non_finite", "Scalar", ReflectEditorHint::Scalar)
            .with_default_value(ReflectedValue::Scalar(f32::NAN)),
        test_field("over_depth", "List", ReflectEditorHint::None).with_default_value(over_depth),
    ];

    for field in cases {
        let field_name = field.name.clone();
        let registration = ReflectTypeRegistration::new(
            ReflectTypePath::new("plugin_a::FieldProbe", "FieldProbe")
                .expect("test type path should be valid"),
            "Field Probe",
            ReflectTypeInfo::struct_with_fields(vec![field]),
            ReflectSerializationStrategy::Value,
        );
        let mut registry = TypeRegistry::default();
        let error = registry
            .register(RuntimeTypeRegistration::metadata(registration))
            .expect_err("invalid field metadata must reject the entire registration");

        assert!(matches!(
            error,
            ReflectError::InvalidFieldRegistration {
                ref type_path,
                field_name: ref rejected_field,
                ..
            } if type_path == "plugin_a::FieldProbe" && rejected_field == &field_name
        ));
        assert!(registry.is_empty());
    }
}

#[test]
fn type_registry_rejects_field_id_and_alias_collisions_atomically() {
    let first = test_field("first", "Scalar", ReflectEditorHint::Scalar);
    let mut duplicate_id = test_field("second", "Scalar", ReflectEditorHint::Scalar);
    duplicate_id.id = first.id;
    let id_registration = ReflectTypeRegistration::new(
        ReflectTypePath::new("plugin_a::DuplicateId", "DuplicateId").unwrap(),
        "Duplicate Id",
        ReflectTypeInfo::struct_with_fields(vec![first, duplicate_id]),
        ReflectSerializationStrategy::Value,
    );
    let mut registry = TypeRegistry::default();
    let error = registry
        .register(RuntimeTypeRegistration::metadata(id_registration))
        .expect_err("duplicate stable field IDs must reject the complete registration");
    assert!(matches!(
        error,
        ReflectError::InvalidFieldRegistration { ref reason, .. }
            if reason.contains("duplicate reflected field ID")
    ));
    assert!(registry.is_empty());

    let alias_registration = ReflectTypeRegistration::new(
        ReflectTypePath::new("plugin_a::DuplicateAlias", "DuplicateAlias").unwrap(),
        "Duplicate Alias",
        ReflectTypeInfo::struct_with_fields(vec![
            test_field("first", "Scalar", ReflectEditorHint::Scalar)
                .with_aliases(vec!["second".to_string()]),
            test_field("second", "Scalar", ReflectEditorHint::Scalar),
        ]),
        ReflectSerializationStrategy::Value,
    );
    let error = registry
        .register(RuntimeTypeRegistration::metadata(alias_registration))
        .expect_err("aliases must not collide with current field names");
    assert!(matches!(
        error,
        ReflectError::InvalidFieldRegistration { ref reason, .. }
            if reason.contains("duplicate reflected field name or alias")
    ));
    assert!(registry.is_empty());
}

#[test]
fn type_registry_rejects_cross_type_field_id_collisions_atomically() {
    let shared_id = ReflectFieldId::from_stable_keys("tests.shared-owner", "value");
    let registration = |type_path: &str, short_type_path: &str| {
        ReflectTypeRegistration::new(
            ReflectTypePath::new(type_path, short_type_path).unwrap(),
            short_type_path,
            ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
                shared_id,
                "value",
                "Scalar",
                ReflectEditorHint::Scalar,
            )]),
            ReflectSerializationStrategy::Value,
        )
    };
    let mut registry = TypeRegistry::default();
    registry
        .register(RuntimeTypeRegistration::metadata(registration(
            "plugin_a::First",
            "First",
        )))
        .unwrap();
    let generation = registry.schema_catalog_generation();

    let error = registry
        .register(RuntimeTypeRegistration::metadata(registration(
            "plugin_a::Second",
            "Second",
        )))
        .expect_err("one stable field ID must have one catalog owner");

    assert!(matches!(
        error,
        ReflectError::InvalidFieldRegistration { ref reason, .. }
            if reason.contains("is already owned by")
    ));
    assert!(!registry.contains_type_path("plugin_a::Second"));
    assert_eq!(registry.schema_catalog_generation(), generation);
}

#[test]
fn type_registry_resolves_stable_field_ids_across_index_storage_boundary() {
    for field_count in [0_usize, 1, 16, 512, 513, 4_096] {
        let type_path = format!("plugin_a::FieldIndex{field_count}");
        let short_type_path = format!("FieldIndex{field_count}");
        let fields = (0..field_count)
            .map(|index| {
                test_field(
                    format!("field_{index}"),
                    "Scalar",
                    ReflectEditorHint::Scalar,
                )
            })
            .collect::<Vec<_>>();
        let field_ids = fields.iter().map(|field| field.id).collect::<Vec<_>>();
        let registration = ReflectTypeRegistration::new(
            ReflectTypePath::new(&type_path, short_type_path).unwrap(),
            format!("Field Index {field_count}"),
            ReflectTypeInfo::struct_with_fields(fields),
            ReflectSerializationStrategy::Value,
        );
        let mut registry = TypeRegistry::default();
        registry
            .register(RuntimeTypeRegistration::metadata(registration))
            .expect("bounded unique fields should build an ID-to-slot index");

        let mut probe_slots = vec![0_usize, field_count / 2, field_count.saturating_sub(1)];
        probe_slots.sort_unstable();
        probe_slots.dedup();
        for field_slot in probe_slots.into_iter().filter(|slot| *slot < field_count) {
            assert_eq!(
                registry
                    .resolve_field_slot_by_id(&type_path, field_ids[field_slot])
                    .unwrap(),
                field_slot as u32
            );
        }

        let unknown_id = ReflectFieldId::from_stable_keys(&type_path, "unknown");
        let error = registry
            .resolve_field_slot_by_id(&type_path, unknown_id)
            .expect_err("an unregistered stable field ID must not resolve");
        assert_eq!(
            error,
            ReflectError::UnknownField {
                type_path: type_path.clone(),
                field_name: unknown_id.to_string(),
            }
        );
    }
}

#[test]
fn type_registry_rejects_over_budget_field_catalog_atomically() {
    let fields = (0..4_097)
        .map(|index| {
            test_field(
                format!("field_{index}"),
                "Scalar",
                ReflectEditorHint::Scalar,
            )
        })
        .collect();
    let registration = ReflectTypeRegistration::new(
        ReflectTypePath::new("plugin_a::Oversized", "Oversized")
            .expect("test type path should be valid"),
        "Oversized",
        ReflectTypeInfo::struct_with_fields(fields),
        ReflectSerializationStrategy::Value,
    );
    let mut registry = TypeRegistry::default();

    let error = registry
        .register(RuntimeTypeRegistration::metadata(registration))
        .expect_err("over-budget field catalogs must reject before publication");

    assert!(matches!(
        error,
        ReflectError::InvalidRegistration { ref reason, .. }
            if reason.contains("must not declare more than 4096 fields")
    ));
    assert!(registry.is_empty());
}

#[test]
fn type_registry_short_path_lookup_reports_ambiguity() {
    let mut registry = TypeRegistry::default();

    registry
        .register(RuntimeTypeRegistration::metadata(metadata_registration(
            "plugin_a::Short",
            "Short",
        )))
        .expect("first short path should be accepted");
    assert!(registry.contains("Short"));
    assert!(!registry.contains_type_path("Short"));
    registry
        .register(RuntimeTypeRegistration::metadata(metadata_registration(
            "plugin_b::Short",
            "Short",
        )))
        .expect("second full type path should be accepted");

    let caller_owned_type_path = String::from("plugin_a::Short");
    let resolved = registry.resolve(caller_owned_type_path.as_str()).unwrap();
    drop(caller_owned_type_path);

    assert_eq!(resolved, "plugin_a::Short");
    assert_eq!(
        registry.resolve("plugin_b::Short").unwrap(),
        "plugin_b::Short"
    );
    assert!(registry.contains("plugin_a::Short"));
    assert!(!registry.contains("Short"));
    assert!(!registry.contains_type_path("Short"));
    assert_eq!(
        registry
            .iter()
            .map(|registration| registration.registration.type_path.type_path())
            .collect::<Vec<_>>(),
        vec!["plugin_a::Short", "plugin_b::Short"]
    );
    assert_eq!(
        registry
            .resolve("Short")
            .expect_err("short path is ambiguous"),
        ReflectError::AmbiguousShortTypePath {
            short_type_path: "Short".to_string(),
        }
    );

    registry.clear();
    assert!(registry.is_empty());
    assert_eq!(
        registry.resolve("Short").expect_err("registry was cleared"),
        ReflectError::UnknownType {
            type_path: "Short".to_string(),
        }
    );
}

#[test]
fn runtime_type_registration_compares_adapter_presence_not_identity() {
    let metadata = metadata_registration("plugin_a::RuntimeOnly", "RuntimeOnly");
    let metadata_only = RuntimeTypeRegistration::metadata(metadata.clone());
    let with_component = RuntimeTypeRegistration {
        registration: metadata.clone(),
        component: Some(dummy_component_adapter()),
        resource: None,
    };
    let with_resource = RuntimeTypeRegistration {
        registration: metadata,
        component: Some(dummy_component_adapter()),
        resource: Some(dummy_resource_adapter()),
    };

    assert_ne!(metadata_only, with_component);
    assert_ne!(with_component, with_resource);
    assert!(format!("{with_resource:?}").contains("has_component_adapter: true"));
    assert!(format!("{with_resource:?}").contains("has_resource_adapter: true"));
}

#[test]
fn world_serialization_skips_reflection_registry_and_rebuilds_it_on_load() {
    let mut world = World::empty();
    let builtin_count = world.type_registry().iter().count();
    world
        .type_registry_mut_for_tests()
        .register(RuntimeTypeRegistration::metadata(metadata_registration(
            "plugin_a::RuntimeOnly",
            "RuntimeOnly",
        )))
        .expect("test registration should be accepted");

    let json = serde_json::to_string(&world).expect("world should serialize");

    assert!(!json.contains("type_registry"));
    assert!(!json.contains("registrations"));
    assert!(!json.contains("RuntimeOnly"));

    let loaded: World = serde_json::from_str(&json).expect("world should deserialize");

    assert_eq!(loaded.type_registry().iter().count(), builtin_count);
    assert!(loaded
        .type_registry()
        .contains_type_path("zircon_runtime::scene::components::Name"));
}
