use crate::asset::project::ProjectGenerationPhase;
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetStatusRecord, AssetUri, AssetUuid};
use crate::core::CoreError;

use super::super::errors::{asset_error, asset_error_message};
use super::super::records::build_status_record;
use super::ProjectAssetManager;

impl ProjectAssetManager {
    /// Relocates an active project source through the Runtime asset pipeline.
    ///
    /// This is the only public mutation entry: it prepares a project candidate, reserves the
    /// live resource rename batch, commits authoring files, then installs the project generation
    /// and publishes the change while the generation gate is held.
    pub fn relocate_project_source(
        &self,
        source_uuid: AssetUuid,
        target: AssetUri,
    ) -> Result<Vec<AssetStatusRecord>, CoreError> {
        let (expected_generation, expected_preparation_epoch, mut candidate) = {
            let _generation = self.project_generation_read();
            let project = self.project_read();
            let Some(active_project) = project.as_ref() else {
                return Err(asset_error_message(
                    "project source relocation requires an active project",
                ));
            };
            (
                active_project.catalog_input_generation().sequence(),
                self.current_project_preparation_epoch(),
                active_project.clone(),
            )
        };
        let prepared_files = candidate
            .prepare_project_source_relocation(source_uuid, target)
            .map_err(asset_error)?;
        if prepared_files.updated_records().is_empty() {
            return Ok(Vec::new());
        }
        let statuses = prepared_files
            .updated_records()
            .iter()
            .map(build_status_record)
            .collect::<Vec<_>>();
        let source = prepared_files.source().clone();
        let target = prepared_files.target().clone();
        let prepared_resources =
            self.prepare_project_source_relocation_resource_sync(&prepared_files);

        let _phase = ProjectGenerationPhase::FileCommit.enter();
        let generation = self.project_generation_write();
        let mut project = self.project_write();
        let Some(active_project) = project.as_ref() else {
            return Err(asset_error_message(
                "project source relocation lost its active project before commit",
            ));
        };
        if active_project.catalog_input_generation().sequence() != expected_generation
            || self.current_project_preparation_epoch() != expected_preparation_epoch
        {
            return Err(asset_error_message(
                "project source relocation was superseded by a newer project generation",
            ));
        }
        let commit_outcome = self.commit_project_source_relocation_resource_sync(
            prepared_resources,
            || prepared_files.commit().map_err(asset_error),
            || {
                *project = Some(candidate);
                drop(project);
            },
        )?;
        self.publish_project_generation(
            generation,
            vec![AssetChange::new(
                AssetChangeKind::Renamed,
                target,
                Some(source),
            )],
        );
        commit_outcome.ensure_durable().map_err(asset_error)?;
        Ok(statuses)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
    use crate::asset::{
        AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
        AssetKind, AssetManager, AssetUri, DataAsset, DataAssetFormat, FunctionAssetImporter,
        ImportedAsset,
    };
    use zircon_runtime_interface::project::RelPath;

    use super::ProjectAssetManager;

    #[test]
    fn pipeline_relocation_commits_live_resource_rename_and_change_event() {
        let root = unique_temp_project_root("pipeline_source_relocation");
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
        ProjectManifest::new(
            "PipelineSourceRelocation",
            AssetUri::parse("res://data/original.counted").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        let source_path = paths
            .asset_root(&RelPath::project_assets())
            .join("data/original.counted");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::write(&source_path, "pipeline-relocation-v1").unwrap();
        let source = AssetUri::parse("res://data/original.counted").unwrap();
        let target = AssetUri::parse("res://moved/renamed.counted").unwrap();

        let manager = ProjectAssetManager::default();
        manager
            .register_asset_importer(counted_data_importer())
            .unwrap();
        manager
            .open_prepared_project(ProjectManager::open(&root).unwrap())
            .unwrap();
        let source_uuid = manager
            .current_project_manager()
            .unwrap()
            .asset_registry()
            .entry_by_path(&source)
            .unwrap()
            .uuid();
        let source_id = manager.resolve_asset_id(&source).unwrap();
        let changes = AssetManager::subscribe_asset_changes(&manager);

        let statuses = manager
            .relocate_project_source(source_uuid, target.clone())
            .expect("pipeline relocation should commit the durable generation");

        assert!(statuses
            .iter()
            .any(|status| status.uri == target.to_string()));
        assert_eq!(manager.resolve_asset_id(&target), Some(source_id));
        assert_eq!(manager.resolve_asset_id(&source), None);
        let change = changes
            .try_recv()
            .expect("pipeline relocation publishes a renamed asset change");
        assert_eq!(change.kind, crate::asset::watch::AssetChangeKind::Renamed);
        assert_eq!(change.uri, target);
        assert_eq!(change.previous_uri, Some(source));

        let _ = fs::remove_dir_all(root);
    }

    fn counted_data_importer() -> FunctionAssetImporter {
        FunctionAssetImporter::new(
            AssetImporterDescriptor::new(
                "test.pipeline.counted.data",
                "test.pipeline.counted",
                AssetKind::Data,
                1,
            )
            .with_source_extensions(["counted"]),
            import_counted_data,
        )
    }

    fn import_counted_data(
        context: &AssetImportContext,
    ) -> Result<AssetImportOutcome, AssetImportError> {
        let text = context.source_text()?;
        Ok(AssetImportOutcome::new(
            context.uri.clone(),
            ImportedAsset::Data(DataAsset {
                uri: context.uri.clone(),
                format: DataAssetFormat::Json,
                text,
                canonical_json: serde_json::json!({ "pipeline": true }),
            }),
        ))
    }

    fn unique_temp_project_root(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "zircon_{label}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
}
