use std::fs;

use crate::ui::host::editor_asset_manager::{editor_meta_path_for_source, EditorAssetMetaDocument};

use super::support::unique_temp_dir;

#[test]
fn editor_asset_metadata_migrates_legacy_runtime_editor_adapter_into_editor_sidecar() {
    let root = unique_temp_dir("editor_asset_metadata_migrate");
    let source_path = root.join("materials").join("grid.material.toml");
    let runtime_meta_path = source_path.with_file_name("grid.material.toml.meta.toml");
    let editor_meta_path = editor_meta_path_for_source(&source_path);
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(
        &runtime_meta_path,
        "editor_adapter = \"material.pbr\"\npreview_state = \"dirty\"\n",
    )
    .unwrap();

    let meta =
        EditorAssetMetaDocument::load_or_migrate(&editor_meta_path, &runtime_meta_path).unwrap();

    assert_eq!(meta.editor_adapter.as_deref(), Some("material.pbr"));
    assert!(editor_meta_path.exists());
    let saved = EditorAssetMetaDocument::load(&editor_meta_path).unwrap();
    assert_eq!(saved.editor_adapter.as_deref(), Some("material.pbr"));

    let _ = fs::remove_dir_all(root);
}
