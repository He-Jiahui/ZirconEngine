use serde_json::json;
use std::time::{Duration, Instant};
use zircon_runtime::core::framework::navigation::{
    NavMeshBakeRequest, NavPathQuery, NavPathStatus, NavigationAreaSettings, NavigationManager,
    AREA_JUMP, NAV_MESH_MODIFIER_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
    NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Real, Transform, Vec3};
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::world::World;

use crate::{
    navigation_component_descriptors, DefaultNavigationManager, NavMeshBakeTaskState,
    NavMeshDirtyBounds,
};

#[test]
fn bake_surface_accepts_typed_resource_json_from_dynamic_properties() {
    let manager = DefaultNavigationManager::new();
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .register_component_type(navigation_component_descriptors()[0].clone())
        .unwrap();
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({
                "enabled": true,
                "volume_size": [4.0, 2.0, 4.0],
                "output_asset": {"resource": "res://navigation/level.znavmesh"}
            }),
        )
        .unwrap();

    let report = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap();

    assert_eq!(
        report.output_asset.as_deref(),
        Some("res://navigation/level.znavmesh")
    );
    assert_eq!(report.surfaces, 1);
    assert_eq!(report.source_triangles, 2);
    assert!(report.baked_polygons > 0);
    assert_eq!(report.tiles, 1);
}

#[test]
fn bake_surface_ignores_script_only_empty_nodes() {
    let manager = DefaultNavigationManager::new();
    let mut world = World::new();
    let surface = world.spawn_node(NodeKind::Cube);
    let empty = world.spawn_node(NodeKind::Empty);
    world
        .register_component_type(navigation_component_descriptors()[0].clone())
        .unwrap();
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({
                "enabled": true,
                "volume_size": [6.0, 2.0, 6.0]
            }),
        )
        .unwrap();
    world
        .update_transform(empty, Transform::from_translation(Vec3::new(1.0, 0.0, 1.0)))
        .unwrap();

    let report = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap();

    assert_eq!(report.source_triangles, 2);
    assert!(report.baked_polygons > 0);
}

#[test]
fn bake_surface_applies_modifier_area_and_embeds_offmesh_links() {
    let manager = DefaultNavigationManager::new();
    let mut world = World::new();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let surface = world.spawn_node(NodeKind::Cube);
    let link = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(link, Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)))
        .unwrap();
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({
                "enabled": true,
                "volume_size": [4.0, 2.0, 4.0]
            }),
        )
        .unwrap();
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_MODIFIER_COMPONENT_TYPE,
            json!({
                "override_area": true,
                "area": AREA_JUMP
            }),
        )
        .unwrap();
    world
        .set_dynamic_component(
            link,
            NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
            json!({
                "activated": true,
                "agent_type": "humanoid",
                "start_local_point": [0.0, 0.0, 0.0],
                "end_local_point": [2.0, 0.0, 0.0],
                "bidirectional": true,
                "area_type": AREA_JUMP
            }),
        )
        .unwrap();

    let report = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap();
    let source_triangles = report.source_triangles;
    let asset = report.asset.unwrap();

    assert_eq!(source_triangles, 2);
    assert!(asset
        .polygons
        .iter()
        .all(|polygon| polygon.area == AREA_JUMP));
    assert_eq!(asset.off_mesh_links.len(), 1);
}

#[test]
fn bake_surface_expands_offmesh_bridge_lanes_and_tracks_stats() {
    let manager = DefaultNavigationManager::new();
    let mut world = World::new();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let surface = world.spawn_node(NodeKind::Cube);
    let bridge = world.spawn_node(NodeKind::Cube);
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({
                "enabled": true,
                "volume_size": [4.0, 2.0, 4.0]
            }),
        )
        .unwrap();
    world
        .set_dynamic_component(
            bridge,
            NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE,
            json!({
                "activated": true,
                "agent_type": "humanoid",
                "start_local_point": [0.0, 0.0, 0.0],
                "end_local_point": [0.0, 0.0, 6.0],
                "width": 3.0,
                "lane_count": 3,
                "bidirectional": true,
                "area_type": AREA_JUMP
            }),
        )
        .unwrap();

    let asset = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap()
        .asset
        .unwrap();
    let stats = manager.stats();

    assert_eq!(asset.off_mesh_links.len(), 3);
    assert!(asset
        .off_mesh_links
        .iter()
        .all(|link| (link.width - 1.0).abs() <= Real::EPSILON));
    assert!(asset
        .off_mesh_links
        .iter()
        .any(|link| (link.start[0] - 1.0).abs() <= Real::EPSILON));
    assert!(asset
        .off_mesh_links
        .iter()
        .any(|link| link.start[0].abs() <= Real::EPSILON));
    assert!(asset
        .off_mesh_links
        .iter()
        .any(|link| (link.start[0] + 1.0).abs() <= Real::EPSILON));
    assert_eq!(stats.active_off_mesh_links, 0);
    assert_eq!(stats.active_off_mesh_bridges, 1);
}

