use crate::ui::host::editor_asset_manager::{AssetCatalogRecord, EditorAssetCatalogRecord};

use super::editor_asset_state::EditorAssetState;

pub(super) fn record_to_facade(
    record: &AssetCatalogRecord,
    state: &EditorAssetState,
) -> EditorAssetCatalogRecord {
    EditorAssetCatalogRecord {
        uuid: record.asset_uuid.to_string(),
        id: record.asset_id.to_string(),
        locator: record.locator.to_string(),
        kind: record.kind,
        display_name: record.display_name.clone(),
        file_name: record.file_name.clone(),
        extension: record.extension.clone(),
        preview_state: record.preview_state,
        meta_path: record.meta_path.to_string_lossy().into_owned(),
        preview_artifact_path: record.preview_artifact_path.to_string_lossy().into_owned(),
        source_mtime_unix_ms: record.source_mtime_unix_ms,
        source_hash: record.source_hash.clone(),
        dirty: record.dirty,
        diagnostics: record.diagnostics.clone(),
        direct_reference_uuids: record
            .direct_references
            .iter()
            .map(|reference| {
                state
                    .catalog_by_uuid
                    .get(&reference.uuid)
                    .map(|target| target.asset_uuid)
                    .or_else(|| state.uuid_by_locator.get(&reference.locator).copied())
                    .unwrap_or(reference.uuid)
                    .to_string()
            })
            .collect(),
    }
}
