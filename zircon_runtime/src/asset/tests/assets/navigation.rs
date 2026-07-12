use serde::Serialize;

use crate::asset::{
    NavMeshAreaCostAsset, NavMeshAsset, NavMeshPolygonAsset, NavMeshTileAsset, NavigationAssetError,
};
use crate::core::framework::navigation::{NavLinkMotion, NavLinkTraversalMode, AREA_JUMP};

#[test]
fn navmesh_binary_roundtrip_reports_typed_errors() {
    let asset = NavMeshAsset::simple_quad("humanoid", 2.0);

    let bytes = asset.to_bytes().expect("serialize navmesh asset");
    let decoded = NavMeshAsset::from_bytes(&bytes).expect("deserialize navmesh asset");
    assert_eq!(decoded, asset);

    let error = NavMeshAsset::from_bytes(b"not a navmesh").expect_err("invalid bytes should fail");
    assert!(matches!(
        error,
        NavigationAssetError::Deserialize(_) | NavigationAssetError::UnsupportedVersion { .. }
    ));
    if let NavigationAssetError::Deserialize(error) = error {
        assert!(std::error::Error::source(&NavigationAssetError::Deserialize(error)).is_some());
    }
}

#[test]
fn navmesh_v1_link_asset_migrates_to_v2_contract() {
    let source = NavMeshAsset::simple_quad("humanoid", 2.0);
    let bytes = bincode::serialize(&NavMeshAssetV1 {
        version: 1,
        agent_type: source.agent_type.clone(),
        settings_hash: source.settings_hash,
        area_costs: source.area_costs.clone(),
        vertices: source.vertices.clone(),
        indices: source.indices.clone(),
        polygons: source.polygons.clone(),
        tiles: source.tiles.clone(),
        off_mesh_links: vec![NavMeshLinkAssetV1 {
            start: [-1.0, 0.0, 0.0],
            end: [1.0, 0.0, 0.0],
            width: 0.5,
            bidirectional: true,
            area: AREA_JUMP,
            cost_override: None,
            traversal_mode: NavLinkTraversalMode::Automatic,
        }],
    })
    .unwrap();

    let migrated = NavMeshAsset::from_bytes(&bytes).expect("migrate v1 navigation asset");

    assert_eq!(migrated.version, NavMeshAsset::VERSION);
    assert_eq!(migrated.off_mesh_links.len(), 1);
    let link = &migrated.off_mesh_links[0];
    assert_eq!(link.id, 1);
    assert_eq!(link.owner_entity, 0);
    assert_eq!(link.lane_index, 0);
    assert_eq!(link.motion, NavLinkMotion::Linear);
    assert_eq!(link.arc_height, 0.0);
}

#[test]
fn navmesh_binary_rejects_unsupported_version_with_typed_error() {
    let mut asset = NavMeshAsset::simple_quad("humanoid", 2.0);
    asset.version = NavMeshAsset::VERSION + 1;
    let bytes = bincode::serialize(&asset).unwrap();

    let error = NavMeshAsset::from_bytes(&bytes).expect_err("unsupported version must fail");

    assert!(matches!(
        error,
        NavigationAssetError::UnsupportedVersion { version } if version == NavMeshAsset::VERSION + 1
    ));
}

#[derive(Serialize)]
struct NavMeshAssetV1 {
    version: u32,
    agent_type: String,
    settings_hash: u64,
    area_costs: Vec<NavMeshAreaCostAsset>,
    vertices: Vec<[f32; 3]>,
    indices: Vec<u32>,
    polygons: Vec<NavMeshPolygonAsset>,
    tiles: Vec<NavMeshTileAsset>,
    off_mesh_links: Vec<NavMeshLinkAssetV1>,
}

#[derive(Serialize)]
struct NavMeshLinkAssetV1 {
    start: [f32; 3],
    end: [f32; 3],
    width: f32,
    bidirectional: bool,
    area: u8,
    cost_override: Option<f32>,
    traversal_mode: NavLinkTraversalMode,
}
