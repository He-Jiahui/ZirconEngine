use super::*;

#[test]
fn componentized_workbench_window_template_bridge_projects_viewport_and_component_surfaces() {
    let _guard = env_lock().lock().unwrap();
    let (bridge, _) = componentized_workbench_projection_fixture();

    assert_eq!(bridge.surface().tree.roots.len(), 1);
    let viewport_frame = bridge
        .control_frame(EditorWorkbenchTemplateControlIds::VIEWPORT)
        .expect("viewport control frame");
    assert!(viewport_frame.width > 0.0);
    assert!(viewport_frame.height > 0.0);
    assert_eq!(bridge.frames().viewport, viewport_frame);
    assert_eq!(
        bridge.layout_frames().document_region_frame,
        Some(viewport_frame)
    );
    assert_eq!(
        bridge.control_frame(EditorWorkbenchTemplateControlIds::VIEWPORT),
        Some(bridge.frames().viewport)
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id(EditorWorkbenchTemplateControlIds::VIEWPORT)
            .expect("viewport projection")
            .frame,
        bridge.frames().viewport
    );

    let primary = bridge
        .host_projection()
        .node_by_control_id(EditorWorkbenchTemplateControlIds::PRIMARY_BUTTON)
        .expect("primary button projection");
    assert_eq!(primary.component, "Button");
    assert_eq!(primary.text.as_deref(), Some("Primary"));
    assert!(primary.routes.iter().any(|route| {
        route.binding_id == "ComponentLab/Primary"
            && route.event_kind == UiEventKind::Click
            && route.route_id.is_some()
    }));
    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let viewport_surface = template_contract_node(&nodes, "WorkbenchViewportSurface");
    let viewport_surface_frame = bridge
        .control_frame("WorkbenchViewportSurface")
        .expect("viewport surface frame");
    assert_eq!(viewport_surface.frame.width, viewport_surface_frame.width);
    assert_eq!(viewport_surface.frame.height, viewport_surface_frame.height);
    let viewport_backdrop = template_contract_node(&nodes, "WorkbenchViewportBackdrop");
    assert_eq!(viewport_backdrop.frame.width, viewport_surface_frame.width);
    assert_eq!(
        viewport_backdrop.frame.height,
        viewport_surface_frame.height
    );
    let viewport_ceiling = template_contract_node(&nodes, "WorkbenchViewportCeiling");
    assert_eq!(viewport_ceiling.frame.width, bridge.frames().viewport.width);
    assert_eq!(viewport_ceiling.frame.height, 92.0);
    let viewport_back_wall = template_contract_node(&nodes, "WorkbenchViewportBackWall");
    assert_eq!(
        viewport_back_wall.frame.width,
        bridge.frames().viewport.width
    );
    assert_eq!(viewport_back_wall.frame.height, 180.0);
    let viewport_floor = template_contract_node(&nodes, "WorkbenchViewportFloor");
    assert_eq!(viewport_floor.frame.width, bridge.frames().viewport.width);
    assert_eq!(viewport_floor.frame.height, 226.0);
    let viewport_grate = template_contract_node(&nodes, "WorkbenchViewportFloorGrateRight");
    assert_eq!(viewport_grate.role.as_str(), "Mount");
    assert_eq!(viewport_grate.frame.width, 42.0);
    assert_eq!(viewport_grate.frame.height, 122.0);
    assert_eq!(viewport_grate.corner_radius, 1.0);
    assert_eq!(viewport_grate.border_width, 1.0);
    let viewport_handrail = template_contract_node(&nodes, "WorkbenchViewportHandrailLeft");
    assert_eq!(viewport_handrail.frame.width, 190.0);
    assert_eq!(viewport_handrail.frame.height, 4.0);
    let viewport_selection = template_contract_node(&nodes, "WorkbenchViewportSelectionTop");
    assert_eq!(viewport_selection.frame.height, 2.0);
    let viewport_axis_x = template_contract_node(&nodes, "WorkbenchViewportAxisX");
    assert_eq!(viewport_axis_x.frame.width, 96.0);
    assert_eq!(viewport_axis_x.frame.height, 4.0);
    let viewport_axis_y = template_contract_node(&nodes, "WorkbenchViewportAxisY");
    assert_eq!(viewport_axis_y.frame.width, 4.0);
    assert_eq!(viewport_axis_y.frame.height, 96.0);
    let viewport_axis_z = template_contract_node(&nodes, "WorkbenchViewportAxisZ");
    assert_eq!(viewport_axis_z.frame.width, 66.0);
    assert_eq!(viewport_axis_z.frame.height, 4.0);
    let viewport_gizmo = template_contract_node(&nodes, "WorkbenchViewportGizmoCenter");
    assert_eq!(viewport_gizmo.frame.width, 36.0);
    assert_eq!(viewport_gizmo.frame.height, 31.0);
    let viewport_lightwash = template_contract_node(&nodes, "WorkbenchViewportLightwashCenter");
    assert_eq!(viewport_lightwash.frame.width, 252.0);
    assert_eq!(viewport_lightwash.frame.height, 150.0);
    assert_eq!(viewport_lightwash.corner_radius, 76.0);
    let viewport_shadow = template_contract_node(&nodes, "WorkbenchViewportShadowTopBay");
    assert_eq!(viewport_shadow.frame.width, 270.0);
    assert_eq!(viewport_shadow.frame.height, 116.0);
    assert_eq!(viewport_shadow.corner_radius, 52.0);
    let viewport_reflection = template_contract_node(&nodes, "WorkbenchViewportFloorReflection");
    assert_eq!(viewport_reflection.frame.width, 118.0);
    assert_eq!(viewport_reflection.frame.height, 170.0);
    let viewport_wall_light = template_contract_node(&nodes, "WorkbenchViewportWallLightFarRight");
    assert_eq!(viewport_wall_light.frame.width, 56.0);
    assert_eq!(viewport_wall_light.frame.height, 8.0);
    let viewport_beacon = template_contract_node(&nodes, "WorkbenchViewportWallBeaconLeft");
    assert_eq!(viewport_beacon.frame.width, 8.0);
    assert_eq!(viewport_beacon.frame.height, 56.0);
    let viewport_grid = template_contract_node(&nodes, "WorkbenchViewportGridH2");
    assert_eq!(viewport_grid.frame.height, 1.0);
    let viewport_floor_panel = template_contract_node(&nodes, "WorkbenchViewportFloorPanel2");
    assert_eq!(viewport_floor_panel.frame.width, 180.0);
    assert_eq!(viewport_floor_panel.frame.height, 58.0);
    let viewport_floor_seam = template_contract_node(&nodes, "WorkbenchViewportFloorSeamRight");
    assert_eq!(viewport_floor_seam.frame.width, 2.0);
    assert_eq!(viewport_floor_seam.frame.height, 198.0);
    let viewport_stairs = template_contract_node(&nodes, "WorkbenchViewportSideLeftStairs");
    assert_eq!(viewport_stairs.frame.width, 110.0);
    assert_eq!(viewport_stairs.frame.height, 90.0);
    let viewport_wall_detail =
        template_contract_node(&nodes, "WorkbenchViewportWallDetailCenterLines");
    assert_eq!(viewport_wall_detail.frame.width, 138.0);
    assert_eq!(viewport_wall_detail.frame.height, 128.0);
    let viewport_back_door = template_contract_node(&nodes, "WorkbenchViewportBackDoor");
    assert_eq!(viewport_back_door.frame.width, 146.0);
    assert_eq!(viewport_back_door.frame.height, 92.0);
    let viewport_door_core = template_contract_node(&nodes, "WorkbenchViewportDoorCore");
    assert_eq!(viewport_door_core.frame.width, 58.0);
    assert_eq!(viewport_door_core.frame.height, 42.0);
    let viewport_column = template_contract_node(&nodes, "WorkbenchViewportWallColumnLeft");
    assert_eq!(viewport_column.frame.width, 32.0);
    assert_eq!(viewport_column.frame.height, 196.0);
    let viewport_cargo_inner = template_contract_node(&nodes, "WorkbenchViewportCargoRightInner");
    assert_eq!(viewport_cargo_inner.frame.width, 190.0);
    assert_eq!(viewport_cargo_inner.frame.height, 54.0);
    let viewport_prop_body = template_contract_node(&nodes, "WorkbenchViewportPropBody");
    assert_eq!(viewport_prop_body.frame.width, 112.0);
    assert_eq!(viewport_prop_body.frame.height, 74.0);
    let viewport_prop_top = template_contract_node(&nodes, "WorkbenchViewportPropTop");
    assert_eq!(viewport_prop_top.frame.width, 112.0);
    assert_eq!(viewport_prop_top.frame.height, 22.0);
    let viewport_gizmo_panel = template_contract_node(&nodes, "WorkbenchViewportGizmoPanel");
    assert_eq!(viewport_gizmo_panel.frame.width, 92.0);
    assert_eq!(viewport_gizmo_panel.frame.height, 86.0);
    let viewport_gizmo_x = template_contract_node(&nodes, "WorkbenchViewportGizmoX");
    let viewport_gizmo_y = template_contract_node(&nodes, "WorkbenchViewportGizmoY");
    let viewport_gizmo_z = template_contract_node(&nodes, "WorkbenchViewportGizmoZ");
    assert_eq!(
        viewport_gizmo_x.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(239, 73, 63)
    );
    assert_eq!(
        viewport_gizmo_y.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(88, 208, 94)
    );
    assert_eq!(
        viewport_gizmo_z.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(88, 208, 94)
    );
    let component_buttons = template_contract_node(&nodes, "WorkbenchComponentButtons");
    assert_eq!(component_buttons.frame.width, 204.0);
    let component_icon_buttons = template_contract_node(&nodes, "WorkbenchComponentIconButtons");
    assert_eq!(component_icon_buttons.frame.width, 210.0);
    for panel_id in [
        "WorkbenchComponentButtons",
        "WorkbenchComponentIconButtons",
        "WorkbenchComponentInputs",
        "WorkbenchComponentSelection",
        "WorkbenchComponentSliders",
        "WorkbenchComponentLabs",
        "WorkbenchComponentList",
        "WorkbenchComponentTable",
        "WorkbenchComponentFeedback",
    ] {
        let panel = template_contract_node(&nodes, panel_id);
        assert_eq!(panel.surface_variant.as_str(), "component-panel");
        assert_eq!(panel.corner_radius, 4.0);
        assert_eq!(panel.border_width, 1.0);
    }
    assert_eq!(
        slot_padding_for_control(&bridge, "WorkbenchButtonsTitle"),
        Some(UiMargin::new(8.0, 4.0, 8.0, 0.0))
    );
    assert_eq!(
        slot_padding_for_control(&bridge, "WorkbenchButtonsRowTail"),
        Some(UiMargin::new(8.0, 0.0, 8.0, 4.0))
    );
    assert_eq!(
        slot_padding_for_control(&bridge, "WorkbenchFeedbackAlerts"),
        Some(UiMargin::new(8.0, 4.0, 0.0, 4.0))
    );
    let component_top_row = template_contract_node(&nodes, "WorkbenchComponentTopRow");
    let component_lower_row = template_contract_node(&nodes, "WorkbenchComponentLowerRow");
    assert_eq!(component_top_row.frame.height, 202.0);
    assert!(
        component_lower_row.frame.y > component_top_row.frame.y + component_top_row.frame.height
    );
}
