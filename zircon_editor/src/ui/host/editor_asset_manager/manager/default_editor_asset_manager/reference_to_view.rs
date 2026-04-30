use zircon_runtime::asset::AssetReference;

use super::super::super::EditorAssetReferenceRecord;
use super::super::preview_refresh::display_name_for_locator::display_name_for_locator;
use super::EditorAssetState;

pub(super) fn reference_to_view(
    reference: &AssetReference,
    state: &EditorAssetState,
) -> EditorAssetReferenceRecord {
    if let Some(record) = state.catalog_by_uuid.get(&reference.uuid).or_else(|| {
        state
            .uuid_by_locator
            .get(&reference.locator)
            .and_then(|uuid| state.catalog_by_uuid.get(uuid))
    }) {
        return EditorAssetReferenceRecord {
            uuid: record.asset_uuid.to_string(),
            locator: record.locator.to_string(),
            display_name: record.display_name.clone(),
            kind: Some(record.kind),
            known_project_asset: true,
        };
    }

    EditorAssetReferenceRecord {
        uuid: reference.uuid.to_string(),
        locator: reference.locator.to_string(),
        display_name: display_name_for_locator(&reference.locator),
        kind: None,
        known_project_asset: false,
    }
}