#[test]
fn bake_surface_respects_disabled_link_generation_and_settings_hash() {
    let manager = DefaultNavigationManager::new();
    let mut world = World::new();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let surface = world.spawn_node(NodeKind::Cube);
    let link = world.spawn_node(NodeKind::Cube);
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({
                "enabled": true,
                "generate_links": false,
                "override_voxel_size": 0.25
            }),
        )
        .unwrap();
    world
        .set_dynamic_component(
            link,
            NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
            json!({
                "activated": true,
                "agent_type": "humanoid",
                "start_local_point": [0.0, 0.0, 0.0],
                "end_local_point": [2.0, 0.0, 0.0]
            }),
        )
        .unwrap();

    let first = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap();
    let first_asset = first.asset.unwrap();
    let mut settings = manager.active_settings();
    settings.areas.push(NavigationAreaSettings {
        id: 7,
        name: "mud".to_string(),
        cost: 5.0,
        walkable: true,
    });
    manager.load_navigation_settings(settings).unwrap();
    let second_asset = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap()
        .asset
        .unwrap();

    assert!(first_asset.off_mesh_links.is_empty());
    assert_ne!(first_asset.settings_hash, 0);
    assert_ne!(first_asset.settings_hash, second_asset.settings_hash);
    assert!(first
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("advanced Recast bake knobs")));
}

#[test]
fn carved_obstacle_removes_static_bake_source() {
    let manager = DefaultNavigationManager::new();
    let mut world = World::new();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let surface = world.spawn_node(NodeKind::Cube);
    let obstacle = world.spawn_node(NodeKind::Cube);
    world
        .set_dynamic_component(surface, NAV_MESH_SURFACE_COMPONENT_TYPE, json!({}))
        .unwrap();
    world
        .set_dynamic_component(
            obstacle,
            NAV_MESH_OBSTACLE_COMPONENT_TYPE,
            json!({
                "shape": "box",
                "size": [2.0, 2.0, 2.0],
                "carve": true
            }),
        )
        .unwrap();

    let report = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap();
    let asset = report.asset.unwrap();

    assert_eq!(report.source_triangles, 0);
    assert!(asset.is_empty());
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("carved")));
}

#[test]
fn bake_input_falls_back_to_render_mesh_without_physics() {
    let manager = DefaultNavigationManager::new();
    let mut world = World::empty();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let surface = world.spawn_node(NodeKind::Empty);
    world.spawn_node(NodeKind::Cube);
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({
                "use_geometry": "physics_colliders",
                "volume_size": [8.0, 4.0, 8.0]
            }),
        )
        .unwrap();

    let report = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap();

    assert_eq!(report.source_triangles, 2);
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("render mesh fallback")));
}

#[test]
fn golden_level_bake_then_path_length_within_tolerance() {
    let manager = DefaultNavigationManager::new();
    let mut world = World::empty();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let surface = world.spawn_node(NodeKind::Empty);
    world.spawn_node(NodeKind::Cube);
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({"volume_size": [8.0, 4.0, 8.0]}),
        )
        .unwrap();

    let asset = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap()
        .asset
        .unwrap();
    let handle = manager.load_nav_mesh(asset).unwrap();
    let mut query = NavPathQuery::new([-0.25, 0.0, -0.25], [0.25, 0.0, 0.25]);
    query.nav_mesh = Some(handle);

    let path = manager.find_path(query).unwrap();

    assert_eq!(path.status, NavPathStatus::Complete);
    assert!((path.length - 0.5_f32.sqrt()).abs() <= 0.1);
}

#[test]
fn modifier_volume_marks_area_id_in_polymesh() {
    let manager = DefaultNavigationManager::new();
    let mut world = World::empty();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let surface = world.spawn_node(NodeKind::Empty);
    let source = world.spawn_node(NodeKind::Cube);
    let modifier_volume = world.spawn_node(NodeKind::Empty);
    world
        .update_transform(
            source,
            Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(
            modifier_volume,
            Transform::from_translation(Vec3::new(2.0, 0.0, 0.0))
                .with_scale(Vec3::new(2.0, 2.0, 2.0)),
        )
        .unwrap();
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({"volume_size": [8.0, 4.0, 8.0]}),
        )
        .unwrap();
    world
        .set_dynamic_component(
            modifier_volume,
            NAV_MESH_MODIFIER_COMPONENT_TYPE,
            json!({
                "override_area": true,
                "area": AREA_JUMP
            }),
        )
        .unwrap();

    let asset = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap()
        .asset
        .unwrap();

    assert!(!asset.polygons.is_empty());
    assert!(asset
        .polygons
        .iter()
        .all(|polygon| polygon.area == AREA_JUMP));
}

