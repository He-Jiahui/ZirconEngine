use crate::asset::NavMeshAsset;
use crate::core::math::Real;

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
    assert_eq!(descriptor.motion, NavLinkMotion::Linear);

    let json = serde_json::to_value(&descriptor).unwrap();
    assert_eq!(json["traversal_mode"], "automatic");
    assert_eq!(json["lane_count"], 1);
    assert_eq!(
        serde_json::from_value::<NavMeshOffMeshBridgeDescriptor>(json).unwrap(),
        descriptor
    );
}

#[test]
fn off_mesh_traverse_state_and_event_are_serializable_contracts() {
    let state = OffMeshTraverseState {
        agent_entity: 7,
        nav_mesh: NavMeshHandle(3),
        link_id: 11,
        owner_entity: 19,
        phase: OffMeshTraversePhase::Traverse,
        progress: 0.5,
        start: [1.0, 0.0, 0.0],
        end: [4.0, 0.0, 0.0],
    };
    let event = OffMeshTraverseEvent::started(&state);

    assert_eq!(event.kind, OffMeshTraverseEventKind::Started);
    assert_eq!(event.link_id, state.link_id);
    assert_eq!(
        serde_json::from_value::<OffMeshTraverseState>(serde_json::to_value(&state).unwrap())
            .unwrap(),
        state
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

#[test]
fn query_filter_carries_area_cost_and_detour_flags() {
    let filter = NavQueryFilter::default().with_area_cost(AREA_JUMP, 7.5);
    assert_eq!(filter.area_costs[AREA_JUMP as usize], 7.5);
    assert!(filter.allows_area(AREA_JUMP));

    let excluded = NavQueryFilter {
        exclude_flags: nav_area_flag(AREA_JUMP),
        ..filter.clone()
    };
    assert!(!excluded.allows_area(AREA_JUMP));
}

#[test]
fn query_filter_serde_round_trip_preserves_all_64_area_costs() {
    let filter = NavQueryFilter {
        area_costs: std::array::from_fn(|index| index as Real + 1.0),
        include_flags: 0x1234,
        exclude_flags: 0x0040,
    };

    let json = serde_json::to_value(&filter).unwrap();
    assert_eq!(json["area_costs"].as_array().unwrap().len(), MAX_NAV_AREAS);
    assert_eq!(
        serde_json::from_value::<NavQueryFilter>(json).unwrap(),
        filter
    );
}

#[test]
fn query_filter_serde_rejects_area_cost_arrays_with_wrong_length() {
    for length in [MAX_NAV_AREAS - 1, MAX_NAV_AREAS + 1] {
        let json = serde_json::json!({
            "area_costs": vec![1.0; length],
            "include_flags": u16::MAX,
            "exclude_flags": 0,
        });

        assert!(serde_json::from_value::<NavQueryFilter>(json).is_err());
    }
}

#[test]
fn query_filter_serde_rejects_non_finite_or_non_positive_area_costs() {
    for invalid in [0.0, -1.0, Real::INFINITY, Real::NEG_INFINITY, Real::NAN] {
        let mut area_costs = vec![1.0; MAX_NAV_AREAS];
        area_costs[AREA_JUMP as usize] = invalid;
        let json = serde_json::json!({
            "area_costs": area_costs,
            "include_flags": u16::MAX,
            "exclude_flags": 0,
        });

        assert!(serde_json::from_value::<NavQueryFilter>(json).is_err());
    }
}
