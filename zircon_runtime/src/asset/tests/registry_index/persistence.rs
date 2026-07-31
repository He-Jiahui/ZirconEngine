use std::collections::BTreeSet;

use crate::asset::registry::{AssetRegistryDiagnostic, AssetRegistryFilter, AssetRegistryIndex};
use crate::asset::{AssetKind, AssetUuid};
use crate::foundation::persistence::atomic_file::AtomicWriteFault;

use super::{registry_root, unique_root, uri, write_asset, write_asset_with_tags};

#[test]
fn corrupt_persisted_index_is_regenerated_from_multiple_project_roots() {
    let project = unique_root("corrupt_multi_root");
    let first = project.join("assets");
    let second = project.join("shared");
    let texture = AssetUuid::new();
    let material = AssetUuid::new();
    write_asset(
        &first,
        "textures/hero.png",
        texture,
        AssetKind::Texture,
        vec![],
    );
    write_asset(
        &second,
        "materials/hero.zmaterial",
        material,
        AssetKind::Material,
        vec![uri("res://textures/hero.png")],
    );
    let persisted_root = registry_root(&project);
    std::fs::create_dir_all(&persisted_root).unwrap();
    std::fs::write(persisted_root.join("asset-registry.json"), b"not-json").unwrap();

    let index = AssetRegistryIndex::load_or_rebuild(&[first, second], &persisted_root).unwrap();

    assert_eq!(index.len(), 2);
    assert_eq!(index.get_dependencies_by_uuid(material), vec![texture]);
    assert!(index.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        AssetRegistryDiagnostic::CorruptPersistenceRebuilt { .. }
    )));
    assert!(persisted_root.join("asset-registry.json").is_file());
    std::fs::remove_dir_all(project).unwrap();
}

#[test]
fn tag_filter_survives_sidecar_rebuild_and_corrupt_registry_recovery() {
    let project = unique_root("tag_authority");
    let assets = project.join("assets");
    let tagged = AssetUuid::new();
    write_asset_with_tags(
        &assets,
        "materials/tagged.zmaterial",
        tagged,
        AssetKind::Material,
        vec![],
        BTreeSet::from(["environment".to_string(), "hero".to_string()]),
    );
    write_asset(
        &assets,
        "materials/plain.zmaterial",
        AssetUuid::new(),
        AssetKind::Material,
        vec![],
    );
    let persisted_root = registry_root(&project);
    let filter = AssetRegistryFilter::default().with_tag("hero");

    let rebuilt =
        AssetRegistryIndex::rebuild_from_project(std::slice::from_ref(&assets), &persisted_root)
            .unwrap();
    assert_eq!(rebuilt.get_assets(&filter)[0].uuid(), tagged);
    assert_eq!(rebuilt.get_assets(&filter).len(), 1);

    std::fs::write(persisted_root.join("asset-registry.json"), b"corrupt").unwrap();
    let recovered = AssetRegistryIndex::load_or_rebuild(&[assets], &persisted_root).unwrap();
    assert_eq!(recovered.get_assets(&filter)[0].uuid(), tagged);
    assert_eq!(recovered.get_assets(&filter).len(), 1);
    assert!(recovered.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        AssetRegistryDiagnostic::CorruptPersistenceRebuilt { .. }
    )));
    std::fs::remove_dir_all(project).unwrap();
}

#[test]
fn later_duplicate_guid_is_reminted_in_its_sidecar_with_typed_diagnostic() {
    let project = unique_root("duplicate_guid");
    let assets = project.join("assets");
    let duplicate = AssetUuid::new();
    write_asset(&assets, "a.data", duplicate, AssetKind::Data, vec![]);
    let second = write_asset(&assets, "b.data", duplicate, AssetKind::Data, vec![]);

    let index =
        AssetRegistryIndex::rebuild_from_project(&[assets], registry_root(&project)).unwrap();

    let second_entry = index.entry_by_path(&uri("res://b.data")).unwrap();
    assert_ne!(second_entry.uuid(), duplicate);
    assert!(index.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic,
        AssetRegistryDiagnostic::DuplicateGuidReminted { original, path, replacement, .. }
            if *original == duplicate && path == &uri("res://b.data") && *replacement == second_entry.uuid()
    )));
    let rewritten =
        crate::asset::project::AssetMetaDocument::load(second.with_file_name("b.data.zmeta"))
            .unwrap();
    assert_eq!(rewritten.uuid, second_entry.uuid());
    std::fs::remove_dir_all(project).unwrap();
}

#[test]
fn registry_atomic_write_faults_keep_formal_file_readable_and_unchanged() {
    let project = unique_root("registry_atomic_faults");
    let assets = project.join("assets");
    write_asset(
        &assets,
        "data/original.data",
        AssetUuid::new(),
        AssetKind::Data,
        vec![],
    );
    let persisted_root = registry_root(&project);
    let original = AssetRegistryIndex::rebuild_from_project(&[assets], &persisted_root).unwrap();
    let formal = persisted_root.join("asset-registry.json");
    let original_bytes = std::fs::read(&formal).unwrap();
    let replacement =
        AssetRegistryIndex::from_entries([crate::asset::registry::AssetRegistryEntry::new(
            AssetUuid::new(),
            uri("res://data/replacement.data"),
            AssetKind::Data,
            "replacement",
        )])
        .unwrap();

    for fault in [
        AtomicWriteFault::Write,
        AtomicWriteFault::Sync,
        AtomicWriteFault::Replace,
    ] {
        replacement
            .persist_with_atomic_fault(&persisted_root, fault)
            .unwrap_err();
        assert_eq!(std::fs::read(&formal).unwrap(), original_bytes);
        assert_eq!(
            crate::asset::registry::persistence::load(&formal).unwrap(),
            original
        );
        assert_eq!(std::fs::read_dir(&persisted_root).unwrap().count(), 1);
    }
    std::fs::remove_dir_all(project).unwrap();
}

#[test]
fn persistence_decode_and_version_errors_keep_typed_sources_and_values() {
    let project = unique_root("registry_typed_persistence_errors");
    let root = registry_root(&project);
    std::fs::create_dir_all(&root).unwrap();
    let formal = root.join("asset-registry.json");
    std::fs::write(&formal, b"not-json").unwrap();
    assert!(matches!(
        crate::asset::registry::persistence::load(&formal).unwrap_err(),
        crate::asset::registry::AssetRegistryError::DecodePersistence { source, .. }
            if source.is_syntax()
    ));
    std::fs::write(&formal, br#"{"format_version":99,"entries":[]}"#).unwrap();
    assert!(matches!(
        crate::asset::registry::persistence::load(&formal).unwrap_err(),
        crate::asset::registry::AssetRegistryError::UnsupportedPersistenceVersion {
            found: 99,
            supported: 1,
            ..
        }
    ));
    std::fs::remove_dir_all(project).unwrap();
}
