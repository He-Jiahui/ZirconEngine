use crate::core::framework::channel::{ChannelReceiver, ChannelWakeCallback};
use crate::core::CoreError;
use crossbeam_channel::unbounded;

use super::super::errors::{asset_error, asset_error_message};
use super::super::project_asset_manager::ProjectAssetManager;
use super::super::records::{build_project_info, build_status_record};
use super::super::resource_sync::{clear_removed_project_resources, project_locators};
use super::super::{
    AssetManager as AssetManagerContract, AssetPipelineInfo, AssetStatusRecord, ProjectInfo,
};
use crate::asset::project::ProjectManager;
use crate::asset::watch::{AssetChange, AssetChangeKind, AssetWatchError};
use crate::asset::{
    AssetImportError, AssetImporterCapabilityReport, AssetImporterHandler, AssetUri,
};
use crate::core::resource::{ResourceMutationBatch, ResourceScheme};
use std::sync::Arc;

impl AssetManagerContract for ProjectAssetManager {
    fn pipeline_info(&self) -> AssetPipelineInfo {
        AssetPipelineInfo {
            default_worker_count: self.default_worker_count(),
        }
    }

    fn register_asset_importer(
        &self,
        importer: Arc<dyn AssetImporterHandler>,
    ) -> Result<(), CoreError> {
        ProjectAssetManager::register_asset_importer_arc(self, importer)
    }

    fn asset_importer_capability_reports(&self) -> Vec<AssetImporterCapabilityReport> {
        ProjectAssetManager::asset_importer_capability_reports(self)
    }

    fn asset_importer_capability_report_for_source(
        &self,
        source_path: &str,
    ) -> Result<AssetImporterCapabilityReport, AssetImportError> {
        ProjectAssetManager::asset_importer_capability_report_for_source(
            self,
            std::path::Path::new(source_path),
        )
    }

    fn open_project(&self, root_path: &str) -> Result<ProjectInfo, CoreError> {
        let project = ProjectManager::open(root_path).map_err(asset_error)?;
        self.open_prepared_project(project)
    }

    fn open_prepared_project(&self, project: ProjectManager) -> Result<ProjectInfo, CoreError> {
        ProjectAssetManager::open_prepared_project(self, project)
    }

    fn close_project(&self) -> Result<Option<std::path::PathBuf>, CoreError> {
        ProjectAssetManager::close_project(self)
    }

    fn current_project_snapshot(&self) -> Option<ProjectManager> {
        self.project_read().as_ref().cloned()
    }

    fn current_project_source_path(
        &self,
        uri: &AssetUri,
    ) -> Result<Option<std::path::PathBuf>, AssetImportError> {
        let project = self.project_read();
        let Some(project) = project.as_ref() else {
            return Ok(None);
        };
        if let Some(path) = self.indexed_project_source_path(uri) {
            return Ok(Some(path));
        }
        match uri.scheme() {
            ResourceScheme::Res | ResourceScheme::Package => {
                Err(AssetImportError::MissingProjectAssetUri { uri: uri.clone() })
            }
            ResourceScheme::Library | ResourceScheme::Builtin | ResourceScheme::Memory => {
                project.source_path_for_uri(uri).map(Some)
            }
        }
    }

    fn current_project_asset_uris(&self) -> Vec<AssetUri> {
        let project = self.project_read();
        let Some(project) = project.as_ref() else {
            return Vec::new();
        };
        project
            .registry()
            .values()
            .map(|record| record.primary_locator().clone())
            .collect()
    }

    fn current_project(&self) -> Option<ProjectInfo> {
        self.project_read().as_ref().map(build_project_info)
    }

    fn asset_status(&self, uri: &str) -> Option<AssetStatusRecord> {
        let uri = AssetUri::parse(uri).ok()?;
        let project = self.project_read();
        let project = project.as_ref()?;
        project
            .registry()
            .get_by_locator(&uri)
            .map(build_status_record)
    }

    fn list_assets(&self) -> Vec<AssetStatusRecord> {
        let project = self.project_read();
        let Some(project) = project.as_ref() else {
            return Vec::new();
        };
        let mut assets = project
            .registry()
            .values()
            .map(build_status_record)
            .collect::<Vec<_>>();
        assets.sort_by(|left, right| left.uri.cmp(&right.uri));
        assets
    }

    fn subscribe_asset_changes(&self) -> ChannelReceiver<AssetChange> {
        self.subscribe_asset_changes_internal(None)
    }

    fn subscribe_asset_changes_with_wake(
        &self,
        wake: ChannelWakeCallback,
    ) -> ChannelReceiver<AssetChange> {
        self.subscribe_asset_changes_internal(Some(wake))
    }

    fn subscribe_asset_watch_errors(&self) -> ChannelReceiver<AssetWatchError> {
        let (sender, receiver) = unbounded();
        self.lock_watch_error_subscribers().push(sender);
        receiver
    }

