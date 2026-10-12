use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
    AssetImporterHandler, AssetKind, AssetManager, AssetUri, DataAsset, DataAssetFormat,
    ImportedAsset,
};
use crate::core::resource::ResourceMutationBatch;

use super::ProjectAssetManager;

#[derive(Debug)]
struct EpochSupersedingCountedImporter {
    descriptor: AssetImporterDescriptor,
    manager: Weak<ProjectAssetManager>,
    advance_epoch: Arc<AtomicBool>,
}

impl EpochSupersedingCountedImporter {
    fn new(manager: Weak<ProjectAssetManager>, advance_epoch: Arc<AtomicBool>) -> Self {
        Self {
            descriptor: AssetImporterDescriptor::new(
                "test.epoch.superseding.counted",
                "test.epoch.superseding",
                AssetKind::Data,
                1,
            )
            .with_source_extensions(["counted"]),
            manager,
            advance_epoch,
        }
    }
}

impl AssetImporterHandler for EpochSupersedingCountedImporter {
    fn descriptor(&self) -> &AssetImporterDescriptor {
        &self.descriptor
    }

    fn import(&self, context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
        if self.advance_epoch.swap(false, Ordering::SeqCst) {
            self.manager
                .upgrade()
                .expect("test manager remains alive while its importer runs")
                .begin_project_preparation();
        }
        let text = context.source_text()?;
        Ok(AssetImportOutcome::new(
            context.uri.clone(),
            ImportedAsset::Data(DataAsset {
                uri: context.uri.clone(),
                format: DataAssetFormat::Json,
                text,
                canonical_json: serde_json::json!({ "superseding": true }),
            }),
        ))
    }
}

