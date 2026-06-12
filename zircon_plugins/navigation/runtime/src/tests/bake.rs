use serde_json::json;
use zircon_runtime::asset::NavigationSettingsAsset;
use zircon_runtime::core::framework::navigation::{
    NavMeshBakeRequest, NavigationAreaSettings, NavigationManager, AREA_JUMP,
    NAV_MESH_MODIFIER_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
    NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Real, Transform, Vec3};
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::world::World;

use crate::{navigation_component_descriptors, DefaultNavigationManager};

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