    fn import_asset(&self, uri: &str) -> Result<Option<AssetStatusRecord>, CoreError> {
        let uri = AssetUri::parse(uri).map_err(asset_error)?;
        let (
            expected_generation,
            expected_preparation_epoch,
            mut candidate,
            source_path,
            previous_source_records,
        ) = {
            let _generation = self.project_generation_read();
            let indexed_source_path = self.indexed_project_source_path(&uri);
            let project = self.project_read();
            let Some(active_project) = project.as_ref() else {
                return Ok(None);
            };
            let source_path = match indexed_source_path {
                Some(path) => path,
                None if uri.scheme() == ResourceScheme::Res => active_project
                    .primary_project_source_path_for_uri(&uri)
                    .map_err(asset_error)?,
                None => {
                    return Err(asset_error(AssetImportError::MissingProjectAssetUri {
                        uri,
                    }));
                }
            };
            (
                active_project.catalog_input_generation().sequence(),
                self.current_project_preparation_epoch(),
                active_project.clone(),
                source_path,
                active_project.source_resource_records(&uri),
            )
        };
        let mut prepared_generation = candidate
            .prepare_targeted_generation(&uri, &source_path)
            .map_err(asset_error)?;
        let ready_payloads = prepared_generation.take_ready_payloads();
        let prepared = self.prepare_targeted_project_resource_sync(
            &candidate,
            &uri,
            source_path,
            &previous_source_records,
            prepared_generation.imported(),
            prepared_generation.affected(),
            ready_payloads,
        );
        let status = candidate
            .registry()
            .get_by_locator(&uri)
            .map(build_status_record);
        let generation = self.project_generation_write();
        let mut project = self.project_write();
        let Some(active_project) = project.as_ref() else {
            return Err(asset_error_message(
                "targeted asset import lost its active project before commit",
            ));
        };
        if active_project.catalog_input_generation().sequence() != expected_generation
            || self.current_project_preparation_epoch() != expected_preparation_epoch
        {
            return Err(asset_error_message(
                "targeted asset import was superseded by a newer project generation",
            ));
        }
        let commit_outcome = self.commit_targeted_project_resource_sync(
            prepared,
            || prepared_generation.commit().map_err(asset_error),
            || {
                *project = Some(candidate);
                drop(project);
            },
        )?;
        let published_changes = status
            .is_some()
            .then(|| AssetChange::new(AssetChangeKind::Modified, uri, None))
            .into_iter()
            .collect();
        self.publish_project_generation(generation, published_changes);
        commit_outcome.ensure_durable().map_err(asset_error)?;
        Ok(status)
    }

    fn reimport_all(&self) -> Result<Vec<AssetStatusRecord>, CoreError> {
        let (expected_generation, expected_preparation_epoch, previous_locators, mut candidate) = {
            let _generation = self.project_generation_read();
            let project = self.project_read();
            let Some(active_project) = project.as_ref() else {
                return Ok(Vec::new());
            };
            (
                active_project.catalog_input_generation().sequence(),
                self.current_project_preparation_epoch(),
                project_locators(active_project),
                active_project.clone(),
            )
        };
        let (imported, prepared_files) = candidate
            .prepare_full_generation(None)
            .map_err(asset_error)?;
        let prepared = self.prepare_project_resource_sync(&candidate)?;
        let statuses = imported.iter().map(build_status_record).collect::<Vec<_>>();
        let generation = self.project_generation_write();
        let mut project = self.project_write();
        let Some(active_project) = project.as_ref() else {
            return Err(asset_error_message(
                "project reimport lost its active project before commit",
            ));
        };
        if active_project.catalog_input_generation().sequence() != expected_generation
            || self.current_project_preparation_epoch() != expected_preparation_epoch
        {
            return Err(asset_error_message(
                "project reimport was superseded by a newer project generation",
            ));
        }
        let batch = clear_removed_project_resources(
            ResourceMutationBatch::new(),
            &previous_locators,
            &candidate,
        );
        let commit_outcome = self.commit_project_resource_sync(
            prepared,
            batch,
            || prepared_files.commit().map_err(asset_error),
            || {
                *project = Some(candidate);
                drop(project);
            },
        )?;
        self.publish_project_generation(
            generation,
            imported
                .into_iter()
                .map(|metadata| {
                    AssetChange::new(
                        AssetChangeKind::Modified,
                        metadata.primary_locator().clone(),
                        None,
                    )
                })
                .collect(),
        );
        commit_outcome.ensure_durable().map_err(asset_error)?;
        Ok(statuses)
    }
}

#[cfg(test)]
mod tests {
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

        fn import(
            &self,
            context: &AssetImportContext,
        ) -> Result<AssetImportOutcome, AssetImportError> {
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
        AssetManager::open_prepared_project(&manager, ProjectManager::open(&root).unwrap())
            .unwrap();
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
}
