use super::{ReflectSchemaCatalog, ReflectSchemaCatalogEntry};
use crate::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldId, ReflectFieldInfo,
    ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
};

fn registration(
    type_path: &str,
    short_type_path: &str,
    field_id: ReflectFieldId,
    field_name: &str,
    aliases: &[&str],
) -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new(type_path, short_type_path).unwrap(),
        short_type_path,
        ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
            field_id,
            field_name,
            "f32",
            ReflectEditorHint::Scalar,
        )
        .with_aliases(aliases.iter().map(|alias| (*alias).to_string()).collect())]),
        ReflectSerializationStrategy::Value,
    )
}

fn entry(
    type_path: &str,
    short_type_path: &str,
    field_key: &str,
    field_name: &str,
) -> ReflectSchemaCatalogEntry {
    ReflectSchemaCatalogEntry::new(registration(
        type_path,
        short_type_path,
        ReflectFieldId::from_stable_keys(type_path, field_key),
        field_name,
        &[],
    ))
}

#[test]
fn catalog_fingerprint_is_independent_of_input_and_metadata_set_order() {
    let alpha_id = ReflectFieldId::from_stable_keys("game.Alpha", "value");
    let alpha = ReflectSchemaCatalogEntry::new(registration(
        "game.Alpha",
        "Shared",
        alpha_id,
        "value",
        &["old_value", "legacy_value"],
    ))
    .with_dependencies(vec!["game.Dependency".to_string()]);
    let dependency = entry("game.Dependency", "Dependency", "enabled", "enabled");

    let reversed_metadata = ReflectSchemaCatalogEntry::new(registration(
        "game.Alpha",
        "Shared",
        alpha_id,
        "value",
        &["legacy_value", "old_value"],
    ))
    .with_dependencies(vec!["game.Dependency".to_string()]);

    let first = ReflectSchemaCatalog::try_new(vec![alpha, dependency.clone()]).unwrap();
    let second = ReflectSchemaCatalog::try_new(vec![dependency, reversed_metadata]).unwrap();

    assert_eq!(first.fingerprint(), second.fingerprint());
    assert_eq!(first.snapshot(), second.snapshot());
}

#[test]
fn catalog_preserves_ambiguous_short_paths_without_selecting_a_winner() {
    let catalog = ReflectSchemaCatalog::try_new(vec![
        entry("game.first.Shared", "Shared", "value", "value"),
        entry("game.second.Shared", "Shared", "value", "value"),
    ])
    .unwrap();

    assert!(matches!(
        catalog.resolve_type_path("Shared"),
        Err(ReflectError::AmbiguousShortTypePath { .. })
    ));
    assert_eq!(
        catalog.resolve_type_path("game.first.Shared").unwrap(),
        "game.first.Shared"
    );
    assert_eq!(
        catalog.ambiguous_short_type_paths().collect::<Vec<_>>(),
        vec!["Shared"]
    );
}

#[test]
fn catalog_rejects_duplicate_full_paths_and_global_field_ids() {
    let duplicate_path = ReflectSchemaCatalog::try_new(vec![
        entry("game.Actor", "Actor", "first", "first"),
        entry("game.Actor", "Actor", "second", "second"),
    ]);
    assert!(matches!(
        duplicate_path,
        Err(ReflectError::DuplicateTypePath { .. })
    ));

    let shared_id = ReflectFieldId::from_stable_keys("stable.Owner", "value");
    let duplicate_field_id = ReflectSchemaCatalog::try_new(vec![
        ReflectSchemaCatalogEntry::new(registration(
            "game.First",
            "First",
            shared_id,
            "value",
            &[],
        )),
        ReflectSchemaCatalogEntry::new(registration(
            "game.Second",
            "Second",
            shared_id,
            "value",
            &[],
        )),
    ]);
    assert!(matches!(
        duplicate_field_id,
        Err(ReflectError::InvalidFieldRegistration { .. })
    ));
}

#[test]
fn catalog_resolves_stable_slots_and_scoped_legacy_aliases() {
    let field_id = ReflectFieldId::from_stable_keys("game.Transform", "translation");
    let catalog =
        ReflectSchemaCatalog::try_new(vec![ReflectSchemaCatalogEntry::new(registration(
            "game.Transform",
            "Transform",
            field_id,
            "translation",
            &["position"],
        ))])
        .unwrap();

    assert_eq!(
        catalog
            .field_slot_by_id("game.Transform", field_id)
            .unwrap(),
        0
    );
    assert_eq!(
        catalog
            .resolve_legacy_field_id("game.Transform", "translation")
            .unwrap(),
        field_id
    );
    assert_eq!(
        catalog
            .resolve_legacy_field_id("game.Transform", "position")
            .unwrap(),
        field_id
    );
    assert!(matches!(
        catalog.resolve_legacy_field_id("game.Transform", "missing"),
        Err(ReflectError::UnknownField { .. })
    ));
}

