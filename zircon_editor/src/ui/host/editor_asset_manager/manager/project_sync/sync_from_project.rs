use std::collections::HashMap;

use zircon_runtime::asset::importer::AssetImportError;
use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::project::{AssetMetaDocument, PreviewState};
use zircon_runtime::core::resource::ResourceState;

use crate::ui::host::editor_asset_manager::{editor_meta_path_for_source, EditorAssetMetaDocument};
use crate::ui::host::editor_asset_manager::{
    AssetCatalogRecord, PreviewArtifactKey, PreviewCache, PreviewScheduler, ReferenceGraph,
};

use super::super::super::{EditorAssetChangeKind, EditorAssetChangeRecord};
use super::super::default_editor_asset_manager::DefaultEditorAssetManager;
use super::super::reference_analysis::direct_references;
use super::{
    display_name_for_path::display_name_for_path, meta_path_for_source::meta_path_for_source,
    preview_source_mtime::preview_source_mtime,
};

impl DefaultEditorAssetManager {
    pub fn sync_from_project(&self, project: ProjectManager) -> Result<(), AssetImportError> {
        let preview_cache = PreviewCache::new(project.paths().library_root())?;
        let mut catalog_by_uuid = HashMap::new();
        let mut uuid_by_locator = HashMap::new();
        let mut preview_scheduler = PreviewScheduler::default();

        for metadata in project.registry().values() {
            let locator = metadata.primary_locator().clone();
            let source_path = project.source_path_for_uri(&locator)?;
            let meta_path = meta_path_for_source(&source_path);
            let meta = AssetMetaDocument::load(&meta_path)?;
            let editor_meta_path = editor_meta_path_for_source(&source_path);
            let editor_meta = EditorAssetMetaDocument::load_or_default(&editor_meta_path)?;
            let preview_state = meta.preview_state;
            let direct_references = if metadata.state == ResourceState::Ready {
                let imported = project.load_artifact_by_id(metadata.id())?;
                direct_references(&imported)
            } else {
                Vec::new()
            };
            let preview_artifact_path =
                preview_cache.path_for(&PreviewArtifactKey::thumbnail(meta.asset_uuid));
            let file_name = source_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string();
            let extension = source_path
                .extension()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let diagnostics = metadata
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.clone())
                .collect::<Vec<_>>();

            let record = AssetCatalogRecord {
                asset_uuid: meta.asset_uuid,
                asset_id: metadata.id(),
                locator: locator.clone(),
                kind: metadata.kind,
                display_name: display_name_for_path(&source_path, &locator),
                file_name,
                extension,
                meta_path,
                meta,
                editor_meta_path,
                editor_meta,
                source_mtime_unix_ms: preview_source_mtime(&source_path),
                source_hash: metadata.source_hash.clone(),
                preview_state,
                preview_artifact_path,
                dirty: preview_state == PreviewState::Dirty,
                diagnostics,
                direct_references,
            };
            if record.dirty {
                preview_scheduler.mark_dirty(record.asset_uuid);
            }

            uuid_by_locator.insert(locator, record.asset_uuid);
            catalog_by_uuid.insert(record.asset_uuid, record);
        }

        let reference_graph = ReferenceGraph::rebuild(catalog_by_uuid.values());
        let change = {
            let mut state = self
                .state
                .write()
                .expect("editor asset state lock poisoned");
            state.project_root = Some(project.paths().root().to_path_buf());
            state.assets_root = Some(project.paths().assets_root().to_path_buf());
            state.library_root = Some(project.paths().library_root().to_path_buf());
            state.project_name = project.manifest().name.clone();
            state.default_scene_uri = Some(project.manifest().default_scene.clone());
            state.catalog_revision += 1;
            state.project = Some(project);
            state.catalog_by_uuid = catalog_by_uuid;
            state.uuid_by_locator = uuid_by_locator;
            state.reference_graph = reference_graph;
            state.preview_cache = Some(preview_cache);
            state.preview_scheduler = preview_scheduler;

            EditorAssetChangeRecord {
                kind: EditorAssetChangeKind::CatalogChanged,
                catalog_revision: state.catalog_revision,
                uuid: None,
                locator: None,
            }
        };
        self.broadcast(change);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
    use zircon_runtime::asset::AssetUri;

    use super::*;

    #[test]
    fn sync_from_project_keeps_error_assets_without_artifacts_in_catalog() {
        let root = unique_temp_project_root("sync_error_asset_without_artifact");
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths.ensure_layout().unwrap();
        ProjectManifest::new(
            "BrokenAssetProject",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        let material_path = paths
            .assets_root()
            .join("materials")
            .join("broken.material.toml");
        fs::create_dir_all(material_path.parent().unwrap()).unwrap();
        fs::write(&material_path, "not valid toml = [").unwrap();

        let mut project = ProjectManager::open(&root).unwrap();
        let records = project.scan_and_import().unwrap();
        assert!(records.iter().any(
            |record| record.state == ResourceState::Error && record.artifact_locator.is_none()
        ));

        let manager = DefaultEditorAssetManager::new();
        manager.sync_from_project(project).unwrap();
        let catalog = manager.catalog_snapshot_record();
        let broken = catalog
            .assets
            .iter()
            .find(|asset| asset.locator == "res://materials/broken.material.toml")
            .expect("broken material remains visible in editor catalog");
        assert!(!broken.diagnostics.is_empty());
        assert!(broken.direct_reference_uuids.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    fn unique_temp_project_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("zircon_editor_{label}_{nanos}"))
    }
}
