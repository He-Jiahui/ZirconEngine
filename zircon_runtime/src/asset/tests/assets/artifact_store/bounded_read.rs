use std::fs;

use crate::asset::project::ProjectPaths;
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    ArtifactStore, AssetId, AssetImportError, AssetKind, AssetUri, DataAsset, DataAssetFormat,
    ImportedAsset,
};
use crate::core::resource::ResourceRecord;

#[test]
fn artifact_store_rejects_raw_payload_over_read_budget_before_opening_chunks() {
    let root = unique_temp_project_root("artifact_store_bounded_read");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let uri = AssetUri::parse("res://data/bounded-read.json").unwrap();
    let asset = ImportedAsset::Data(DataAsset {
        uri: uri.clone(),
        format: DataAssetFormat::Json,
        text: "{\"bounded\":true}".to_string(),
        canonical_json: serde_json::json!({"bounded": true}),
    });
    let store = ArtifactStore::default();
    let artifact_uri = store
        .write(
            &paths,
            &ResourceRecord::new(AssetId::new(), AssetKind::Data, uri),
            &asset,
        )
        .unwrap();
    fs::remove_dir_all(paths.asset_artifact_root().join("chunks")).unwrap();

    let error = store
        .read_with_raw_payload_limit(&paths, &artifact_uri, 1)
        .expect_err("manifest raw bytes must be rejected before chunk I/O");

    assert!(matches!(
        error,
        AssetImportError::ArtifactRawPayloadLimitExceeded { limit_bytes: 1, .. }
    ));

    let _ = fs::remove_dir_all(root);
}
