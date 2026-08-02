use std::fs::{self, File, FileTimes};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use zircon_runtime_interface::project::RelPath;

use crate::asset::project::{
    AssetMetaDocument, AssetMetaEntry, AssetSourceUnit, PackageAssetRegistry,
    ProjectCatalogInputGeneration, ProjectCatalogInputRecord, ProjectManager, ProjectManifest,
    ProjectPaths,
};
use crate::asset::{AssetKind, AssetReference, AssetUri, AssetUuid};
use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceRecord};

use super::super::unique_temp_project_root;

#[test]
fn project_catalog_input_generation_reuses_identity_when_every_input_is_unchanged() {
    let (root, source_path, mut project) = data_project("catalog_generation_unchanged");
    project.scan_and_import().unwrap();
    let first = project.catalog_input_generation();

    project.scan_and_import().unwrap();
    let second = project.catalog_input_generation();

    assert!(Arc::ptr_eq(&first, &second));
    assert!(second.delta_since(&first).is_unchanged());
    assert_eq!(second.records().count(), 1);

    drop(source_path);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_catalog_input_generation_detects_source_touch_with_unchanged_digest() {
    let (root, source_path, mut project) = data_project("catalog_generation_source_touch");
    project.scan_and_import().unwrap();
    let first = project.catalog_input_generation();
    let first_record = first.records().next().unwrap().clone();

    let touched_at = SystemTime::now() + Duration::from_secs(5);
    File::options()
        .write(true)
        .open(&source_path)
        .unwrap()
        .set_times(FileTimes::new().set_modified(touched_at))
        .unwrap();
    project.scan_and_import().unwrap();
    let second = project.catalog_input_generation();
    let second_record = second.record(first_record.resource().id()).unwrap();

    assert!(!Arc::ptr_eq(&first, &second));
    assert_eq!(
        first_record.resource().source_hash,
        second_record.resource().source_hash
    );
    assert_ne!(
        first_record.source_mtime_unix_ms(),
        second_record.source_mtime_unix_ms()
    );
    let delta = second.delta_since(&first);
    assert_eq!(delta.modified.len(), 1);
    assert!(delta.added.is_empty());
    assert!(delta.removed.is_empty());
    assert!(delta.renamed.is_empty());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_catalog_input_generation_detects_catalog_meta_projection_changes() {
    let root = PathBuf::from("catalog-meta-project");
    let manifest = manifest();
    let packages = PackageAssetRegistry::default();
    let mut meta = meta("res://data/catalog.json");
    let previous = ProjectCatalogInputGeneration::from_test_records(
        &root,
        manifest.clone(),
        packages.clone(),
        [test_record(&root, meta.clone(), Vec::new())],
    );

    meta.unit = AssetSourceUnit::Compound;
    meta.included_files = vec![locator("res://data/catalog/part.json")];
    meta.entries.push(AssetMetaEntry {
        uuid: AssetUuid::new(),
        url: locator("res://data/catalog.json#part"),
        asset_kind: AssetKind::Data,
        artifact_locator: None,
        dependencies: Vec::new(),
        tags: Default::default(),
    });
    let current = ProjectCatalogInputGeneration::from_test_records(
        &root,
        manifest,
        packages,
        [test_record(&root, meta, Vec::new())],
    );

    let delta = current.delta_since(&previous);
    assert_eq!(delta.modified.len(), 1);
    assert!(!delta.is_unchanged());
}

#[test]
fn project_catalog_input_generation_detects_artifact_reference_payload_changes() {
    let root = PathBuf::from("catalog-reference-project");
    let manifest = manifest();
    let packages = PackageAssetRegistry::default();
    let meta = meta("res://data/catalog.json");
    let previous = ProjectCatalogInputGeneration::from_test_records(
        &root,
        manifest.clone(),
        packages.clone(),
        [test_record(&root, meta.clone(), Vec::new())],
    );
    let current = ProjectCatalogInputGeneration::from_test_records(
        &root,
        manifest,
        packages,
        [test_record(
            &root,
            meta,
            vec![AssetReference::from_locator(locator(
                "res://materials/referenced.zmaterial",
            ))],
        )],
    );

    let delta = current.delta_since(&previous);
    assert_eq!(delta.modified.len(), 1);
    assert_eq!(
        delta.modified[0].direct_references()[0].locator,
        locator("res://materials/referenced.zmaterial")
    );
}

#[test]
fn project_catalog_input_generation_publishes_package_root_changes_atomically() {
    let (root, _source_path, mut project) = data_project("catalog_generation_package_root");
    project.scan_and_import().unwrap();
    let first = project.catalog_input_generation();

    project
        .register_package_asset_root("com.zircon.catalog", root.join("package-assets"))
        .unwrap();
    let second = project.catalog_input_generation();

    assert!(!Arc::ptr_eq(&first, &second));
    let delta = second.delta_since(&first);
    assert!(delta.project_metadata_changed);
    assert!(delta.added.is_empty());
    assert!(delta.modified.is_empty());
    assert!(delta.removed.is_empty());
    assert!(delta.renamed.is_empty());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_catalog_input_generation_classifies_added_removed_and_renamed_rows() {
    let root = PathBuf::from("catalog-delta-project");
    let manifest = manifest();
    let packages = PackageAssetRegistry::default();
    let removed = test_record(&root, meta("res://data/removed.json"), Vec::new());
    let mut renamed_before = test_record(&root, meta("res://data/before.json"), Vec::new());
    let renamed_id = renamed_before.resource().id();
    let previous = ProjectCatalogInputGeneration::from_test_records(
        &root,
        manifest.clone(),
        packages.clone(),
        [removed, renamed_before.clone()],
    );

    renamed_before.set_locator_for_test(locator("res://data/after.json"));
    assert_eq!(renamed_before.resource().id(), renamed_id);
    let added = test_record(&root, meta("res://data/added.json"), Vec::new());
    let current = ProjectCatalogInputGeneration::from_test_records(
        &root,
        manifest,
        packages,
        [renamed_before, added],
    );

    let delta = current.delta_since(&previous);
    assert_eq!(delta.added.len(), 1);
    assert_eq!(delta.removed.len(), 1);
    assert_eq!(delta.renamed.len(), 1);
    assert_eq!(
        delta.renamed[0].previous_locator,
        locator("res://data/before.json")
    );
    assert_eq!(
        delta.renamed[0].current_locator,
        locator("res://data/after.json")
    );
}

fn data_project(label: &str) -> (PathBuf, PathBuf, ProjectManager) {
    let root = unique_temp_project_root(label);
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    manifest().save(paths.manifest_path()).unwrap();
    let source_path = paths
        .asset_root(&RelPath::project_assets())
        .join("data")
        .join("catalog.json");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, br#"{"catalog":true}"#).unwrap();
    let project = ProjectManager::open(&root).unwrap();
    (root, source_path, project)
}

fn test_record(
    project_root: &Path,
    meta: AssetMetaDocument,
    direct_references: Vec<AssetReference>,
) -> ProjectCatalogInputRecord {
    let source_path = project_root.join(meta.url.path());
    let resource = ResourceRecord::new(
        ResourceId::from_asset_uuid(meta.uuid),
        ResourceKind::Data,
        meta.url.clone(),
    );
    ProjectCatalogInputRecord::new_for_test(
        resource,
        source_path.clone(),
        source_path.with_extension("zmeta"),
        meta,
        7,
        direct_references,
    )
}

fn meta(locator_text: &str) -> AssetMetaDocument {
    AssetMetaDocument::new(AssetUuid::new(), locator(locator_text), AssetKind::Data)
}

fn manifest() -> ProjectManifest {
    ProjectManifest::new(
        "Catalog Generation",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
}

fn locator(value: &str) -> ResourceLocator {
    ResourceLocator::parse(value).unwrap()
}
