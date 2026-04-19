use zircon_runtime::asset::project::PreviewState;
use zircon_runtime::asset::importer::AssetImportError;
use zircon_runtime::asset::AssetUuid;

use crate::AssetCatalogRecord;
use super::super::super::{EditorAssetChangeKind, EditorAssetChangeRecord};
use super::super::default_editor_asset_manager::DefaultEditorAssetManager;

impl DefaultEditorAssetManager {
    pub fn mark_preview_dirty(
        &self,
        asset_uuid: AssetUuid,
    ) -> Result<Option<AssetCatalogRecord>, AssetImportError> {
        let change = {
            let mut state = self
                .state
                .write()
                .expect("editor asset state lock poisoned");
            let updated = {
                let Some(record) = state.catalog_by_uuid.get_mut(&asset_uuid) else {
                    return Ok(None);
                };
                record.preview_state = PreviewState::Dirty;
                record.dirty = true;
                record.meta.preview_state = PreviewState::Dirty;
                record.meta.save(&record.meta_path)?;
                record.clone()
            };
            state.preview_scheduler.mark_dirty(asset_uuid);
            Some(EditorAssetChangeRecord {
                kind: EditorAssetChangeKind::PreviewChanged,
                catalog_revision: state.catalog_revision,
                uuid: Some(updated.asset_uuid.to_string()),
                locator: Some(updated.locator.to_string()),
            })
        };
        if let Some(change) = change {
            self.broadcast(change);
        }
        Ok(self.record_by_uuid(asset_uuid))
    }
}
