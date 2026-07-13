use zircon_runtime::core::framework::navigation::NavMeshAsset;

use super::support::two_island_asset;

#[test]
fn navmesh_asset_binary_roundtrip_is_deterministic() {
    let asset = two_island_asset(true);
    let bytes = asset.to_bytes().unwrap();
    let roundtrip = NavMeshAsset::from_bytes(&bytes).unwrap();

    assert_eq!(roundtrip, asset);
    assert_eq!(roundtrip.to_bytes().unwrap(), bytes);
}
