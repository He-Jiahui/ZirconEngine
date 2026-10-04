use std::collections::HashMap;

use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime::asset::{AssetReference, AssetUri, AssetUuid};

use crate::ui::host::editor_asset_manager::{AssetCatalogRecord, EditorAssetCatalogRecord};

pub(in crate::ui::host::editor_asset_manager::manager) fn record_to_view(
    record: &AssetCatalogRecord,
    catalog_by_uuid: &HashMap<AssetUuid, AssetCatalogRecord>,
    uuid_by_locator: &HashMap<AssetUri, AssetUuid>,
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
        meta_path: ProjectPaths::display_path(&record.meta_path)
            .to_string_lossy()
            .into_owned(),
        preview_artifact_path: ProjectPaths::display_path(&record.preview_artifact_path)
            .to_string_lossy()
            .into_owned(),
        source_mtime_unix_ms: record.source_mtime_unix_ms,
        source_hash: record.source_hash.clone(),
        dirty: record.dirty,
        diagnostics: record.diagnostics.clone(),
        direct_reference_uuids: record
            .direct_references
            .iter()
            .map(|reference| {
                reference_record(reference, catalog_by_uuid, uuid_by_locator)
                    .map(|target| target.asset_uuid)
                    .unwrap_or(reference.uuid)
                    .to_string()
            })
            .collect(),
    }
}

pub(super) fn reference_record<'a>(
    reference: &AssetReference,
    catalog_by_uuid: &'a HashMap<AssetUuid, AssetCatalogRecord>,
    uuid_by_locator: &HashMap<AssetUri, AssetUuid>,
) -> Option<&'a AssetCatalogRecord> {
    if let Some(record) = catalog_by_uuid.get(&reference.uuid) {
        return Some(record);
    }
    // Only AssetReference::from_locator identities may resolve by locator. An explicit
    // missing UUID must not bind to a different asset that now occupies its old hint.
    if reference.uuid != AssetUuid::from_stable_label(&reference.locator.to_string()) {
        return None;
    }
    uuid_by_locator
        .get(&reference.locator)
        .and_then(|uuid| catalog_by_uuid.get(uuid))
}

#[cfg(test)]
#[path = "record/tests/reference_identity_tests.rs"]
mod reference_identity_tests;
