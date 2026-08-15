use std::fs::{self, File, FileTimes};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use zircon_runtime_interface::project::RelPath;

use crate::asset::project::{
    AssetMetaDocument, AssetMetaEntry, AssetSourceUnit, PackageAssetRegistry,
    ProjectCatalogInputGeneration, ProjectCatalogInputRecord, ProjectCatalogInputSource,
    ProjectManager, ProjectManifest, ProjectPaths,
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

#[test]
fn project_root_change_reconciles_the_full_catalog_delta() {
    let previous_root = PathBuf::from("catalog-root-change-previous");
    let current_root = PathBuf::from("catalog-root-change-current");
    let manifest = manifest();
    let packages = PackageAssetRegistry::default();
    let record = test_record(
        &previous_root,
        meta("res://data/root-change.json"),
        Vec::new(),
    );
    let id = record.resource().id();
    let previous = ProjectCatalogInputGeneration::from_test_records(
        &previous_root,
        manifest.clone(),
        packages.clone(),
        [record],
    );

    let current = ProjectCatalogInputGeneration::publish_metadata(
        &previous,
        &current_root,
        &manifest,
        &packages,
    );

    let delta = current.delta_since(&previous);
    assert!(delta.project_metadata_changed);
    assert_eq!(delta.added.len(), 1);
    assert_eq!(delta.added[0].resource().id(), id);
    assert_eq!(delta.removed.len(), 1);
    assert_eq!(delta.removed[0].resource().id(), id);
    assert!(delta.modified.is_empty());
    assert!(delta.renamed.is_empty());
}

#[test]
fn skipped_catalog_delta_reconciles_added_removed_and_renamed_rows() {
    let root = PathBuf::from("catalog-skipped-delta-reconciliation");
    let manifest = manifest();
    let packages = PackageAssetRegistry::default();
    let removed = test_record(&root, meta("res://data/removed.json"), Vec::new());
    let removed_id = removed.resource().id();
    let mut renamed_before = test_record(&root, meta("res://data/before.json"), Vec::new());
    let renamed_id = renamed_before.resource().id();
    let previous = ProjectCatalogInputGeneration::from_test_records(
        &root,
        manifest.clone(),
        packages.clone(),
        [removed, renamed_before.clone()],
    );

    renamed_before.set_locator_for_test(locator("res://data/after.json"));
    let after_first = ProjectCatalogInputGeneration::publish_targeted(
        &previous,
        &root,
        &manifest,
        &packages,
        [renamed_before.resource().clone()],
        std::collections::HashMap::from([(renamed_id, source_from_record(&renamed_before))]),
        [removed_id],
    );
    let direct_delta = after_first.delta_since(&previous);
    assert_eq!(direct_delta.removed.len(), 1);
    assert_eq!(direct_delta.renamed.len(), 1);

    let added = test_record(&root, meta("res://data/added.json"), Vec::new());
    let added_id = added.resource().id();
    let current = ProjectCatalogInputGeneration::publish_targeted(
        &after_first,
        &root,
        &manifest,
        &packages,
        [added.resource().clone()],
        std::collections::HashMap::from([(added_id, source_from_record(&added))]),
        std::iter::empty(),
    );

    let skipped_delta = current.delta_since(&previous);
    assert_eq!(skipped_delta.added.len(), 1);
    assert_eq!(skipped_delta.added[0].resource().id(), added_id);
    assert_eq!(skipped_delta.removed.len(), 1);
    assert_eq!(skipped_delta.removed[0].resource().id(), removed_id);
    assert_eq!(skipped_delta.renamed.len(), 1);
    assert_eq!(
        skipped_delta.renamed[0].previous_locator,
        locator("res://data/before.json")
    );
    assert_eq!(
        skipped_delta.renamed[0].current_locator,
        locator("res://data/after.json")
    );
    assert!(skipped_delta.modified.is_empty());
}

#[test]
fn targeted_catalog_publish_reuses_unchanged_record_identity() {
    let root = PathBuf::from("catalog-targeted-publication");
    let manifest = manifest();
    let packages = PackageAssetRegistry::default();
    let unchanged = test_record(&root, meta("res://data/unchanged.json"), Vec::new());
    let changed = test_record(&root, meta("res://data/changed.json"), Vec::new());
    let unchanged_id = unchanged.resource().id();
    let changed_id = changed.resource().id();
    let previous = ProjectCatalogInputGeneration::from_test_records(
        &root,
        manifest.clone(),
        packages.clone(),
        [unchanged, changed.clone()],
    );

    let mut changed_resource = changed.resource().clone();
    changed_resource.source_hash = "changed".to_owned();
    let changed_source = ProjectCatalogInputSource::new(
        changed.source_path().to_path_buf(),
        changed.meta_path().to_path_buf(),
        changed.meta().clone(),
        changed.source_mtime_unix_ms(),
        changed.direct_references().to_vec(),
    );

    let current = ProjectCatalogInputGeneration::publish_targeted(
        &previous,
        &root,
        &manifest,
        &packages,
        [changed_resource],
        std::collections::HashMap::from([(changed_id, changed_source)]),
        std::iter::empty(),
    );

    assert!(!Arc::ptr_eq(&previous, &current));
    assert!(std::ptr::eq(
        previous.record(unchanged_id).unwrap(),
        current.record(unchanged_id).unwrap(),
    ));
    assert!(!std::ptr::eq(
        previous.record(changed_id).unwrap(),
        current.record(changed_id).unwrap(),
    ));
    let delta = current.delta_since(&previous);
    assert!(delta.added.is_empty());
    assert_eq!(delta.modified.len(), 1);
    assert!(delta.removed.is_empty());
    assert!(delta.renamed.is_empty());
}

#[test]
fn targeted_catalog_publish_removes_only_the_requested_record() {
    let root = PathBuf::from("catalog-targeted-removal");
    let manifest = manifest();
    let packages = PackageAssetRegistry::default();
    let retained = test_record(&root, meta("res://data/retained.json"), Vec::new());
    let removed = test_record(&root, meta("res://data/removed.json"), Vec::new());
    let retained_id = retained.resource().id();
    let removed_id = removed.resource().id();
    let previous = ProjectCatalogInputGeneration::from_test_records(
        &root,
        manifest.clone(),
        packages.clone(),
        [retained, removed],
    );

    let current = ProjectCatalogInputGeneration::publish_targeted(
        &previous,
        &root,
        &manifest,
        &packages,
        std::iter::empty(),
        std::collections::HashMap::new(),
        [removed_id],
    );

    assert!(std::ptr::eq(
        previous.record(retained_id).unwrap(),
        current.record(retained_id).unwrap(),
    ));
    assert!(current.record(removed_id).is_none());
    let delta = current.delta_since(&previous);
    assert!(delta.added.is_empty());
    assert!(delta.modified.is_empty());
    assert_eq!(delta.removed.len(), 1);
    assert!(delta.renamed.is_empty());
}

#[test]
fn targeted_catalog_replacement_is_modified_after_removing_its_previous_record() {
    let root = PathBuf::from("catalog-targeted-replacement");
    let manifest = manifest();
    let packages = PackageAssetRegistry::default();
    let original = test_record(&root, meta("res://data/replaced.json"), Vec::new());
    let id = original.resource().id();
    let previous = ProjectCatalogInputGeneration::from_test_records(
        &root,
        manifest.clone(),
        packages.clone(),
        [original.clone()],
    );
    let mut replacement = original.resource().clone();
    replacement.source_hash = "replacement".to_owned();
    let replacement_source = ProjectCatalogInputSource::new(
        original.source_path().to_path_buf(),
        original.meta_path().to_path_buf(),
        original.meta().clone(),
        original.source_mtime_unix_ms(),
        original.direct_references().to_vec(),
    );

    let current = ProjectCatalogInputGeneration::publish_targeted(
        &previous,
        &root,
        &manifest,
        &packages,
        [replacement],
        std::collections::HashMap::from([(id, replacement_source)]),
        [id],
    );

    let delta = current.delta_since(&previous);
    assert!(delta.added.is_empty());
    assert_eq!(delta.modified.len(), 1);
    assert!(delta.removed.is_empty());
    assert!(delta.renamed.is_empty());
}

#[test]
fn catalog_delta_uses_local_successor_changes_and_reconciles_skipped_generations() {
    let root = PathBuf::from("catalog-local-successor-delta");
    let manifest = manifest();
    let packages = PackageAssetRegistry::default();
    let first = test_record(&root, meta("res://data/first.json"), Vec::new());
    let second = test_record(&root, meta("res://data/second.json"), Vec::new());
    let first_id = first.resource().id();
    let second_id = second.resource().id();
    let initial = ProjectCatalogInputGeneration::from_test_records(
        &root,
        manifest.clone(),
        packages.clone(),
        [first.clone(), second.clone()],
    );

    let mut first_changed = first.resource().clone();
    first_changed.source_hash = "first-changed".to_owned();
    let after_first = ProjectCatalogInputGeneration::publish_targeted(
        &initial,
        &root,
        &manifest,
        &packages,
        [first_changed],
        std::collections::HashMap::from([(first_id, source_from_record(&first))]),
        std::iter::empty(),
    );
    let mut second_changed = second.resource().clone();
    second_changed.source_hash = "second-changed".to_owned();
    let after_second = ProjectCatalogInputGeneration::publish_targeted(
        &after_first,
        &root,
        &manifest,
        &packages,
        [second_changed],
        std::collections::HashMap::from([(second_id, source_from_record(&second))]),
        std::iter::empty(),
    );

    let direct_delta = after_second.delta_since(&after_first);
    assert_eq!(direct_delta.modified.len(), 1);
    assert_eq!(
        direct_delta.modified[0].resource().primary_locator(),
        &locator("res://data/second.json")
    );
    let skipped_delta = after_second.delta_since(&initial);
    assert_eq!(skipped_delta.modified.len(), 2);
    assert!(skipped_delta.added.is_empty());
    assert!(skipped_delta.removed.is_empty());
    assert!(skipped_delta.renamed.is_empty());
}

#[test]
fn targeted_import_remint_preserves_catalog_inputs_for_owner_and_target() {
    let (root, target_path, mut project) = data_project("catalog_targeted_remint");
    let owner_path = target_path.with_file_name("owner.json");
    let target_uri = AssetUri::parse("res://data/catalog.json").unwrap();
    let owner_uri = AssetUri::parse("res://data/owner.json").unwrap();
    fs::write(&owner_path, br#"{"owner":true}"#).unwrap();
    project.scan_and_import().unwrap();
    let catalog_before_remint = project.catalog_input_generation();
    let original_target_id = project.registry().get_by_locator(&target_uri).unwrap().id();

    let owner_meta_path = owner_path.with_file_name("owner.json.zmeta");
    let owner_meta = AssetMetaDocument::load(&owner_meta_path).unwrap();
    let target_meta_path = target_path.with_file_name("catalog.json.zmeta");
    let mut target_meta = AssetMetaDocument::load(&target_meta_path).unwrap();
    target_meta.uuid = owner_meta.uuid;
    target_meta.save(&target_meta_path).unwrap();

    project
        .import_targeted_source(&target_uri, &target_path)
        .unwrap();

    let target = project.registry().get_by_locator(&target_uri).unwrap();
    let owner = project.registry().get_by_locator(&owner_uri).unwrap();
    let catalog = project.catalog_input_generation();
    assert_ne!(target.id(), owner.id());
    assert_eq!(catalog.records().count(), 2);
    assert_eq!(
        catalog.record(target.id()).unwrap().source_path(),
        target_path.as_path()
    );
    assert_eq!(
        catalog.record(owner.id()).unwrap().source_path(),
        owner_path.as_path()
    );
    let delta = catalog.delta_since(&catalog_before_remint);
    assert_eq!(delta.added.len(), 1);
    assert_eq!(delta.added[0].resource().id(), target.id());
    assert_eq!(delta.removed.len(), 1);
    assert_eq!(delta.removed[0].resource().id(), original_target_id);
    assert!(delta.modified.is_empty());
    assert!(delta.renamed.is_empty());

    fs::remove_dir_all(root).unwrap();
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

fn source_from_record(record: &ProjectCatalogInputRecord) -> ProjectCatalogInputSource {
    ProjectCatalogInputSource::new(
        record.source_path().to_path_buf(),
        record.meta_path().to_path_buf(),
        record.meta().clone(),
        record.source_mtime_unix_ms(),
        record.direct_references().to_vec(),
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
