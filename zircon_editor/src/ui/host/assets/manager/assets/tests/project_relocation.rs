use zircon_runtime::asset::{AssetUri, AssetUuid};
use zircon_runtime::core::CoreError;

use super::DefaultEditorAssetManager;

#[test]
fn relocation_requires_the_runtime_asset_owner() {
    let error = DefaultEditorAssetManager::new()
        .submit_project_source_relocation(
            AssetUuid::from_stable_label("editor-relocation-requires-runtime"),
            AssetUri::parse("res://relocated.asset").unwrap(),
        )
        .unwrap_err();

    assert_eq!(
        error,
        CoreError::Initialization(
            "EditorAssetManager".to_string(),
            "project source relocation requires the runtime asset manager".to_string(),
        )
    );
}