#[test]
fn project_queries_resolve_inside_the_manager_without_cloning_a_project_snapshot() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_asset_manager_project_queries_{}_{}",
        std::process::id(),
        unique
    ));
    let shader_path = root.join("assets/shaders/query.wgsl");
    let unrelated_path = root.join("assets/shaders/unrelated.wgsl");
    fs::create_dir_all(shader_path.parent().unwrap()).unwrap();
    ProjectManifest::new(
        "Project Query Fixture",
        AssetUri::parse("res://shaders/query.wgsl").unwrap(),
        1,
    )
    .save(root.join("zircon-project.toml"))
    .unwrap();
    fs::write(
        &shader_path,
        "@vertex fn vs_main() -> @builtin(position) vec4f { return vec4f(0.0, 0.0, 0.0, 1.0); }",
    )
    .unwrap();
    fs::write(
        &unrelated_path,
        "@vertex fn vs_main() -> @builtin(position) vec4f { return vec4f(1.0, 0.0, 0.0, 1.0); }",
    )
    .unwrap();
    let manager = ProjectAssetManager::default();
    let mut project = ProjectManager::open(&root).unwrap();
    project.scan_and_import().unwrap();
    let prepared = manager.prepare_project_resource_sync(&project).unwrap();
    let mut project_state = manager.project_write();
    manager
        .commit_project_resource_sync(
            prepared,
            ResourceMutationBatch::new(),
            || Ok(()),
            || {
                *project_state = Some(project);
                drop(project_state);
            },
        )
        .unwrap();
    let locator = AssetUri::parse("res://shaders/query.wgsl").unwrap();
    let labelled = AssetUri::parse("res://shaders/query.wgsl#vertex").unwrap();
    let unrelated = AssetUri::parse("res://shaders/unrelated.wgsl").unwrap();

    fs::remove_file(&unrelated_path).unwrap();

    assert_eq!(
        AssetManager::current_project_source_path(&manager, &locator).unwrap(),
        Some(shader_path)
    );
    assert_eq!(
        AssetManager::current_project_source_path(&manager, &labelled).unwrap(),
        AssetManager::current_project_source_path(&manager, &locator).unwrap()
    );
    assert_eq!(
        AssetManager::current_project_source_path(&manager, &unrelated).unwrap(),
        Some(unrelated_path.clone())
    );
    assert!(AssetManager::current_project_asset_uris(&manager).contains(&locator));
    assert!(AssetManager::current_project_asset_uris(&manager).contains(&unrelated));

    let candidate = manager.project_read().as_ref().unwrap().clone();
    assert!(manager.prepare_project_resource_sync(&candidate).is_err());
    assert_eq!(
        AssetManager::current_project_source_path(&manager, &unrelated).unwrap(),
        Some(unrelated_path)
    );

    drop(manager);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn package_source_path_is_generation_indexed_and_missing_after_reimport_removes_it() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_asset_manager_package_queries_{}_{}",
        std::process::id(),
        unique
    ));
    let package_root = std::env::temp_dir().join(format!(
        "zircon_asset_manager_package_sources_{}_{}",
        std::process::id(),
        unique
    ));
    fs::create_dir_all(root.join("assets")).unwrap();
    fs::create_dir_all(package_root.join("data")).unwrap();
    ProjectManifest::new(
        "Package Query Fixture",
        AssetUri::parse("res://scenes/default.scene.toml").unwrap(),
        1,
    )
    .save(root.join("zircon-project.toml"))
    .unwrap();
    let package_source = package_root.join("data/settings.json");
    fs::write(&package_source, r#"{ "enabled": true }"#).unwrap();

    let mut project = ProjectManager::open(&root).unwrap();
    project
        .register_package_asset_root("com.zircon.fixture", &package_root)
        .unwrap();
    project.scan_and_import().unwrap();
    let manager = ProjectAssetManager::default();
    let prepared = manager.prepare_project_resource_sync(&project).unwrap();
    let mut project_state = manager.project_write();
    manager
        .commit_project_resource_sync(
            prepared,
            ResourceMutationBatch::new(),
            || Ok(()),
            || {
                *project_state = Some(project);
                drop(project_state);
            },
        )
        .unwrap();
    let locator = AssetUri::parse("package://com.zircon.fixture/data/settings.json").unwrap();

    assert_eq!(
        AssetManager::current_project_source_path(&manager, &locator).unwrap(),
        Some(package_source.clone())
    );

    fs::remove_file(&package_source).unwrap();
    AssetManager::reimport_all(&manager).unwrap();

    assert!(matches!(
        AssetManager::current_project_source_path(&manager, &locator),
        Err(AssetImportError::MissingProjectAssetUri { uri }) if uri == locator
    ));

    drop(manager);
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn targeted_facade_import_preserves_unrelated_deleted_generation_entry() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_asset_manager_targeted_import_{}_{}",
        std::process::id(),
        unique
    ));
    let target_path = root.join("assets/data/target.json");
    let unrelated_path = root.join("assets/data/unrelated.json");
    fs::create_dir_all(target_path.parent().unwrap()).unwrap();
    ProjectManifest::new(
        "Targeted Facade Fixture",
        AssetUri::parse("res://data/target.json").unwrap(),
        1,
    )
    .save(root.join("zircon-project.toml"))
    .unwrap();
    fs::write(&target_path, r#"{ "version": 1 }"#).unwrap();
    fs::write(&unrelated_path, r#"{ "retained": true }"#).unwrap();
    let manager = ProjectAssetManager::default();
    AssetManager::open_prepared_project(&manager, ProjectManager::open(&root).unwrap()).unwrap();
    let target = AssetUri::parse("res://data/target.json").unwrap();
    let unrelated = AssetUri::parse("res://data/unrelated.json").unwrap();

    fs::write(&target_path, r#"{ "version": 2 }"#).unwrap();
    fs::remove_file(&unrelated_path).unwrap();
    let status = AssetManager::import_asset(&manager, &target.to_string())
        .unwrap()
        .expect("targeted status");

    assert_eq!(status.uri, target.to_string());
    assert!(AssetManager::current_project_asset_uris(&manager).contains(&unrelated));
    assert_eq!(
        AssetManager::current_project_source_path(&manager, &unrelated).unwrap(),
        Some(unrelated_path)
    );
    drop(manager);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn targeted_facade_import_superseded_after_prepare_leaves_disk_generation_unchanged() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_asset_manager_targeted_superseded_{}_{}",
        std::process::id(),
        unique
    ));
    let target_path = root.join("assets/data/target.counted");
    fs::create_dir_all(target_path.parent().unwrap()).unwrap();
    ProjectManifest::new(
        "Targeted Superseded Fixture",
        AssetUri::parse("res://data/target.counted").unwrap(),
        1,
    )
    .save(root.join("zircon-project.toml"))
    .unwrap();
    fs::write(&target_path, "target-v1").unwrap();
    let paths = ProjectPaths::from_root(&root).unwrap();

    let manager = Arc::new(ProjectAssetManager::default());
    let advance_epoch = Arc::new(AtomicBool::new(false));
    let mut project = ProjectManager::open(&root).unwrap();
    project
        .register_asset_importer(EpochSupersedingCountedImporter::new(
            Arc::downgrade(&manager),
            advance_epoch.clone(),
        ))
        .unwrap();
    manager.open_prepared_project(project).unwrap();

    let target_uri = AssetUri::parse("res://data/target.counted").unwrap();
    let active_project = manager.current_project_snapshot().unwrap();
    let record = active_project
        .registry()
        .get_by_locator(&target_uri)
        .cloned()
        .unwrap();
    let artifact_path = paths
        .asset_artifact_root()
        .join(record.artifact_locator().unwrap().path());
    let meta_path = target_path.with_file_name("target.counted.zmeta");
    let registry_path = paths.registry_root().join("asset-registry.json");
    let artifact_before = fs::read(&artifact_path).unwrap();
    let meta_before = fs::read(&meta_path).unwrap();
    let registry_before = fs::read(&registry_path).unwrap();

    fs::write(&target_path, "target-v2").unwrap();
    advance_epoch.store(true, Ordering::SeqCst);
    let error = AssetManager::import_asset(&*manager, &target_uri.to_string()).unwrap_err();

    assert!(error.to_string().contains("superseded"));
    assert_eq!(fs::read(&artifact_path).unwrap(), artifact_before);
    assert_eq!(fs::read(&meta_path).unwrap(), meta_before);
    assert_eq!(fs::read(&registry_path).unwrap(), registry_before);
    assert_eq!(
        manager
            .current_project_snapshot()
            .unwrap()
            .registry()
            .get_by_locator(&target_uri)
            .unwrap()
            .source_hash,
        record.source_hash,
    );

    drop(manager);
    let _ = fs::remove_dir_all(root);
}
