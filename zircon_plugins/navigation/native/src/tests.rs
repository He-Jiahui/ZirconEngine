use zircon_runtime::asset::{
    NavMeshAreaCostAsset, NavMeshAsset, NavMeshLinkAsset, NavMeshPolygonAsset,
};
use zircon_runtime::core::framework::navigation::{
    NavPathQuery, NavPathStatus, NavRaycastQuery, NavSampleQuery, NavigationErrorKind, AREA_JUMP,
    AREA_WALKABLE, DEFAULT_AREA_MASK,
};
use zircon_runtime::core::math::Real;

use super::*;

#[test]
fn native_recast_detour_modules_are_linked() {
    assert_eq!(native_backend_version(), 1);
    assert!(native_runtime_modules_available());
}

#[test]
fn simple_surface_path_uses_baked_asset() {
    let backend = RecastBackend;
    let asset = backend
        .bake_simple_surface(RecastBakeInput {
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
fn mismatched_agent_type_returns_structured_error() {
    let backend = RecastBackend;
    let asset = NavMeshAsset::simple_quad("humanoid", 5.0);
    let mut query = NavPathQuery::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    query.agent_type = "large_creature".to_string();

    let error = backend.find_path(&asset, &query).unwrap_err();

    assert_eq!(error.kind, NavigationErrorKind::InvalidConfiguration);
}

#[test]
fn navmesh_asset_binary_roundtrip_is_deterministic() {
    let asset = two_island_asset(true);
    let bytes = asset.to_bytes().unwrap();
    let roundtrip = NavMeshAsset::from_bytes(&bytes).unwrap();

    assert_eq!(roundtrip, asset);
    assert_eq!(roundtrip.to_bytes().unwrap(), bytes);
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
fn raycast_ignores_offmesh_links_for_straight_visibility() {
    let backend = RecastBackend;
    let asset = two_island_asset(true);

    let result = backend
        .raycast(
            &asset,
            &NavRaycastQuery {
                nav_mesh: None,
                start: [0.0, 0.0, 0.0],
                end: [8.0, 0.0, 0.0],
                agent_type: "humanoid".to_string(),
                area_mask: DEFAULT_AREA_MASK,
            },
        )
        .unwrap();

    assert!(result.hit);
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

#[test]
fn raycast_reports_gap_between_connected_islands_as_hit() {
    let backend = RecastBackend;
    let asset = two_island_asset(false);

    let result = backend
        .raycast(
            &asset,
            &NavRaycastQuery {
                nav_mesh: None,
                start: [0.0, 0.0, 0.0],
                end: [1.5, 0.0, 0.0],
                agent_type: "humanoid".to_string(),
                area_mask: DEFAULT_AREA_MASK,
            },
        )
        .unwrap();

    assert!(result.hit);
    assert!(result.position[0] > 1.0);
}

#[test]
fn triangle_mesh_bake_filters_steep_faces_through_recast_rasterization() {
    let backend = RecastBackend;

    let asset = backend
        .bake_triangle_mesh(RecastBakeMeshInput {
            agent_type: "humanoid".to_string(),
            vertices: vec![
                [-2.0, 0.0, -2.0],
                [2.0, 0.0, -2.0],
                [2.0, 0.0, 2.0],
                [-2.0, 0.0, 2.0],
                [6.0, 0.0, -1.0],
                [6.0, 3.0, -1.0],
                [6.0, 3.0, 1.0],
                [6.0, 0.0, 1.0],
            ],
            indices: vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7],
            triangle_areas: vec![AREA_WALKABLE; 4],
            default_area: AREA_WALKABLE,
        })
        .unwrap();

    assert!(asset.polygons.len() < 4);
    let baked_vertices = asset
        .debug_triangles()
        .iter()
        .flat_map(|triangle| triangle.vertices)
        .collect::<Vec<_>>();
    let min_y = baked_vertices
        .iter()
        .map(|vertex| vertex[1])
        .fold(Real::INFINITY, Real::min);
    let max_y = baked_vertices
        .iter()
        .map(|vertex| vertex[1])
        .fold(Real::NEG_INFINITY, Real::max);
    assert!(max_y - min_y < 0.5);
}

#[test]
fn triangle_mesh_bake_rejects_non_finite_vertices_before_native_ffi() {
    let backend = RecastBackend;

    let error = backend
        .bake_triangle_mesh(RecastBakeMeshInput {
            agent_type: "humanoid".to_string(),
            vertices: vec![[0.0, 0.0, 0.0], [Real::NAN, 0.0, 0.0], [0.0, 0.0, 1.0]],
            indices: vec![0, 1, 2],
            triangle_areas: Vec::new(),
            default_area: AREA_WALKABLE,
        })
        .unwrap_err();

    assert_eq!(error.kind, NavigationErrorKind::BakeFailed);
    assert!(error.message.contains("non-finite"));
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

fn two_island_asset(with_link: bool) -> NavMeshAsset {
    let mut asset = NavMeshAsset {
        version: NavMeshAsset::VERSION,
        agent_type: "humanoid".to_string(),
        settings_hash: 0,
        area_costs: vec![
            NavMeshAreaCostAsset {
                area: AREA_WALKABLE,
                cost: 1.0,
                walkable: true,
            },
            NavMeshAreaCostAsset {
                area: AREA_JUMP,
                cost: 2.0,
                walkable: true,
            },
        ],
        vertices: vec![
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
            [7.0, 0.0, -1.0],
            [9.0, 0.0, -1.0],
            [9.0, 0.0, 1.0],
            [7.0, 0.0, 1.0],
        ],
        indices: vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7],
        polygons: vec![
            NavMeshPolygonAsset {
                first_index: 0,
                index_count: 6,
                area: AREA_WALKABLE,
                tile: 0,
            },
            NavMeshPolygonAsset {
                first_index: 6,
                index_count: 6,
                area: AREA_WALKABLE,
                tile: 1,
            },
        ],
        tiles: Vec::new(),
        off_mesh_links: Vec::new(),
    };
    if with_link {
        asset.off_mesh_links.push(NavMeshLinkAsset {
            start: [1.0, 0.0, 0.0],
            end: [7.0, 0.0, 0.0],
            width: 0.5,
            bidirectional: true,
            area: AREA_JUMP,
            cost_override: None,
            traversal_mode: Default::default(),
        });
    }
    asset
}

fn corner_touching_fan_polygon_asset() -> NavMeshAsset {
    NavMeshAsset {
        version: NavMeshAsset::VERSION,
        agent_type: "humanoid".to_string(),
        settings_hash: 0,
        area_costs: Vec::new(),
        vertices: vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 0.0, 2.0],
            [0.0, 0.0, 2.0],
            [-1.0, 0.0, 0.0],
            [0.0, 0.0, -1.0],
        ],
        indices: vec![0, 1, 2, 0, 2, 3, 0, 4, 5],
        polygons: vec![
            NavMeshPolygonAsset {
                first_index: 0,
                index_count: 6,
                area: AREA_WALKABLE,
                tile: 0,
            },
            NavMeshPolygonAsset {
                first_index: 6,
                index_count: 3,
                area: AREA_WALKABLE,
                tile: 0,
            },
        ],
        tiles: Vec::new(),
        off_mesh_links: Vec::new(),
    }
}
