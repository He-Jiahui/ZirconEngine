use zircon_plugin_navigation_recast::RecastBackend;
use zircon_runtime::asset::{NavMeshAsset, NavMeshPolygonAsset, NavMeshTileAsset};
use zircon_runtime::core::framework::navigation::{
    NavPathQuery, NavPathStatus, NavRaycastQuery, NavSampleQuery, AREA_WALKABLE, DEFAULT_AREA_MASK,
};

#[test]
fn detour_path_query_string_pulls_corridor_without_graph_centroid_waypoints() {
    let backend = RecastBackend;
    let asset = three_square_corridor_asset();

    let result = backend
        .find_path(&asset, &NavPathQuery::new([0.2, 0.0, 0.5], [2.8, 0.0, 0.5]))
        .unwrap();

    assert_eq!(result.status, NavPathStatus::Complete);
    assert_eq!(result.points.len(), 2);
    assert_eq!(result.visited_nodes, 3);
    assert!((result.length - 2.6).abs() < 0.001);
}

#[test]
fn detour_sample_query_projects_to_navmesh_inside_extents() {
    let backend = RecastBackend;
    let asset = three_square_corridor_asset();

    let hit = backend
        .sample_position(
            &asset,
            &NavSampleQuery {
                nav_mesh: None,
                position: [1.5, 3.0, 0.5],
                extents: [0.25, 5.0, 0.25],
                agent_type: "humanoid".to_string(),
                area_mask: DEFAULT_AREA_MASK,
            },
        )
        .unwrap()
        .unwrap();

    assert_eq!(hit.position, [1.5, 0.0, 0.5]);
    assert_eq!(hit.area, AREA_WALKABLE);
    assert!((hit.distance - 3.0).abs() < 0.001);
}

#[test]
fn detour_raycast_reports_wall_hit_at_polygon_boundary() {
    let backend = RecastBackend;
    let asset = three_square_corridor_asset();

    let result = backend
        .raycast(
            &asset,
            &NavRaycastQuery {
                nav_mesh: None,
                start: [0.5, 0.0, 0.5],
                end: [4.0, 0.0, 0.5],
                agent_type: "humanoid".to_string(),
                area_mask: DEFAULT_AREA_MASK,
            },
        )
        .unwrap();

    assert!(result.hit);
    assert!(
        (result.position[0] - 3.0).abs() < 0.01,
        "expected boundary hit near x=3.0, got {result:?}"
    );
    assert!(
        (result.distance - 2.5).abs() < 0.01,
        "expected boundary distance near 2.5, got {result:?}"
    );
}

#[test]
fn tile_cache_carved_obstacle_blocks_corridor_path() {
    let backend = RecastBackend;
    let asset = three_square_corridor_asset();

    let result = backend
        .find_path_with_obstacles(
            &asset,
            &NavPathQuery::new([0.2, 0.0, 0.5], [2.8, 0.0, 0.5]),
            &[
                zircon_plugin_navigation_recast::RecastNavigationObstacle::box_obstacle(
                    [1.5, 0.0, 0.5],
                    [0.55, 1.0, 0.6],
                ),
            ],
        )
        .unwrap();

    assert_eq!(result.status, NavPathStatus::NoPath);
}

fn three_square_corridor_asset() -> NavMeshAsset {
    NavMeshAsset {
        version: NavMeshAsset::VERSION,
        agent_type: "humanoid".to_string(),
        settings_hash: 0,
        area_costs: NavMeshAsset::empty("humanoid").area_costs,
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [2.0, 0.0, 1.0],
            [3.0, 0.0, 1.0],
        ],
        indices: vec![0, 1, 5, 0, 5, 4, 1, 2, 6, 1, 6, 5, 2, 3, 7, 2, 7, 6],
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
                tile: 0,
            },
            NavMeshPolygonAsset {
                first_index: 12,
                index_count: 6,
                area: AREA_WALKABLE,
                tile: 0,
            },
        ],
        tiles: vec![NavMeshTileAsset {
            id: 0,
            bounds_min: [0.0, 0.0, 0.0],
            bounds_max: [3.0, 0.0, 1.0],
            polygon_count: 3,
        }],
        off_mesh_links: Vec::new(),
    }
}
