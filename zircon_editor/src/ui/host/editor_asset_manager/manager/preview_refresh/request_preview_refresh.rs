use zircon_runtime::asset::importer::AssetImportError;
use zircon_runtime::asset::project::PreviewState;
use zircon_runtime::asset::AssetUuid;

use super::super::super::{EditorAssetChangeKind, EditorAssetChangeRecord};
use super::super::default_editor_asset_manager::DefaultEditorAssetManager;
use super::generate_preview_artifact::generate_preview_artifact;
use crate::ui::host::editor_asset_manager::AssetCatalogRecord;

impl DefaultEditorAssetManager {
    pub fn request_preview_refresh(
        &self,
        asset_uuid: AssetUuid,
        visible: bool,
    ) -> Result<Option<AssetCatalogRecord>, AssetImportError> {
        let change = {
            let mut state = self
                .state
                .write()
                .expect("editor asset state lock poisoned");
            let should_refresh = state.preview_scheduler.request_refresh(asset_uuid, visible);
            let catalog_revision = state.catalog_revision;
            let cache = state.preview_cache.as_ref().cloned().ok_or_else(|| {
                AssetImportError::Parse("preview cache is not initialized".to_string())
            })?;
            let project = state.project.as_ref().cloned().ok_or_else(|| {
                AssetImportError::Parse("editor project is not initialized".to_string())
            })?;
            let Some(record) = state.catalog_by_uuid.get_mut(&asset_uuid) else {
                return Ok(None);
            };
            if !should_refresh {
                return Ok(Some(record.clone()));
            }

            match generate_preview_artifact(&project, record, &cache) {
                Ok(path) => {
                    record.preview_artifact_path = path;
                    record.preview_state = PreviewState::Ready;
                    record.dirty = false;
                    record.meta.preview_state = PreviewState::Ready;
                    record.meta.save(&record.meta_path)?;
                }
                Err(error) => {
                    record.preview_state = PreviewState::Error;
                    record.dirty = false;
                    record.meta.preview_state = PreviewState::Error;
                    record.meta.save(&record.meta_path)?;
                    return Err(error);
                }
            }

            Some(EditorAssetChangeRecord {
                kind: EditorAssetChangeKind::PreviewChanged,
                catalog_revision,
                uuid: Some(record.asset_uuid.to_string()),
                locator: Some(record.locator.to_string()),
            })
        };
        if let Some(change) = change {
            self.broadcast(change);
        }
        Ok(self.record_by_uuid(asset_uuid))
    }
}