#[test]
fn tiled_bake_does_not_block_main_thread() {
    let manager = DefaultNavigationManager::new();
    let world = tiled_test_world(24);
    let started_at = Instant::now();

    let task = manager.start_tiled_bake(world, NavMeshBakeRequest::default());

    assert!(started_at.elapsed() < Duration::from_millis(50));
    assert!(matches!(
        manager.bake_task_state(task),
        Some(NavMeshBakeTaskState::Pending | NavMeshBakeTaskState::Ready)
    ));
    let report = wait_for_tiled_bake(&manager, task);
    assert!(report.tiles > 1);
    assert!(manager
        .bake_diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.message.contains("tile")));
}

#[test]
fn tile_boundary_paths_are_continuous() {
    let manager = DefaultNavigationManager::new();
    let world = tiled_test_world(8);
    let asset = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap()
        .asset
        .unwrap();
    assert!(asset.tiles.len() > 1);
    let handle = manager.load_nav_mesh(asset).unwrap();
    let mut query = NavPathQuery::new([-3.75, 0.5, 0.0], [3.75, 0.5, 0.0]);
    query.nav_mesh = Some(handle);

    let path = manager.find_path(query).unwrap();

    assert_eq!(path.status, NavPathStatus::Complete);
    assert!(path.length >= 7.0 && path.length <= 8.0);
}

#[test]
fn dirty_tile_rebuild_only_affects_neighbors() {
    let manager = DefaultNavigationManager::new();
    let mut world = tiled_test_world(10);
    let initial = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap()
        .asset
        .unwrap();
    let changed = world.node_records()[6].id;
    world
        .update_transform(
            changed,
            Transform::from_translation(Vec3::new(0.5, 0.25, 0.0)),
        )
        .unwrap();

    let started_at = Instant::now();
    let task = manager.start_dirty_tile_rebuild(
        world,
        NavMeshBakeRequest::default(),
        NavMeshDirtyBounds::new([0.0, -1.0, -0.5], [1.0, 2.0, 0.5]),
    );
    assert!(started_at.elapsed() < Duration::from_millis(50));
    let dirty = wait_for_dirty_bake(&manager, task).unwrap();
    let rebuilt = dirty.rebuilt_tile_ids;
    let preserved = dirty.preserved_tile_ids;
    let asset = dirty.report.asset.unwrap();

    assert!(!rebuilt.is_empty());
    assert!(!preserved.is_empty());
    assert!(rebuilt.len() <= 9);
    assert!(preserved.iter().all(|id| !rebuilt.contains(id)));
    for tile_id in preserved {
        assert_eq!(
            tile_signature(&initial, tile_id),
            tile_signature(&asset, tile_id),
            "tile {tile_id} outside the dirty neighborhood changed"
        );
    }
}

#[test]
fn dirty_tile_rebuild_reconciles_vacated_and_new_tiles() {
    let manager = DefaultNavigationManager::new();
    let mut world = tiled_test_world(10);
    let initial = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap()
        .asset
        .unwrap();
    let moved = world.node_records()[10].id;
    world
        .update_transform(moved, Transform::from_translation(Vec3::new(6.5, 0.0, 0.0)))
        .unwrap();

    let task = manager.start_dirty_tile_rebuild(
        world,
        NavMeshBakeRequest::default(),
        NavMeshDirtyBounds::new([4.0, -1.0, -0.5], [7.1, 2.0, 0.5]),
    );
    let dirty = wait_for_dirty_bake(&manager, task).unwrap();
    let asset = dirty.report.asset.unwrap();

    assert!(!dirty.preserved_tile_ids.is_empty());
    assert!(asset
        .tiles
        .iter()
        .any(|tile| tile.bounds_max[0] >= 7.0 && tile.polygon_count > 0));
    let vacated = initial
        .tiles
        .iter()
        .find(|tile| tile.bounds_min[0] >= 4.0)
        .unwrap();
    assert_eq!(
        asset
            .tiles
            .iter()
            .find(|tile| tile.id == vacated.id)
            .unwrap()
            .polygon_count,
        0
    );
}

#[test]
fn dirty_tile_rebuild_rejects_changed_bake_identity() {
    let manager = DefaultNavigationManager::new();
    let mut world = tiled_test_world(6);
    manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap();
    let surface = world.node_records()[0].id;
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({
                "volume_size": [8.0, 4.0, 4.0],
                "override_tile_size": 1,
                "generate_links": false
            }),
        )
        .unwrap();

    let task = manager.start_dirty_tile_rebuild(
        world,
        NavMeshBakeRequest::default(),
        NavMeshDirtyBounds::new([-1.0, -1.0, -1.0], [1.0, 2.0, 1.0]),
    );
    let error = wait_for_dirty_bake(&manager, task).unwrap_err();

    assert!(error.message.contains("identity changed"));
}

