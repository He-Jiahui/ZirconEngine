use zircon_runtime::asset::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    NavPathQuery, NavPathStatus, NavigationErrorKind, AREA_WALKABLE, DEFAULT_AREA_MASK,
};

use crate::RecastBackend;

use super::support::{corner_touching_fan_polygon_asset, two_island_asset};

#[test]
fn simple_surface_path_uses_baked_asset() {
    let backend = RecastBackend;
    let asset = backend
        .bake_simple_surface(crate::RecastBakeInput {
            agent_type: "humanoid".to_string(),
            source_vertices: 4,
            source_triangles: 2,
            half_extent: 5.0,
        })
        .unwrap();

    let result = backend
        .find_path(&asset, &NavPathQuery::new([0.0, 0.0, 0.0], [3.0, 0.0, 4.0]))
        .unwrap();

    assert_eq!(result.status, NavPathStatus::Complete);
    assert_eq!(result.length, 5.0);
    assert_eq!(result.points.len(), 2);
}

#[test]
fn area_mask_can_block_walkable_area() {
    let backend = RecastBackend;
    let asset = NavMeshAsset::simple_quad("humanoid", 5.0);
    let mut query = NavPathQuery::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    query.area_mask = DEFAULT_AREA_MASK & !(1_u64 << AREA_WALKABLE);

    let result = backend.find_path(&asset, &query).unwrap();

    assert_eq!(result.status, NavPathStatus::NoPath);
}

#[test]
fn disconnected_polygons_return_no_path_without_link() {
    let backend = RecastBackend;
    let asset = two_island_asset(false);

    let result = backend
        .find_path(&asset, &NavPathQuery::new([0.0, 0.0, 0.0], [8.0, 0.0, 0.0]))
        .unwrap();

    assert_eq!(result.status, NavPathStatus::NoPath);
}

#[test]
fn off_mesh_link_bridges_disconnected_polygons() {
    let backend = RecastBackend;
    let asset = two_island_asset(true);

    let result = backend
        .find_path(&asset, &NavPathQuery::new([0.0, 0.0, 0.0], [8.0, 0.0, 0.0]))
        .unwrap();

    assert_eq!(result.status, NavPathStatus::Complete);
    assert!(result
        .points
        .iter()
        .any(|point| point.flags.iter().any(|flag| flag == "off_mesh_link")));
}

#[test]
fn mismatched_agent_type_returns_structured_error() {
    let backend = RecastBackend;
    let asset = NavMeshAsset::simple_quad("humanoid", 5.0);
    let mut query = NavPathQuery::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    query.agent_type = "large_creature".to_string();

    let error = backend.find_path(&asset, &query).unwrap_err();

    assert_eq!(error.kind, NavigationErrorKind::InvalidConfiguration);
}

#[test]
fn polygon_graph_requires_shared_edge_not_repeated_fan_root() {
    let backend = RecastBackend;
    let asset = corner_touching_fan_polygon_asset();

    let result = backend
        .find_path(
            &asset,
            &NavPathQuery::new([1.0, 0.0, 1.0], [-0.25, 0.0, -0.25]),
        )
        .unwrap();

    assert_eq!(result.status, NavPathStatus::NoPath);
}
