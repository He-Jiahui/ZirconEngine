use super::*;

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
            .map(|registration| registration.registration.type_path.type_path.as_str())
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
    assert!(
        loaded
            .type_registry()
            .contains_type_path("zircon_runtime::scene::components::Name")
    );
}
