use crate::asset::NavMeshAsset;

use super::*;

#[test]
fn defaults_match_humanoid_agent_contract() {
    let agent = NavigationAgentSettings::humanoid();
    assert_eq!(agent.id, "humanoid");
    assert_eq!(agent.radius, 0.5);
    assert_eq!(agent.height, 2.0);
    assert_eq!(agent.max_climb, 0.4);
    assert_eq!(agent.max_slope_degrees, 45.0);
    assert_eq!(agent.speed, 3.5);
    assert_eq!(agent.acceleration, 8.0);
    assert_eq!(agent.angular_speed_degrees, 360.0);
    assert_eq!(agent.stopping_distance, 0.1);
}

#[test]
fn component_type_ids_are_plugin_prefixed() {
    for type_id in [
        NAV_MESH_SURFACE_COMPONENT_TYPE,
        NAV_MESH_MODIFIER_COMPONENT_TYPE,
        NAV_MESH_AGENT_COMPONENT_TYPE,
        NAV_MESH_OBSTACLE_COMPONENT_TYPE,
        NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE,
        NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    ] {
        assert!(type_id.starts_with("navigation.Component."));
    }
}

#[test]
fn off_mesh_bridge_descriptor_is_a_first_class_navigation_contract() {
    let descriptor = NavMeshOffMeshBridgeDescriptor::default();

    assert_eq!(descriptor.start_local_point, [-0.5, 0.0, 0.0]);
    assert_eq!(descriptor.end_local_point, [0.5, 0.0, 0.0]);
    assert_eq!(descriptor.width, 1.0);
    assert_eq!(descriptor.lane_count, 1);
    assert_eq!(descriptor.area_type, AREA_JUMP);
    assert_eq!(descriptor.agent_type, DEFAULT_AGENT_TYPE);
    assert_eq!(descriptor.traversal_mode, NavLinkTraversalMode::Automatic);

    let json = serde_json::to_value(&descriptor).unwrap();
    assert_eq!(json["traversal_mode"], "automatic");
    assert_eq!(json["lane_count"], 1);
    assert_eq!(
        serde_json::from_value::<NavMeshOffMeshBridgeDescriptor>(json).unwrap(),
        descriptor
    );
}

#[test]
fn nav_mesh_asset_gizmo_snapshot_projects_triangle_edges() {
    let snapshot = NavigationGizmoSnapshot::from_nav_mesh_asset(&NavMeshAsset::simple_quad(
        DEFAULT_AGENT_TYPE,
        2.0,
    ));
    let overlay = snapshot.to_scene_gizmo_overlay(42, true);

    assert_eq!(snapshot.triangles.len(), 2);
    assert_eq!(overlay.owner, 42);
    assert_eq!(overlay.lines.len(), 6);
    assert!(overlay.selected);
}
