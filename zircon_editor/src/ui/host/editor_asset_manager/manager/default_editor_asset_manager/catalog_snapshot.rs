use super::super::super::EditorAssetCatalogSnapshotRecord;
use super::super::folder_projection::build_folder_records;
use super::{record_to_facade::record_to_facade, DefaultEditorAssetManager};

impl DefaultEditorAssetManager {
    pub(crate) fn catalog_snapshot_record(&self) -> EditorAssetCatalogSnapshotRecord {
        let state = self.state.read().expect("editor asset state lock poisoned");
        let mut assets = state
            .catalog_by_uuid
            .values()
            .map(|record| record_to_facade(record, &state))
            .collect::<Vec<_>>();
        assets.sort_by(|left, right| left.locator.cmp(&right.locator));

        EditorAssetCatalogSnapshotRecord {
            project_name: state.project_name.clone(),
            project_root: state
                .project_root
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
            assets_root: state
                .assets_root
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
            library_root: state
                .library_root
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
            default_scene_uri: state
                .default_scene_uri
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            catalog_revision: state.catalog_revision,
            folders: build_folder_records(&state),
            assets,
        }
    }
}
