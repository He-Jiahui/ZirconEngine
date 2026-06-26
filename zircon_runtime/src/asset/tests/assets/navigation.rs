use crate::asset::{NavMeshAsset, NavigationAssetError};

#[test]
fn navmesh_binary_roundtrip_reports_typed_errors() {
    let asset = NavMeshAsset::simple_quad("humanoid", 2.0);

    let bytes = asset.to_bytes().expect("serialize navmesh asset");
    let decoded = NavMeshAsset::from_bytes(&bytes).expect("deserialize navmesh asset");
    assert_eq!(decoded, asset);

    let error = NavMeshAsset::from_bytes(b"not a navmesh").expect_err("invalid bytes should fail");
    assert!(matches!(error, NavigationAssetError::Deserialize(_)));
    assert!(std::error::Error::source(&error).is_some());
}