#[test]
fn catalog_requires_a_complete_acyclic_dependency_closure() {
    let missing =
        ReflectSchemaCatalog::try_new(vec![entry("game.Actor", "Actor", "value", "value")
            .with_dependencies(vec!["game.Missing".to_string()])]);
    assert!(matches!(
        missing,
        Err(ReflectError::InvalidRegistration { .. })
    ));

    let cycle = ReflectSchemaCatalog::try_new(vec![
        entry("game.First", "First", "value", "value")
            .with_dependencies(vec!["game.Second".to_string()]),
        entry("game.Second", "Second", "value", "value")
            .with_dependencies(vec!["game.First".to_string()]),
    ]);
    assert!(matches!(
        cycle,
        Err(ReflectError::InvalidRegistration { .. })
    ));
}

#[test]
fn catalog_dependency_order_places_dependencies_before_consumers() {
    let catalog = ReflectSchemaCatalog::try_new(vec![
        entry("game.Leaf", "Leaf", "value", "value"),
        entry("game.Middle", "Middle", "value", "value")
            .with_dependencies(vec!["game.Leaf".to_string()]),
        entry("game.Root", "Root", "value", "value")
            .with_dependencies(vec!["game.Middle".to_string()]),
    ])
    .unwrap();

    assert_eq!(
        catalog.dependency_order().collect::<Vec<_>>(),
        vec!["game.Leaf", "game.Middle", "game.Root"]
    );
}

#[test]
fn catalog_rejects_tampered_or_unknown_version_snapshots() {
    let catalog =
        ReflectSchemaCatalog::try_new(vec![entry("game.Actor", "Actor", "value", "value")])
            .unwrap();
    let mut tampered = catalog.snapshot();
    tampered.entries[0].registration.display_name = "Tampered".to_string();
    assert!(matches!(
        ReflectSchemaCatalog::try_from_snapshot(tampered),
        Err(ReflectError::InvalidRegistration { .. })
    ));

    let mut unknown_version = catalog.snapshot();
    unknown_version.algorithm_version += 1;
    assert!(matches!(
        ReflectSchemaCatalog::try_from_snapshot(unknown_version),
        Err(ReflectError::InvalidRegistration { .. })
    ));
}

#[test]
fn incremental_catalog_matches_batch_publication() {
    let leaf = entry("game.Leaf", "Leaf", "value", "value");
    let root = entry("game.Root", "Root", "value", "value")
        .with_dependencies(vec!["game.Leaf".to_string()]);
    let batch = ReflectSchemaCatalog::try_new(vec![root.clone(), leaf.clone()]).unwrap();

    let mut incremental = ReflectSchemaCatalog::default();
    incremental.try_insert(leaf).unwrap();
    incremental.try_insert(root).unwrap();

    assert_eq!(incremental.snapshot(), batch.snapshot());
}

#[test]
fn catalog_replace_is_atomic_and_remove_rejects_live_dependents() {
    let leaf = entry("game.Leaf", "Leaf", "value", "value");
    let root = entry("game.Root", "Root", "value", "value")
        .with_dependencies(vec!["game.Leaf".to_string()]);
    let mut catalog = ReflectSchemaCatalog::try_new(vec![leaf, root]).unwrap();
    let before = catalog.snapshot();

    let conflicting_id = catalog
        .entries()
        .next()
        .unwrap()
        .registration
        .type_info
        .fields[0]
        .id;
    let invalid_replacement = ReflectSchemaCatalogEntry::new(registration(
        "game.Root",
        "Root",
        conflicting_id,
        "other",
        &[],
    ));
    assert!(catalog.try_replace(invalid_replacement).is_err());
    assert_eq!(catalog.snapshot(), before);

    assert!(matches!(
        catalog.try_remove("game.Leaf"),
        Err(ReflectError::InvalidRegistration { .. })
    ));
    assert!(catalog.try_remove("game.Root").unwrap().is_some());
    assert!(catalog.try_remove("game.Leaf").unwrap().is_some());
    assert!(catalog.is_empty());
}
