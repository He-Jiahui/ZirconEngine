use super::super::super::{
    EditorAssetDetailsRecord, EditorAssetReferenceRecord, EditorAssetSubassetRecord,
};
use super::{
    record_to_view::record_to_view, reference_to_view::reference_to_view, DefaultEditorAssetManager,
};

impl DefaultEditorAssetManager {
    pub(crate) fn asset_details_record(&self, uuid: &str) -> Option<EditorAssetDetailsRecord> {
        let asset_uuid = uuid.parse().ok()?;
        let state = self.state.read().expect("editor asset state lock poisoned");
        let record = state.catalog_by_uuid.get(&asset_uuid)?;

        let mut direct_references = record
            .direct_references
            .iter()
            .map(|reference| reference_to_view(reference, &state))
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
            asset: record_to_view(record, &state),
            direct_references,
            referenced_by,
            editor_adapter: record.editor_meta.editor_adapter.clone(),
            package_id: record.locator.package_id().map(str::to_string),
            unit: record.meta.unit,
            included_files: record
                .meta
                .included_files
                .iter()
                .map(ToString::to_string)
                .collect(),
            subassets: record
                .meta
                .entries
                .iter()
                .filter(|entry| entry.url.label().is_some())
                .map(|entry| EditorAssetSubassetRecord {
                    uuid: entry.uuid.to_string(),
                    locator: entry.url.to_string(),
                    kind: entry.asset_kind,
                    artifact_locator: entry.artifact_locator.as_ref().map(ToString::to_string),
                    dependency_locators: entry
                        .dependencies
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                })
                .collect(),
        })
    }
}