#[test]
fn dirty_tile_rebuild_can_empty_the_entire_previous_grid() {
    let manager = DefaultNavigationManager::new();
    let mut world = tiled_test_world(4);
    let initial = manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap()
        .asset
        .unwrap();
    let sources = world
        .node_records()
        .iter()
        .skip(1)
        .map(|record| record.id)
        .collect::<Vec<_>>();
    for source in sources {
        assert!(world.remove_entity(source));
    }
    let min_x = initial
        .tiles
        .iter()
        .map(|tile| tile.bounds_min[0])
        .fold(Real::INFINITY, Real::min);
    let max_x = initial
        .tiles
        .iter()
        .map(|tile| tile.bounds_max[0])
        .fold(Real::NEG_INFINITY, Real::max);

    let task = manager.start_dirty_tile_rebuild(
        world,
        NavMeshBakeRequest::default(),
        NavMeshDirtyBounds::new([min_x, -1.0, -1.0], [max_x, 2.0, 1.0]),
    );
    let dirty = wait_for_dirty_bake(&manager, task).unwrap();
    let asset = dirty.report.asset.unwrap();

    assert_eq!(dirty.preserved_tile_ids.len(), 0);
    assert_eq!(asset.polygons.len(), 0);
    assert!(asset.tiles.iter().all(|tile| tile.polygon_count == 0));
}

#[test]
fn successful_non_tiled_bake_clears_previous_tiled_snapshot() {
    let manager = DefaultNavigationManager::new();
    let mut world = tiled_test_world(4);
    manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap();
    let surface = world.node_records()[0].id;
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({"volume_size": [6.0, 4.0, 4.0]}),
        )
        .unwrap();
    manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap();
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({"volume_size": [6.0, 4.0, 4.0], "override_tile_size": 1}),
        )
        .unwrap();

    let task = manager.start_dirty_tile_rebuild(
        world,
        NavMeshBakeRequest::default(),
        NavMeshDirtyBounds::new([-1.0, -1.0, -1.0], [1.0, 2.0, 1.0]),
    );
    let error = wait_for_dirty_bake(&manager, task).unwrap_err();

    assert!(error.message.contains("previously completed tiled bake"));
}

pub(super) fn tiled_test_world(cube_count: usize) -> World {
    let mut world = World::empty();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let surface = world.spawn_node(NodeKind::Empty);
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({
                "volume_size": [cube_count as Real + 2.0, 4.0, 4.0],
                "override_tile_size": 1
            }),
        )
        .unwrap();
    let offset = cube_count as Real * 0.5;
    for index in 0..cube_count {
        let cube = world.spawn_node(NodeKind::Cube);
        world
            .update_transform(
                cube,
                Transform::from_translation(Vec3::new(index as Real - offset + 0.5, 0.0, 0.0)),
            )
            .unwrap();
    }
    world
}

pub(super) fn wait_for_tiled_bake(
    manager: &DefaultNavigationManager,
    task: crate::NavMeshBakeTaskHandle,
) -> zircon_runtime::core::framework::navigation::NavMeshBakeReport {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(result) = manager.try_harvest_tiled_bake(task) {
            return result.unwrap();
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for tiled bake"
        );
        std::thread::yield_now();
    }
}

pub(super) fn wait_for_dirty_bake(
    manager: &DefaultNavigationManager,
    task: crate::NavMeshBakeTaskHandle,
) -> Result<
    crate::NavMeshDirtyBakeReport,
    zircon_runtime::core::framework::navigation::NavigationError,
> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(result) = manager.try_harvest_dirty_tile_rebuild(task) {
            return result;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for dirty tiled bake"
        );
        std::thread::yield_now();
    }
}

fn tile_signature(
    asset: &zircon_runtime::core::framework::navigation::NavMeshAsset,
    tile_id: u32,
) -> Vec<([[u32; 3]; 3], u8)> {
    let mut signature = asset
        .debug_triangles()
        .into_iter()
        .filter(|triangle| triangle.tile == tile_id)
        .map(|triangle| {
            let mut vertices = triangle.vertices.map(|vertex| {
                [
                    vertex[0].to_bits(),
                    vertex[1].to_bits(),
                    vertex[2].to_bits(),
                ]
            });
            vertices.sort_unstable();
            (vertices, triangle.area)
        })
        .collect::<Vec<_>>();
    signature.sort_unstable();
    signature
}
