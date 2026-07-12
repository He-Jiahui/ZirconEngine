use zircon_runtime::asset::{NavMeshAsset, NavigationAssetError};
use zircon_runtime::core::framework::navigation::{NavLinkMotion, AREA_JUMP};

#[test]
fn v1_link_asset_migrates_before_runtime_consumption() {
    let bytes = v1_link_asset_bytes();

    let migrated = NavMeshAsset::from_bytes(&bytes).expect("migrate v1 navigation asset");

    assert_eq!(migrated.version, NavMeshAsset::VERSION);
    assert_eq!(migrated.off_mesh_links.len(), 1);
    let link = &migrated.off_mesh_links[0];
    assert_eq!(link.id, 1);
    assert_eq!(link.motion, NavLinkMotion::Linear);
    assert_eq!(link.arc_height, 0.0);
}

#[test]
fn unsupported_asset_version_is_rejected_before_runtime_consumption() {
    let mut asset = NavMeshAsset::simple_quad("humanoid", 2.0);
    asset.version = NavMeshAsset::VERSION + 1;
    let bytes = asset.to_bytes().unwrap();

    let error = NavMeshAsset::from_bytes(&bytes).expect_err("unsupported version must fail");

    assert!(matches!(
        error,
        NavigationAssetError::UnsupportedVersion { version }
            if version == NavMeshAsset::VERSION + 1
    ));
}

/// Fixed bincode-1 layout for the v1 asset header and one automatic off-mesh link.
fn v1_link_asset_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    bytes.extend_from_slice(&8_u64.to_le_bytes());
    bytes.extend_from_slice(b"humanoid");
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    for _ in 0..5 {
        bytes.extend_from_slice(&0_u64.to_le_bytes());
    }
    bytes.extend_from_slice(&1_u64.to_le_bytes());
    for value in [-1.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.push(1);
    bytes.push(AREA_JUMP);
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes
}
