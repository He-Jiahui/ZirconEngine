use super::*;

pub(super) fn asset_drag_source_catalog() -> EditorAssetCatalogSnapshotRecord {
    EditorAssetCatalogSnapshotRecord {
        project_name: "Sandbox".to_string(),
        project_root: "E:/Sandbox".to_string(),
        assets_root: "E:/Sandbox/assets".to_string(),
        cache_root: "E:/Sandbox/.zircon/cache".to_string(),
        default_scene_uri: "res://scenes/main.scene.toml".to_string(),
        catalog_revision: 1,
        folders: vec![EditorAssetFolderRecord {
            folder_id: "res://".to_string(),
            parent_folder_id: None,
            locator_prefix: "res://".to_string(),
            display_name: "Assets".to_string(),
            child_folder_ids: Vec::new(),
            direct_asset_uuids: vec!["asset-uuid-1".to_string()],
            recursive_asset_count: 1,
        }],
        assets: vec![EditorAssetCatalogRecord {
            uuid: "asset-uuid-1".to_string(),
            id: "asset-id-1".to_string(),
            locator: "res://grid.albedo.png".to_string(),
            kind: ResourceKind::Texture,
            display_name: "Grid Albedo".to_string(),
            file_name: "grid.albedo.png".to_string(),
            extension: "png".to_string(),
            preview_state: PreviewState::Ready,
            meta_path: "E:/Sandbox/assets/grid.albedo.png.zmeta".to_string(),
            preview_artifact_path: "E:/Sandbox/.zircon/cache/editor-previews/grid.png".to_string(),
            source_mtime_unix_ms: 1,
            source_hash: "grid".to_string(),
            dirty: false,
            diagnostics: Vec::new(),
            direct_reference_uuids: Vec::new(),
        }],
    }
}

pub(super) fn shared_asset_drag_source_catalog() -> Arc<EditorAssetCatalogGeneration> {
    Arc::new(EditorAssetCatalogGeneration::from_snapshot_record(
        asset_drag_source_catalog(),
        1,
    ))
}

pub(super) fn asset_drag_source_catalog_with_reference() -> (
    EditorAssetCatalogSnapshotRecord,
    EditorAssetCatalogRecord,
    EditorAssetCatalogRecord,
) {
    let mut catalog = asset_drag_source_catalog();
    catalog.folders[0]
        .direct_asset_uuids
        .push("asset-uuid-2".to_string());
    catalog.folders[0].recursive_asset_count = 2;
    catalog.assets[0]
        .direct_reference_uuids
        .push("asset-uuid-2".to_string());
    let material_asset = EditorAssetCatalogRecord {
        uuid: "asset-uuid-2".to_string(),
        id: "asset-id-2".to_string(),
        locator: "res://materials/runtime_demo.mat".to_string(),
        kind: ResourceKind::Material,
        display_name: "Runtime Demo".to_string(),
        file_name: "runtime_demo.mat".to_string(),
        extension: "mat".to_string(),
        preview_state: PreviewState::Ready,
        meta_path: "E:/Sandbox/assets/materials/runtime_demo.mat.zmeta".to_string(),
        preview_artifact_path: "E:/Sandbox/.zircon/cache/editor-previews/runtime_demo.png"
            .to_string(),
        source_mtime_unix_ms: 2,
        source_hash: "runtime-demo".to_string(),
        dirty: false,
        diagnostics: Vec::new(),
        direct_reference_uuids: Vec::new(),
    };
    let source_asset = catalog.assets[0].clone();
    catalog.assets.push(material_asset.clone());
    (catalog, source_asset, material_asset)
}
