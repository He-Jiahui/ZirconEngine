use zircon_runtime::core::framework::navigation::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    NavSampleQuery, AREA_WALKABLE, DEFAULT_AREA_MASK,
};

use crate::RecastBackend;

use super::support::two_island_asset;

#[test]
fn sample_clamps_to_asset_bounds() {
    let backend = RecastBackend;
    let asset = NavMeshAsset::simple_quad("humanoid", 5.0);

    let hit = backend
        .sample_position(
            &asset,
            &NavSampleQuery {
                nav_mesh: None,
                position: [10.0, 0.0, 0.0],
                extents: [6.0, 1.0, 6.0],
                agent_type: "humanoid".to_string(),
                area_mask: DEFAULT_AREA_MASK,
            },
        )
        .unwrap()
        .unwrap();

    assert!((hit.position[0] - 5.0).abs() < 0.001);
    assert_eq!(hit.position[1], 0.0);
    assert_eq!(hit.position[2], 0.0);
}

#[test]
fn sample_position_uses_nearest_polygon_not_gap_aabb() {
    let backend = RecastBackend;
    let asset = two_island_asset(false);

    let hit = backend
        .sample_position(
            &asset,
            &NavSampleQuery {
                nav_mesh: None,
                position: [4.0, 0.0, 0.0],
                extents: [5.0, 1.0, 5.0],
                agent_type: "humanoid".to_string(),
                area_mask: DEFAULT_AREA_MASK,
            },
        )
        .unwrap()
        .unwrap();

    assert_ne!(hit.position, [4.0, 0.0, 0.0]);
    assert!(hit.position[0] < 2.0 || hit.position[0] > 6.0);
}

#[test]
fn sample_position_projects_vertical_query_onto_polygon_plane() {
    let backend = RecastBackend;
    let asset = NavMeshAsset::simple_quad("humanoid", 5.0);

    let hit = backend
        .sample_position(
            &asset,
            &NavSampleQuery {
                nav_mesh: None,
                position: [0.0, 3.0, 0.0],
                extents: [1.0, 5.0, 1.0],
                agent_type: "humanoid".to_string(),
                area_mask: DEFAULT_AREA_MASK,
            },
        )
        .unwrap()
        .unwrap();

    assert_eq!(hit.position, [0.0, 0.0, 0.0]);
    assert_eq!(hit.distance, 3.0);
}

#[test]
fn sample_position_projects_to_triangle_edge_not_polygon_aabb_gap() {
    let backend = RecastBackend;
    let asset = NavMeshAsset::from_triangle_mesh(
        "humanoid",
        vec![[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 0.0, 4.0]],
        vec![0, 1, 2],
        AREA_WALKABLE,
    );

    let hit = backend
        .sample_position(
            &asset,
            &NavSampleQuery {
                nav_mesh: None,
                position: [3.5, 0.0, 3.5],
                extents: [10.0, 1.0, 10.0],
                agent_type: "humanoid".to_string(),
                area_mask: DEFAULT_AREA_MASK,
            },
        )
        .unwrap()
        .unwrap();

    assert!((hit.position[0] + hit.position[2] - 4.0).abs() < 0.001);
    assert_ne!(hit.position, [3.5, 0.0, 3.5]);
}
