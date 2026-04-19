use super::super::super::{EditorAssetDetailsRecord, EditorAssetReferenceRecord};
use super::{
    record_to_facade::record_to_facade, reference_to_facade::reference_to_facade,
    DefaultEditorAssetManager,
};

impl DefaultEditorAssetManager {
    pub(crate) fn asset_details_record(&self, uuid: &str) -> Option<EditorAssetDetailsRecord> {
        let asset_uuid = uuid.parse().ok()?;
        let state = self.state.read().expect("editor asset state lock poisoned");
        let record = state.catalog_by_uuid.get(&asset_uuid)?;

        let mut direct_references = record
            .direct_references
            .iter()
            .map(|reference| reference_to_facade(reference, &state))
            .collect::<Vec<_>>();
        direct_references.sort_by(|left, right| {
            left.display_name
                .cmp(&right.display_name)
                .then(left.locator.cmp(&right.locator))
        });

        let mut referenced_by = state
            .reference_graph
            .incoming(asset_uuid)
            .into_iter()
            .filter_map(|source_uuid| state.catalog_by_uuid.get(&source_uuid))
            .map(|source| EditorAssetReferenceRecord {
                uuid: source.asset_uuid.to_string(),
                locator: source.locator.to_string(),
                display_name: source.display_name.clone(),
                kind: Some(source.kind),
                known_project_asset: true,
            })
            .collect::<Vec<_>>();
        referenced_by.sort_by(|left, right| {
            left.display_name
                .cmp(&right.display_name)
                .then(left.locator.cmp(&right.locator))
        });

        Some(EditorAssetDetailsRecord {
            asset: record_to_facade(record),
            direct_references,
            referenced_by,
            editor_adapter: record.meta.editor_adapter.clone(),
        })
    }
}
