use super::super::support::*;
use crate::ui::retained_host::callback_dispatch::load_startup_builtin_template_runtime;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::workbench_popup_actions::WORKBENCH_POPUP_CANCEL_ACTION_ID;
use crate::ui::retained_host::HostInvalidationMask;
use crate::ui::retained_host::{
    to_host_contract_workbench_window_nodes, TemplatePaneMenuItemData, TemplatePaneNodeData,
    TemplatePaneOptionData,
};
use crate::ui::workbench::autolayout::{compact_bottom_height_limit, WorkbenchChromeMetrics};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::reference::EditorWorkbenchTemplateControlIds;
use crate::ui::workbench::snapshot::{
    InspectorPluginComponentPropertySnapshot, InspectorPluginComponentSnapshot, InspectorSnapshot,
    SceneEntry,
};
use std::sync::Arc;
use zircon_runtime_interface::ui::style::UiStyleColor;
use zircon_runtime_interface::ui::tree::UiVisibility;
use zircon_runtime_interface::ui::v2::{
    UI_V2_REPEAT_ATTRIBUTE, UI_V2_REPEAT_FIELD_AUTHORED_COUNT, UI_V2_REPEAT_FIELD_KIND,
    UI_V2_REPEAT_FIELD_NODE_PATH_NAMESPACE, UI_V2_REPEAT_FIELD_PROTOTYPE,
    UI_V2_REPEAT_FIELD_VIRTUAL_CONTROL_PREFIX, UI_V2_REPEAT_KIND_VIRTUAL_ROWS,
};

#[test]
fn builtin_host_window_template_bridge_recomputes_surface_backed_frames_with_shell_size() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let initial = bridge
        .host_projection()
        .node_by_control_id("DocumentHostRoot")
        .expect("document host control should exist")
        .frame;
    assert_eq!(initial, UiFrame::new(44.0, 57.0, 1236.0, 639.0));

    bridge.recompute_layout(UiSize::new(960.0, 540.0)).unwrap();

    let recomputed = bridge
        .host_projection()
        .node_by_control_id("DocumentHostRoot")
        .expect("document host control should exist after recompute")
        .frame;
    assert_eq!(recomputed, UiFrame::new(44.0, 57.0, 916.0, 459.0));

    assert_eq!(
        bridge.control_frame("PaneSurfaceRoot"),
        Some(UiFrame::new(44.0, 89.0, 916.0, 427.0))
    );

    let root_frames = bridge.root_shell_frames();
    assert_eq!(
        root_frames.shell_frame,
        Some(UiFrame::new(0.0, 0.0, 960.0, 540.0))
    );
    assert_eq!(
        root_frames.menu_bar_frame,
        Some(UiFrame::new(0.0, 0.0, 960.0, 24.0))
    );
    assert_eq!(
        root_frames.activity_rail_frame,
        Some(UiFrame::new(0.0, 57.0, 44.0, 459.0))
    );
    assert_eq!(
        root_frames.host_page_strip_frame,
        Some(UiFrame::new(0.0, 24.0, 960.0, 32.0))
    );
    assert_eq!(
        root_frames.host_body_frame,
        Some(UiFrame::new(0.0, 57.0, 960.0, 459.0))
    );
    assert_eq!(
        root_frames.document_host_frame,
        Some(UiFrame::new(44.0, 57.0, 916.0, 459.0))
    );
    assert_eq!(
        root_frames.document_tabs_frame,
        Some(UiFrame::new(44.0, 57.0, 916.0, 32.0))
    );
    assert_eq!(
        root_frames.pane_surface_frame,
        Some(UiFrame::new(44.0, 89.0, 916.0, 427.0))
    );
    assert_eq!(
        root_frames.status_bar_frame,
        Some(UiFrame::new(0.0, 516.0, 960.0, 24.0))
    );
}

#[test]
fn builtin_host_window_template_bridge_exports_visible_drawer_shell_and_header_frames_from_workbench_model(
) {
    let _guard = env_lock().lock().unwrap();

    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let mut bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            UiSize::new(1280.0, 720.0),
            &model,
            &WorkbenchChromeMetrics::default(),
        )
        .unwrap();

    let root_frames = bridge.root_shell_frames();
    let body_frame = root_frames
        .host_body_frame
        .expect("workbench body projection frame should exist");
    let metrics = WorkbenchChromeMetrics::default();
    let requested_bottom_height = 164.0_f32;
    let expected_bottom_height =
        compact_bottom_height_limit((body_frame.height - metrics.separator_thickness).max(0.0))
            .map(|limit| requested_bottom_height.min(limit))
            .unwrap_or(requested_bottom_height);
    let expected_bottom_height = round_to_layout_pixel(expected_bottom_height);
    let expected_center_height = round_to_layout_pixel(
        body_frame.height - expected_bottom_height - metrics.separator_thickness,
    );
    let expected_bottom_y =
        round_to_layout_pixel(body_frame.y + body_frame.height - expected_bottom_height);
    let expected_bottom_content_height = round_to_layout_pixel(
        (expected_bottom_height - metrics.panel_header_height - metrics.separator_thickness)
            .max(0.0),
    );
    assert_eq!(
        root_frames.left_drawer_shell_frame,
        Some(UiFrame::new(
            body_frame.x,
            body_frame.y,
            312.0,
            expected_center_height
        ))
    );
    assert_eq!(
        root_frames.left_drawer_header_frame,
        Some(UiFrame::new(body_frame.x + 35.0, body_frame.y, 277.0, 25.0))
    );
    assert_eq!(
        root_frames.left_drawer_content_frame,
        Some(UiFrame::new(
            body_frame.x + 35.0,
            body_frame.y + 25.0,
            277.0,
            expected_center_height - 25.0,
        ))
    );
    assert_eq!(
        root_frames.right_drawer_shell_frame,
        Some(UiFrame::new(
            body_frame.x + body_frame.width - 308.0,
            body_frame.y,
            308.0,
            expected_center_height,
        ))
    );
    assert_eq!(
        root_frames.right_drawer_header_frame,
        Some(UiFrame::new(
            body_frame.x + body_frame.width - 308.0,
            body_frame.y,
            273.0,
            25.0,
        ))
    );
    assert_eq!(
        root_frames.right_drawer_content_frame,
        Some(UiFrame::new(
            body_frame.x + body_frame.width - 308.0,
            body_frame.y + 25.0,
            273.0,
            expected_center_height - 25.0,
        ))
    );
    assert_eq!(
        root_frames.bottom_drawer_shell_frame,
        Some(UiFrame::new(
            body_frame.x,
            expected_bottom_y,
            body_frame.width,
            expected_bottom_height,
        ))
    );
    assert_eq!(
        root_frames.bottom_drawer_header_frame,
        Some(UiFrame::new(
            body_frame.x,
            expected_bottom_y,
            body_frame.width,
            25.0,
        ))
    );
    assert_eq!(
        root_frames.bottom_drawer_content_frame,
        Some(UiFrame::new(
            body_frame.x,
            expected_bottom_y + metrics.panel_header_height + metrics.separator_thickness,
            body_frame.width,
            expected_bottom_content_height,
        ))
    );
}

#[test]
fn componentized_workbench_window_template_bridge_exports_surface_projection_frames_and_routes() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert_eq!(bridge.surface().tree.roots.len(), 1);
    assert_eq!(
        bridge.frames().viewport,
        UiFrame::new(404.0, 60.0, 864.0, 428.0)
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
    let component_top_row = template_contract_node(&nodes, "WorkbenchComponentTopRow");
    let component_lower_row = template_contract_node(&nodes, "WorkbenchComponentLowerRow");
    assert_eq!(component_top_row.frame.height, 202.0);
    assert!(
        component_lower_row.frame.y > component_top_row.frame.y + component_top_row.frame.height
    );
    let primary_button = template_contract_node(&nodes, "WorkbenchPrimaryButton");
    let secondary_button = template_contract_node(&nodes, "WorkbenchSecondaryButton");
    let tertiary_button = template_contract_node(&nodes, "WorkbenchTertiaryButton");
    let outline_button = template_contract_node(&nodes, "WorkbenchOutlineButton");
    let button_icon = template_contract_node(&nodes, "WorkbenchButtonIcon");
    let button_delete = template_contract_node(&nodes, "WorkbenchButtonDelete");
    let disabled_button = template_contract_node(&nodes, "WorkbenchDisabledButton");
    assert!((primary_button.label_brightness - 1.0).abs() < 0.001);
    assert!((secondary_button.label_brightness - 1.01).abs() < 0.001);
    assert_eq!(primary_button.layout_offset_x, 3.0);
    assert_eq!(primary_button.layout_offset_y, -1.0);
    assert_eq!(primary_button.font_size, 12.22);
    assert_eq!(
        style_color_u8(
            primary_button
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        Some([53, 199, 208, 255])
    );
    assert_eq!(
        style_color_u8(primary_button.button_style.element.border_color.as_ref()),
        Some([53, 199, 208, 255])
    );
    assert_eq!(secondary_button.layout_offset_x, 1.0);
    assert_eq!(secondary_button.layout_offset_y, -1.0);
    assert_eq!(secondary_button.font_size, 12.22);
    assert_eq!(
        style_color_u8(
            secondary_button
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        Some([29, 35, 40, 255])
    );
    assert_eq!(tertiary_button.role.as_str(), "Button");
    assert_eq!(tertiary_button.text.as_str(), "Tertiary");
    assert_eq!(tertiary_button.button_variant.as_str(), "text");
    assert_eq!(tertiary_button.layout_offset_x, 1.0);
    assert_eq!(tertiary_button.corner_radius, 9.0);
    assert_eq!(outline_button.text.as_str(), "Outline");
    assert_eq!(outline_button.layout_offset_x, 1.0);
    assert_eq!(outline_button.corner_radius, 9.0);
    assert_eq!(
        style_color_u8(
            tertiary_button
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        Some([29, 35, 40, 255])
    );
    assert_eq!(
        style_color_u8(tertiary_button.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    assert_eq!(
        style_color_u8(
            tertiary_button
                .button_style
                .element
                .foreground_color
                .as_ref()
        ),
        Some([216, 227, 231, 255])
    );
    assert_eq!(
        style_color_u8(outline_button.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    assert_eq!(
        style_color_u8(
            outline_button
                .button_style
                .element
                .foreground_color
                .as_ref()
        ),
        Some([216, 227, 231, 255])
    );
    assert_eq!(button_icon.text.as_str(), "Icon");
    assert_eq!(button_icon.icon_name.as_str(), "plus");
    assert_eq!(button_icon.layout_offset_x, 3.0);
    assert_eq!(button_icon.layout_offset_y, 1.0);
    assert!((button_icon.label_brightness - 1.02).abs() < 0.001);
    assert_eq!(button_icon.corner_radius, 9.0);
    assert_eq!(
        style_color_u8(button_icon.button_style.element.background_color.as_ref()),
        Some([29, 35, 40, 255])
    );
    assert_eq!(
        style_color_u8(button_icon.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    assert_eq!(
        style_color_u8(button_icon.button_style.element.foreground_color.as_ref()),
        Some([216, 227, 231, 255])
    );
    assert_eq!(button_delete.icon_name.as_str(), "trash");
    assert_eq!(button_delete.validation_level.as_str(), "danger");
    assert_eq!(button_delete.corner_radius, 9.0);
    assert!((button_delete.label_brightness - 1.02).abs() < 0.001);
    assert_eq!(
        style_color_u8(button_delete.button_style.element.foreground_color.as_ref()),
        Some([216, 227, 231, 255])
    );
    assert!(disabled_button.disabled);
    assert_eq!(disabled_button.layout_offset_x, -1.0);
    assert_eq!(disabled_button.layout_offset_y, 3.5);
    assert_eq!(
        style_color_u8(
            disabled_button
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        Some([39, 44, 48, 255])
    );
    assert_eq!(
        style_color_u8(disabled_button.button_style.element.border_color.as_ref()),
        Some([52, 61, 68, 255])
    );
    assert_eq!(
        style_color_u8(
            disabled_button
                .button_style
                .element
                .foreground_color
                .as_ref()
        ),
        Some([216, 227, 231, 255])
    );
    assert!((disabled_button.button_style.element.opacity - 0.72).abs() < 0.001);
    let dropdown_button = template_contract_node(&nodes, "WorkbenchDropdownButton");
    assert_eq!(dropdown_button.role.as_str(), "Dropdown");
    assert_eq!(dropdown_button.value_text.as_str(), "Dropdown");
    assert_eq!(dropdown_button.options.row_count(), 3);
    assert_eq!(dropdown_button.layout_offset_x, -1.0);
    assert!((dropdown_button.label_brightness - 1.005).abs() < 0.001);
    assert_eq!(dropdown_button.layout_offset_y, 3.5);
    assert_eq!(
        dropdown_button.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(127, 138, 145)
    );
    assert_eq!(
        dropdown_button.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(103, 115, 122)
    );
    assert_eq!(
        style_color_u8(dropdown_button.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    let mini_add = template_contract_node(&nodes, "WorkbenchMiniAdd");
    assert_eq!(
        style_color_u8(mini_add.button_style.element.background_color.as_ref()),
        Some([23, 28, 32, 255])
    );
    assert_eq!(
        style_color_u8(mini_add.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    assert_eq!(mini_add.corner_radius, 10.0);
    let mini_eye = template_contract_node(&nodes, "WorkbenchMiniEye");
    let mini_eye_off = template_contract_node(&nodes, "WorkbenchMiniEyeOff");
    let mini_lock = template_contract_node(&nodes, "WorkbenchMiniLock");
    let mini_more = template_contract_node(&nodes, "WorkbenchMiniMore");
    let mini_delete = template_contract_node(&nodes, "WorkbenchMiniDelete");
    assert_eq!(mini_eye.role.as_str(), "IconButton");
    assert_eq!(mini_eye.icon_name.as_str(), "eye");
    assert_eq!(mini_eye.value_number, 18.0);
    assert!((mini_eye.layout_offset_y - 1.35).abs() < 0.001);
    assert_eq!(mini_eye.frame.width, 38.0);
    assert_eq!(
        mini_eye.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(152, 163, 168)
    );
    assert_eq!(
        style_color_u8(mini_eye.button_style.element.background_color.as_ref()),
        Some([23, 28, 32, 255])
    );
    assert_eq!(
        style_color_u8(mini_eye.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    assert_eq!(mini_eye.corner_radius, 10.0);
    assert_eq!(mini_eye_off.icon_name.as_str(), "eye-off");
    assert_eq!(mini_lock.icon_name.as_str(), "lock");
    assert_eq!(mini_more.icon_name.as_str(), "more");
    assert_eq!(mini_delete.corner_radius, 10.0);
    assert_eq!(
        style_color_u8(mini_delete.button_style.element.border_color.as_ref()),
        Some([236, 111, 98, 255])
    );
    let icon_toggle = template_contract_node(&nodes, "WorkbenchIconToggleSegmented");
    assert_eq!(icon_toggle.value_text.as_str(), "grid");
    assert_eq!(icon_toggle.options.row_count(), 3);
    assert_eq!(icon_toggle.options.row_data(0).as_deref(), Some("grid"));
    assert_eq!(icon_toggle.options.row_data(1).as_deref(), Some("list"));
    assert_eq!(icon_toggle.options.row_data(2).as_deref(), Some("columns"));
    assert_eq!(icon_toggle.layout_offset_y, 1.0);
    assert!(icon_toggle.has_selected_segment_border_width);
    assert_eq!(icon_toggle.selected_segment_border_width, 0.0);
    assert_eq!(icon_toggle.selected_segment_underline_height, 1.0);
    assert_eq!(
        icon_toggle.selected_segment_underline_color,
        crate::ui::retained_host::primitives::Color::from_argb_u8(122, 50, 211, 222)
    );
    let component_inputs = template_contract_node(&nodes, "WorkbenchComponentInputs");
    let component_selection = template_contract_node(&nodes, "WorkbenchComponentSelection");
    let component_sliders = template_contract_node(&nodes, "WorkbenchComponentSliders");
    let component_labs = template_contract_node(&nodes, "WorkbenchComponentLabs");
    let labs_tabs = template_contract_node(&nodes, "WorkbenchLabsTabs");
    let component_list = template_contract_node(&nodes, "WorkbenchComponentList");
    assert_eq!(component_inputs.frame.width, 214.0);
    assert_eq!(component_selection.frame.width, 168.0);
    assert_eq!(component_sliders.frame.width, 260.0);
    assert_eq!(component_labs.frame.width, 236.0);
    assert_eq!(labs_tabs.frame.width, 216.0);
    assert_eq!(
        style_color_u8(labs_tabs.button_style.element.background_color.as_ref()),
        Some([20, 25, 29, 255])
    );
    assert!(component_selection.frame.x > component_inputs.frame.x + component_inputs.frame.width);
    assert!(
        component_sliders.frame.x > component_selection.frame.x + component_selection.frame.width
    );
    assert!(component_labs.frame.x > component_sliders.frame.x + component_sliders.frame.width);
    assert!(component_list.frame.x > component_labs.frame.x + component_labs.frame.width);
    assert_eq!(component_selection.layout_content_offset_x, 9.0);
    let checkbox_on = template_contract_node(&nodes, "WorkbenchCheckboxOn");
    assert_eq!(checkbox_on.layout_icon_size, 16.0);
    assert_eq!(checkbox_on.layout_content_offset_x, 9.0);
    assert_eq!(
        checkbox_on.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(130, 140, 147)
    );
    assert_eq!(
        style_color_u8(checkbox_on.button_style.element.background_color.as_ref()),
        Some([32, 159, 168, 255])
    );
    assert_eq!(
        style_color_u8(checkbox_on.button_style.element.border_color.as_ref()),
        Some([32, 159, 168, 255])
    );
    let checkbox_off = template_contract_node(&nodes, "WorkbenchCheckboxOff");
    assert_eq!(checkbox_off.layout_icon_size, 16.0);
    assert_eq!(checkbox_off.layout_content_offset_x, 9.0);
    assert_eq!(
        checkbox_off.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(130, 140, 147)
    );
    assert_eq!(
        style_color_u8(checkbox_off.button_style.element.background_color.as_ref()),
        Some([20, 26, 30, 255])
    );
    assert_eq!(
        style_color_u8(checkbox_off.button_style.element.border_color.as_ref()),
        Some([66, 78, 86, 255])
    );
    let radio_on = template_contract_node(&nodes, "WorkbenchRadioOn");
    assert_eq!(radio_on.layout_icon_size, 16.0);
    assert_eq!(radio_on.layout_content_offset_x, 9.0);
    assert_eq!(radio_on.value_number, 7.0);
    assert_eq!(
        radio_on.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(130, 140, 147)
    );
    assert_eq!(
        radio_on.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(67, 216, 226)
    );
    assert_eq!(
        style_color_u8(radio_on.button_style.element.background_color.as_ref()),
        Some([27, 39, 45, 255])
    );
    assert_eq!(
        style_color_u8(radio_on.button_style.element.border_color.as_ref()),
        Some([76, 91, 99, 255])
    );
    let radio_off = template_contract_node(&nodes, "WorkbenchRadioOff");
    assert_eq!(radio_off.layout_icon_size, 16.0);
    assert_eq!(radio_off.layout_content_offset_x, 9.0);
    assert_eq!(radio_off.value_number, 7.0);
    assert_eq!(
        radio_off.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(130, 140, 147)
    );
    assert_eq!(
        radio_off.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(67, 216, 226)
    );
    assert_eq!(
        style_color_u8(radio_off.button_style.element.background_color.as_ref()),
        Some([20, 26, 30, 255])
    );
    assert_eq!(
        style_color_u8(radio_off.button_style.element.border_color.as_ref()),
        Some([66, 78, 86, 255])
    );
    let toggle = template_contract_node(&nodes, "WorkbenchToggleOn");
    assert_eq!(toggle.value_number, 34.0);
    assert_eq!(toggle.layout_icon_size, 14.0);
    assert_eq!(toggle.layout_content_offset_x, 10.0);
    assert_eq!(toggle.layout_content_offset_y, 18.0);
    assert_eq!(
        style_color_u8(toggle.button_style.element.background_color.as_ref()),
        Some([53, 199, 208, 255])
    );
    assert_eq!(
        style_color_u8(toggle.button_style.element.foreground_color.as_ref()),
        Some([255, 255, 255, 255])
    );
    assert_eq!(
        style_color_u8(toggle.button_style.element.border_color.as_ref()),
        Some([49, 191, 201, 255])
    );
    assert!(toggle.frame.x >= component_labs.frame.x);
    assert!(
        toggle.frame.x + toggle.frame.width
            <= component_labs.frame.x + component_labs.frame.width + 0.001
    );
    let component_table = template_contract_node(&nodes, "WorkbenchComponentTable");
    let component_feedback = template_contract_node(&nodes, "WorkbenchComponentFeedback");
    assert_eq!(component_table.frame.width, 590.0);
    assert!(component_table.frame.y >= component_lower_row.frame.y);
    assert!(component_feedback.frame.y >= component_lower_row.frame.y);
    let feedback_alerts = template_contract_node(&nodes, "WorkbenchFeedbackAlerts");
    let feedback_toast_column = template_contract_node(&nodes, "WorkbenchFeedbackToastColumn");
    assert_eq!(feedback_alerts.frame.width, 390.0);
    assert_eq!(feedback_toast_column.frame.width, 390.0);
    assert!(feedback_alerts.frame.x >= component_feedback.frame.x);
    assert!(
        feedback_alerts.frame.x + feedback_alerts.frame.width
            <= component_feedback.frame.x + component_feedback.frame.width + 0.001
    );
    let info_alert = template_contract_node(&nodes, "WorkbenchInfoAlert");
    let success_alert = template_contract_node(&nodes, "WorkbenchSuccessAlert");
    let warning_alert = template_contract_node(&nodes, "WorkbenchWarningAlert");
    let error_alert = template_contract_node(&nodes, "WorkbenchErrorAlert");
    assert!(info_alert.frame.x >= feedback_alerts.frame.x);
    assert!(
        info_alert.frame.x + info_alert.frame.width
            <= feedback_alerts.frame.x + feedback_alerts.frame.width + 0.001
    );
    assert!(
        (success_alert.frame.y - (info_alert.frame.y + info_alert.frame.height + 6.0)).abs()
            < 0.001
    );
    assert!(
        (warning_alert.frame.y - (success_alert.frame.y + success_alert.frame.height + 6.0)).abs()
            < 0.001
    );
    assert!(
        (error_alert.frame.y - (warning_alert.frame.y + warning_alert.frame.height + 6.0)).abs()
            < 0.001
    );
    let feedback_tooltip = template_contract_node(&nodes, "WorkbenchTooltipRoot");
    let standalone_toast = template_contract_node(&nodes, "WorkbenchToastRoot");
    assert!(feedback_tooltip.frame.x > feedback_alerts.frame.x + feedback_alerts.frame.width);
    assert!(
        feedback_toast_column.frame.x > feedback_tooltip.frame.x + feedback_tooltip.frame.width
    );
    assert!(standalone_toast.frame.x >= feedback_toast_column.frame.x);
    assert!(
        standalone_toast.frame.x + standalone_toast.frame.width
            <= feedback_toast_column.frame.x + feedback_toast_column.frame.width + 0.001
    );
    assert!(standalone_toast.frame.y > feedback_tooltip.frame.y);
    assert!(
        standalone_toast.frame.y + standalone_toast.frame.height
            <= component_feedback.frame.y + component_feedback.frame.height + 0.001
    );
    let table_item = template_contract_node(&nodes, "WorkbenchTableItem");
    assert_eq!(table_item.role.as_str(), "Table");
    assert_eq!(table_item.component_role.as_str(), "table");
    assert_eq!(table_item.options.row_count(), 4);
    assert_eq!(table_item.options.row_data(0).as_deref(), Some("Item_01"));
    assert_eq!(table_item.options.row_data(1).as_deref(), Some("Mesh"));
    assert_eq!(table_item.options.row_data(2).as_deref(), Some("2.4 MB"));
    assert_eq!(table_item.options.row_data(3).as_deref(), Some("2m ago"));
    assert_eq!(table_item.layout_first_cell_offset_x, 4.0);
    assert!(!table_item.selected);
    let table = template_contract_node(&nodes, "WorkbenchTableSelected");
    assert_eq!(table.role.as_str(), "Table");
    assert_eq!(table.component_role.as_str(), "table");
    assert_eq!(table.options.row_count(), 4);
    assert_eq!(table.layout_offset_x, -1.0);
    assert_eq!(table.layout_offset_y, -1.5);
    assert_eq!(table.options.row_data(0).as_deref(), Some("Item_02"));
    assert_eq!(table.options.row_data(1).as_deref(), Some("Material"));
    assert_eq!(table.options.row_data(2).as_deref(), Some("512 KB"));
    assert_eq!(table.options.row_data(3).as_deref(), Some("10m ago"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchTableSelected").as_deref(),
        Some("#12383d")
    );
    let table_header = template_contract_node(&nodes, "WorkbenchTableHeader");
    assert!(table_header.frame.x >= component_table.frame.x);
    assert!(
        table_header.frame.x + table_header.frame.width
            <= component_table.frame.x + component_table.frame.width + 0.001
    );
    assert_eq!(table_header.layout_content_offset_x, -1.0);
    assert_eq!(table_header.layout_content_offset_y, 3.0);
    assert_eq!(table_header.layout_first_cell_offset_x, 0.0);
    assert!(
        (table_item.frame.y - (table_header.frame.y + table_header.frame.height)).abs() < 0.001
    );
    assert!((table.frame.y - (table_item.frame.y + table_item.frame.height)).abs() < 0.001);
    assert_eq!(table.layout_first_cell_offset_x, 0.0);
    let table_tail = template_contract_node(&nodes, "WorkbenchTableTail");
    assert!((table_tail.frame.y - (table.frame.y + table.frame.height)).abs() < 0.001);
    assert_eq!(table_tail.layout_content_offset_y, -0.5);
    assert_eq!(table_tail.layout_first_cell_offset_x, 6.0);
    assert_eq!(table_tail.layout_second_cell_offset_x, 2.0);
    assert_eq!(table_tail.layout_fourth_cell_offset_x, -2.0);
    assert_eq!(
        table_tail.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(170, 181, 186)
    );
    let segmented = template_contract_node(&nodes, "WorkbenchInputSegmented");
    assert_eq!(segmented.label_text.as_str(), "Segmented Control");
    assert_eq!(
        segmented.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(161, 172, 178)
    );
    assert!((segmented.label_brightness - 0.94).abs() < 0.001);
    assert_eq!(segmented.layout_offset_x, -0.5);
    assert_eq!(segmented.frame.height, 48.0);
    let labs_tabs = template_contract_node(&nodes, "WorkbenchLabsTabs");
    assert!(segmented.frame.x >= component_labs.frame.x);
    assert!(
        segmented.frame.x + segmented.frame.width
            <= component_labs.frame.x + component_labs.frame.width + 0.001
    );
    assert!(segmented.frame.y > labs_tabs.frame.y + labs_tabs.frame.height);
    assert!(toggle.frame.y > segmented.frame.y + segmented.frame.height);
    let input_slider = template_contract_node(&nodes, "WorkbenchInputSlider");
    assert_eq!(input_slider.label_text.as_str(), "Value");
    assert_eq!(input_slider.value_text.as_str(), "0.75");
    assert_eq!(input_slider.layout_offset_x, -18.0);
    assert_eq!(input_slider.layout_offset_y, 1.0);
    assert_eq!(input_slider.layout_icon_size, 9.0);
    assert_eq!(input_slider.layout_content_offset_x, -10.0);
    assert_eq!(input_slider.layout_first_cell_offset_x, 18.0);
    assert_eq!(
        input_slider.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(183, 241, 248)
    );
    assert_eq!(
        style_color_u8(input_slider.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    assert_eq!(
        input_slider.state_layer_color,
        crate::ui::retained_host::primitives::Color::from_argb_u8(61, 50, 211, 222)
    );
    assert!(input_slider.frame.x >= component_sliders.frame.x);
    assert!(
        input_slider.frame.x + input_slider.frame.width
            <= component_sliders.frame.x + component_sliders.frame.width + 0.001
    );
    assert_eq!(
        input_slider.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(136, 147, 153)
    );
    assert_eq!(
        style_color_u8(input_slider.button_style.element.background_color.as_ref()),
        Some([17, 22, 26, 255])
    );
    assert_eq!(
        input_slider.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(32, 153, 162)
    );
    let range_slider = template_contract_node(&nodes, "WorkbenchInputRangeSlider");
    assert_eq!(range_slider.label_text.as_str(), "Range");
    assert_eq!(range_slider.value_text.as_str(), "0.80");
    assert_eq!(range_slider.layout_offset_x, -18.0);
    assert_eq!(range_slider.layout_icon_size, 9.0);
    assert_eq!(range_slider.layout_content_offset_x, -10.0);
    assert_eq!(range_slider.layout_first_cell_offset_x, 18.0);
    assert_eq!(range_slider.layout_second_cell_offset_x, 20.0);
    assert_eq!(
        range_slider.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(183, 241, 248)
    );
    assert_eq!(range_slider.frame.height, 46.0);
    assert_eq!(range_slider.value_color, input_slider.value_color);
    assert!(range_slider.frame.x >= component_sliders.frame.x);
    assert!(
        range_slider.frame.x + range_slider.frame.width
            <= component_sliders.frame.x + component_sliders.frame.width + 0.001
    );
    let steps_slider = template_contract_node(&nodes, "WorkbenchInputStepsSlider");
    assert_eq!(steps_slider.label_text.as_str(), "Steps");
    assert_eq!(steps_slider.value_text.as_str(), "3");
    assert_eq!(steps_slider.layout_offset_x, -18.0);
    assert_eq!(steps_slider.layout_icon_size, 9.0);
    assert_eq!(steps_slider.layout_content_offset_x, -10.0);
    assert_eq!(steps_slider.layout_first_cell_offset_x, 18.0);
    assert_eq!(steps_slider.layout_third_cell_offset_x, 5.0);
    assert_eq!(
        steps_slider.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(183, 241, 248)
    );
    assert!(steps_slider.frame.x >= component_sliders.frame.x);
    assert!(
        steps_slider.frame.x + steps_slider.frame.width
            <= component_sliders.frame.x + component_sliders.frame.width + 0.001
    );
    let list_group = template_contract_node(&nodes, "WorkbenchListGroup");
    let menu_title = template_contract_node(&nodes, "WorkbenchMenuTitle");
    let popup_menu = template_contract_node(&nodes, "WorkbenchPopupMenu");
    assert!(list_group.frame.x >= component_list.frame.x);
    assert!(menu_title.frame.y > list_group.frame.y + list_group.frame.height);
    assert!(popup_menu.frame.y > menu_title.frame.y + menu_title.frame.height);
    assert_eq!(steps_slider.value_color, input_slider.value_color);
    let input_focused = template_contract_node(&nodes, "WorkbenchInputFocused");
    assert!(input_focused.focused);
    assert_eq!(
        render_border_for_control(&bridge, "WorkbenchInputFocused").as_deref(),
        Some("#1b98a0")
    );
    let input_disabled = template_contract_node(&nodes, "WorkbenchInputDisabled");
    assert!(input_disabled.disabled);
    assert!((input_disabled.button_style.element.opacity - 0.94).abs() < 0.001);
    let selection_title = template_contract_node(&nodes, "WorkbenchSelectionTitle");
    assert_eq!(
        selection_title.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(131, 141, 148)
    );
    let labs_tab_one = template_contract_node(&nodes, "WorkbenchLabsTabOne");
    let labs_tab_two = template_contract_node(&nodes, "WorkbenchLabsTabTwo");
    let labs_tab_three = template_contract_node(&nodes, "WorkbenchLabsTabThree");
    assert_eq!(labs_tab_one.text.as_str(), "Tab 1");
    assert_eq!(labs_tab_two.text.as_str(), "Tab 2");
    assert_eq!(labs_tab_three.text.as_str(), "Tab 3");
    assert_eq!(labs_tab_one.layout_offset_x, 3.0);
    assert_eq!(labs_tab_one.layout_offset_y, 2.0);
    assert!(labs_tab_one.selected);
    assert!(!labs_tab_two.selected);
    assert!(!labs_tab_three.selected);
    let list_item = template_contract_node(&nodes, "WorkbenchListItem");
    assert_eq!(
        list_item.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(197, 208, 213)
    );
    assert_eq!(
        list_item.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(141, 156, 164)
    );
    let list_selected = template_contract_node(&nodes, "WorkbenchListSelected");
    assert_eq!(
        list_selected.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(53, 199, 208)
    );
    assert_eq!(
        list_selected.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(122, 230, 240)
    );
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchListSelected").as_deref(),
        Some("#12383d")
    );
    let list_disabled = template_contract_node(&nodes, "WorkbenchListDisabled");
    assert_eq!(
        list_disabled.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(88, 101, 108)
    );
    assert_eq!(
        list_disabled.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(88, 101, 108)
    );
    assert!((list_selected.frame.y - (list_item.frame.y + list_item.frame.height)).abs() < 0.001);
    assert!(
        (list_disabled.frame.y - (list_selected.frame.y + list_selected.frame.height)).abs()
            < 0.001
    );
    let position_axis_x = template_contract_node(&nodes, "WorkbenchTransformPositionAxisX");
    let position_value_x = template_contract_node(&nodes, "WorkbenchTransformPositionX");
    assert_eq!(
        position_axis_x.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(86, 104, 113)
    );
    assert_eq!(
        position_value_x.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(146, 158, 164)
    );
    let scale_axis_x = template_contract_node(&nodes, "WorkbenchTransformScaleAxisX");
    let scale_value_x = template_contract_node(&nodes, "WorkbenchTransformScaleX");
    assert_eq!(
        scale_axis_x.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(104, 118, 126)
    );
    assert_eq!(
        scale_value_x.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(146, 158, 164)
    );
    let status_ready = template_contract_node(&nodes, "WorkbenchStatusReady");
    assert!((status_ready.layout_offset_x - 4.0).abs() < 0.001);
    assert!((status_ready.layout_offset_y + 1.0).abs() < 0.001);
    assert!((status_ready.layout_content_offset_x - 8.0).abs() < 0.001);
    assert!((status_ready.value_number - 9.0).abs() < 0.001);
    assert_eq!(
        status_ready.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(143, 154, 160)
    );
    assert_eq!(
        status_ready.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(78, 170, 95)
    );
    let status_errors = template_contract_node(&nodes, "WorkbenchStatusErrors");
    assert_eq!(
        status_errors.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(88, 184, 102)
    );
    assert_eq!(
        status_errors.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(17, 32, 24)
    );
    assert!((status_errors.layout_icon_size - 12.04).abs() < 0.001);
    let status_warnings = template_contract_node(&nodes, "WorkbenchStatusWarnings");
    assert!((status_warnings.layout_offset_x - 5.5).abs() < 0.001);
    assert!((status_warnings.layout_offset_y + 2.0).abs() < 0.001);
    assert!((status_warnings.layout_content_offset_x - 6.45).abs() < 0.001);
    assert!((status_warnings.layout_content_offset_y + 2.0).abs() < 0.001);
    assert!((status_warnings.value_number - 21.0).abs() < 0.001);
    assert_eq!(
        status_warnings.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(135, 146, 153)
    );
    assert_eq!(
        status_warnings.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(242, 195, 86)
    );
    assert_eq!(
        status_warnings.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(17, 24, 26)
    );
    assert!((status_warnings.icon_stroke_width - 1.45).abs() < 0.001);
    let status_messages = template_contract_node(&nodes, "WorkbenchStatusMessages");
    assert!((status_messages.layout_offset_x + 6.0).abs() < 0.001);
    assert!((status_messages.layout_offset_y + 2.0).abs() < 0.001);
    assert!((status_messages.layout_content_offset_y - 2.0).abs() < 0.001);
    assert!((status_messages.value_number - 18.0).abs() < 0.001);
    assert_eq!(
        status_messages.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(151, 163, 169)
    );
    assert_eq!(
        status_messages.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(76, 154, 232)
    );
    let toast = template_contract_node(&nodes, "WorkbenchToastRoot");
    assert!((toast.layout_offset_x + 15.0).abs() < 0.001);
    assert!((toast.layout_offset_y - 0.0).abs() < 0.001);
    assert!((toast.value_number - 18.0).abs() < 0.001);
    assert_eq!(
        toast.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(32, 159, 169)
    );
    assert_eq!(
        toast.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(35, 143, 152)
    );
    let tooltip = template_contract_node(&nodes, "WorkbenchTooltipRoot");
    assert_eq!(tooltip.role.as_str(), "Tooltip");
    assert_eq!(tooltip.component_role.as_str(), "tooltip");
    assert_eq!(tooltip.text.as_str(), "Tooltip");
    assert_eq!(tooltip.label_text.as_str(), "This is a tooltip");
    assert_eq!(tooltip.frame.width, 110.0);
    assert_eq!(tooltip.frame.height, 78.0);
    assert_eq!(tooltip.layout_icon_size, 18.0);
    assert_eq!(tooltip.layout_content_offset_y, 56.0);
    assert_eq!(tooltip.value_number, 8.0);
    assert_eq!(
        tooltip.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(23, 28, 32)
    );
    assert_eq!(
        tooltip.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(168, 179, 184)
    );
    assert_eq!(
        tooltip.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(37, 156, 167)
    );
    assert_eq!(
        style_color_u8(tooltip.button_style.element.background_color.as_ref()),
        Some([23, 28, 32, 255])
    );
    assert_eq!(
        style_color_u8(tooltip.button_style.element.border_color.as_ref()),
        Some([37, 45, 50, 255])
    );
    assert_eq!(
        style_color_u8(tooltip.button_style.element.foreground_color.as_ref()),
        Some([216, 227, 231, 255])
    );
    let status_grid = template_contract_node(&nodes, "WorkbenchStatusGrid");
    assert!((status_grid.layout_offset_y + 0.5).abs() < 0.001);
    assert_eq!(
        status_grid.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(125, 137, 144)
    );
    let status_snap = template_contract_node(&nodes, "WorkbenchStatusSnap");
    assert!((status_snap.layout_offset_y + 0.5).abs() < 0.001);
    assert_eq!(
        status_snap.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(125, 137, 144)
    );
    let status_snap_toggle = template_contract_node(&nodes, "WorkbenchStatusSnapToggle");
    let status_world = template_contract_node(&nodes, "WorkbenchStatusWorld");
    let status_target = template_contract_node(&nodes, "WorkbenchStatusTarget");
    let status_zoom = template_contract_node(&nodes, "WorkbenchStatusZoom");
    assert!((status_snap_toggle.layout_offset_y + 0.5).abs() < 0.001);
    assert!((status_world.layout_offset_y + 0.5).abs() < 0.001);
    assert!((status_target.layout_offset_y + 0.5).abs() < 0.001);
    assert!((status_zoom.layout_offset_y + 0.5).abs() < 0.001);
    let inspector_material_row = template_contract_node(&nodes, "WorkbenchMaterialRow");
    assert_eq!(inspector_material_row.text.as_str(), "Materials");
    assert_eq!(
        inspector_material_row.label_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(154, 165, 171)
    );
    assert_eq!(
        inspector_material_row.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(143, 154, 160)
    );
    assert_eq!(
        inspector_material_row.icon_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(194, 204, 209)
    );
    assert_eq!(
        style_color_u8(
            inspector_material_row
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        Some([23, 28, 32, 255])
    );
    assert_eq!(
        style_color_u8(
            inspector_material_row
                .button_style
                .element
                .border_color
                .as_ref()
        ),
        Some([23, 28, 32, 255])
    );
    assert_eq!(inspector_material_row.layout_icon_size, 15.0);
    let add_component = template_contract_node(&nodes, "WorkbenchAddComponent");
    assert_eq!(
        style_color_u8(add_component.button_style.element.border_color.as_ref()),
        Some([42, 50, 56, 255])
    );
    let scale_link = template_contract_node(&nodes, "WorkbenchTransformScaleLink");
    assert_eq!(scale_link.layout_offset_x, -12.0);
    assert_eq!(scale_link.layout_icon_size, 17.0);
    assert!(matches!(
        bridge
            .binding_for_control(
                EditorWorkbenchTemplateControlIds::PRIMARY_BUTTON,
                UiEventKind::Click,
            )
            .expect("primary button binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.button.primary"
    ));
    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchTertiaryButton", UiEventKind::Click)
            .expect("tertiary button binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.button.tertiary"
    ));
    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchDropdownButton", UiEventKind::Click)
            .expect("button dropdown open binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.button_dropdown.open"
    ));
    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchDropdownButton", UiEventKind::Change)
            .expect("button dropdown select binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.button_dropdown.select"
    ));
    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchIconToggleSegmented", UiEventKind::Change)
            .expect("icon toggle segmented binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.icon_toggle_segment.select"
    ));
    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchModuleMaterial", UiEventKind::Click)
            .expect("material module tab binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.select"
    ));
    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchModuleCompile", UiEventKind::Click)
            .expect("compile module command binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.compile.invoke"
    ));
    let module_workspace = template_contract_node(&nodes, "WorkbenchMainBandModuleWorkspace");
    assert_eq!(
        module_workspace.control_id.as_str(),
        "WorkbenchMainBandModuleWorkspace"
    );
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchEffectRulesTab")
        .expect("effect rules tab projection")
        .routes
        .iter()
        .any(|route| route.binding_id == "WorkbenchModule/EffectRulesTab"
            && route.event_kind == UiEventKind::Click));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchMaterialGraphTab")
        .expect("material graph tab projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "WorkbenchModule/MaterialGraphTab"
                && route.event_kind == UiEventKind::Click
        ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchAssetsImportButton")
        .expect("assets import button projection")
        .routes
        .iter()
        .any(|route| route.binding_id == "WorkbenchModule/AssetsImport"
            && route.event_kind == UiEventKind::Click));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchMaterialDomainDropdown")
        .expect("material domain dropdown projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "WorkbenchModule/MaterialDomainEdit"
                && route.event_kind == UiEventKind::Change
        ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchBehaviorNodeRow03")
        .expect("behavior node row projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "WorkbenchModule/BehaviorNodeCooldown"
                && route.event_kind == UiEventKind::Click
        ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchVfxSystemField")
        .expect("vfx system field projection")
        .routes
        .iter()
        .any(|route| route.binding_id == "WorkbenchModule/VfxSystemEdit"
            && route.event_kind == UiEventKind::Change));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchModuleAbility")
        .expect("ability module tab projection")
        .routes
        .iter()
        .any(|route| route.binding_id == "WorkbenchModule/Ability"
            && route.event_kind == UiEventKind::Click));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchAbilityNameField")
        .expect("ability name field projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "WorkbenchModule/AbilityNameEdit"
                && route.event_kind == UiEventKind::Change
        ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchTagsAbilityActivateRow")
        .expect("tags ability row projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "WorkbenchModule/TagsAbilityActivate"
                && route.event_kind == UiEventKind::Click
        ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchPerceptionConfigDropdown")
        .expect("perception config dropdown projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "WorkbenchModule/PerceptionConfigEdit"
                && route.event_kind == UiEventKind::Change
        ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchRenderLightingPassRow")
        .expect("render lighting pass row projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "WorkbenchModule/RenderLightingPass"
                && route.event_kind == UiEventKind::Click
        ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchHudScreenDropdown")
        .expect("hud screen dropdown projection")
        .routes
        .iter()
        .any(|route| route.binding_id == "WorkbenchModule/HudScreenEdit"
            && route.event_kind == UiEventKind::Change));

    assert!(matches!(
        bridge
            .binding_for_control("WorkbenchToolMove", UiEventKind::Click)
            .expect("move tool binding")
            .payload(),
        EditorUiBindingPayload::ViewportCommand(ViewportCommand::SetTool(SceneViewportTool::Move))
    ));

    bridge.recompute_layout(UiSize::new(1440.0, 941.0)).unwrap();
    assert_eq!(
        bridge.frames().viewport,
        UiFrame::new(404.0, 60.0, 632.0, 428.0)
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id(EditorWorkbenchTemplateControlIds::VIEWPORT)
            .expect("resized viewport projection")
            .frame,
        bridge.frames().viewport
    );
}

#[test]
fn componentized_workbench_window_template_bridge_updates_module_navigation_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchModuleEffect", "selected"));
    assert!(control_bool(&bridge, "WorkbenchModuleCompile", "selected"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchSceneWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchModuleMaterial",
        "selected"
    ));
    assert!(!control_bool(&bridge, "WorkbenchModuleAssets", "selected"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchModuleMaterial", UiEventKind::Click)
            .unwrap()
            .expect("material module tab should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.select"
    ));
    assert!(!control_bool(&bridge, "WorkbenchModuleEffect", "selected"));
    assert!(control_bool(&bridge, "WorkbenchModuleMaterial", "selected"));
    assert!(control_bool(&bridge, "WorkbenchModuleMaterial", "checked"));
    assert!(!control_bool(&bridge, "WorkbenchModuleAssets", "selected"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchSceneWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleMaterialWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleEffectWorkspace"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchMaterialRailGap")
            .expect("visible material rail gap projection")
            .frame
            .width,
        72.0
    );

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchMaterialNodeRow02", UiEventKind::Click)
            .unwrap()
            .expect("material node row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.node_roughness.select"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchMaterialNodeRow02",
        "selected"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchMaterialBaseColorRow",
        "selected"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchMaterialDomainDropdown", UiEventKind::Change)
            .unwrap()
            .expect("material domain dropdown should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.domain.edit"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchModuleEffect", UiEventKind::Click)
            .unwrap()
            .expect("effect module tab should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.effect.select"
    ));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleEffectWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleMaterialWorkspace"),
        Some(UiVisibility::Collapsed)
    );
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchEffectDamageFireRow", UiEventKind::Click)
            .unwrap()
            .expect("effect asset row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.effect.damage_fire_row.select"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchEffectHealthRegenRow",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchEffectDamageFireRow",
        "selected"
    ));
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchEffectModifierHealingRow", UiEventKind::Click)
            .unwrap()
            .expect("effect modifier row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.effect.modifier_healing.select"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchEffectModifierHealthRow",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchEffectModifierHealingRow",
        "selected"
    ));
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchEffectMagnitudeField", UiEventKind::Submit)
            .unwrap()
            .expect("effect magnitude field should expose a preview submit binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.effect.magnitude.commit"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchModuleMaterial", UiEventKind::Click)
            .unwrap()
            .expect("material module tab should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.select"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchMaterialPreviewTab", UiEventKind::Click)
            .unwrap()
            .expect("material preview tab should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.preview_tab.select"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchMaterialGraphTab",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchMaterialPreviewTab",
        "selected"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchMaterialNormalRow", UiEventKind::Click)
            .unwrap()
            .expect("material normal row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.normal_row.select"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchMaterialBaseColorRow",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchMaterialNormalRow",
        "selected"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchModuleDiff", UiEventKind::Click)
            .unwrap()
            .expect("diff module command should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.diff.invoke"
    ));
    assert!(!control_bool(&bridge, "WorkbenchModuleCompile", "selected"));
    assert!(control_bool(&bridge, "WorkbenchModuleDiff", "selected"));
    assert!(control_bool(&bridge, "WorkbenchModuleDiff", "checked"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchModuleBrowse", UiEventKind::Click)
            .unwrap()
            .expect("browse module command should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.browse.invoke"
    ));
    assert!(control_bool(&bridge, "WorkbenchModuleBrowse", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchModuleDiff", "selected"));
    assert!(control_bool(&bridge, "WorkbenchModuleAssets", "selected"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleAssetsWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleMaterialWorkspace"),
        Some(UiVisibility::Collapsed)
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchModuleMaterial",
        "selected"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchAssetsImportButton", UiEventKind::Click)
            .unwrap()
            .expect("assets import button should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.assets.import.invoke"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchAssetsImportButton",
        "selected"
    ));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchModuleAbility", UiEventKind::Click)
            .unwrap()
            .expect("ability module tab should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.ability.select"
    ));
    assert!(control_bool(&bridge, "WorkbenchModuleAbility", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchModuleAssets", "selected"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchSceneWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleAbilityWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleAssetsWorkspace"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchAbilityRailGap")
            .expect("visible ability rail gap projection")
            .frame
            .width,
        72.0
    );
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchAbilityTaskCostRow", UiEventKind::Click)
            .unwrap()
            .expect("ability task row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.ability.task_cost.select"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchAbilityTaskCostRow",
        "selected"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchAbilityTaskActivateRow",
        "selected"
    ));
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchAbilityNameField", UiEventKind::Change)
            .unwrap()
            .expect("ability name field should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.ability.name.edit"
    ));

    for (control_id, action_id, workspace_id) in [
        (
            "WorkbenchModuleTags",
            "workbench.module.tags.select",
            "WorkbenchModuleTagsWorkspace",
        ),
        (
            "WorkbenchModulePerception",
            "workbench.module.perception.select",
            "WorkbenchModulePerceptionWorkspace",
        ),
        (
            "WorkbenchModuleRender",
            "workbench.module.render.select",
            "WorkbenchModuleRenderWorkspace",
        ),
        (
            "WorkbenchModuleHud",
            "workbench.module.hud.select",
            "WorkbenchModuleHudWorkspace",
        ),
    ] {
        assert!(matches!(
            bridge
                .dispatch_control_state(control_id, UiEventKind::Click)
                .unwrap()
                .expect("additional module tab should expose a preview binding")
                .payload(),
            EditorUiBindingPayload::MenuAction { action_id: dispatched_action }
                if dispatched_action == action_id
        ));
        assert!(control_bool(&bridge, control_id, "selected"));
        assert_eq!(
            control_visibility(&bridge, workspace_id),
            Some(UiVisibility::Visible)
        );
        assert_eq!(
            control_visibility(&bridge, "WorkbenchModuleAbilityWorkspace"),
            Some(UiVisibility::Collapsed)
        );
    }

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchTagsAbilityActivateRow", UiEventKind::Click)
            .unwrap()
            .expect("tags row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.tags.ability_activate.select"
    ));
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchPerceptionConfigDropdown", UiEventKind::Change)
            .unwrap()
            .expect("perception dropdown should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.perception.config.edit"
    ));
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchRenderLightingPassRow", UiEventKind::Click)
            .unwrap()
            .expect("render pass row should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.render.lighting_pass.select"
    ));
    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchHudScreenDropdown", UiEventKind::Change)
            .unwrap()
            .expect("hud dropdown should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.hud.screen.edit"
    ));
}

#[test]
fn componentized_workbench_window_template_bridge_syncs_scene_and_inspector_snapshot_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    assert_virtual_row_repeat(
        &bridge,
        "WorkbenchInspectorMesh",
        "WorkbenchComponentPropertySlot04Row",
        "WorkbenchComponentPropertyVirtualRow",
        4,
        "v2",
    );
    let scene_entries = vec![
        SceneEntry {
            id: 1,
            name: "World".to_string(),
            depth: 0,
            selected: false,
        },
        SceneEntry {
            id: 2,
            name: "GameplayRoot".to_string(),
            depth: 1,
            selected: true,
        },
        SceneEntry {
            id: 3,
            name: "CameraRig".to_string(),
            depth: 2,
            selected: false,
        },
    ];
    let inspector = InspectorSnapshot {
        id: 2,
        name: "GameplayRoot".to_string(),
        parent: "World".to_string(),
        translation: ["12.0".to_string(), "3.5".to_string(), "-8.0".to_string()],
        plugin_components: vec![InspectorPluginComponentSnapshot {
            component_id: "zircon.transform".to_string(),
            display_name: "Transform Component".to_string(),
            plugin_id: "zircon.core".to_string(),
            drawer_available: false,
            drawer_ui_document: None,
            drawer_controller: None,
            drawer_template_id: None,
            drawer_data_root: None,
            drawer_bindings: Vec::new(),
            diagnostic: None,
            properties: vec![
                InspectorPluginComponentPropertySnapshot {
                    field_id: "visible".to_string(),
                    name: "visible".to_string(),
                    label: "Visible".to_string(),
                    value: "true".to_string(),
                    value_kind: "bool".to_string(),
                    editable: true,
                },
                InspectorPluginComponentPropertySnapshot {
                    field_id: "cast_shadows".to_string(),
                    name: "cast_shadows".to_string(),
                    label: "Cast Shadows".to_string(),
                    value: "false".to_string(),
                    value_kind: "bool".to_string(),
                    editable: true,
                },
                InspectorPluginComponentPropertySnapshot {
                    field_id: "receive_shadows".to_string(),
                    name: "receive_shadows".to_string(),
                    label: "Receive Shadows".to_string(),
                    value: "true".to_string(),
                    value_kind: "bool".to_string(),
                    editable: true,
                },
                InspectorPluginComponentPropertySnapshot {
                    field_id: "material_slot".to_string(),
                    name: "material_slot".to_string(),
                    label: "Material Slot".to_string(),
                    value: "0".to_string(),
                    value_kind: "u32".to_string(),
                    editable: true,
                },
                InspectorPluginComponentPropertySnapshot {
                    field_id: "lightmap_index".to_string(),
                    name: "lightmap_index".to_string(),
                    label: "Lightmap".to_string(),
                    value: "1".to_string(),
                    value_kind: "u32".to_string(),
                    editable: true,
                },
            ],
        }],
    };

    bridge
        .sync_scene_and_inspector(&scene_entries, Some(&inspector))
        .unwrap();

    assert_eq!(
        control_string(&bridge, "WorkbenchSceneRootItem", "text").as_deref(),
        Some("World")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneEnvironmentItem", "text").as_deref(),
        Some("GameplayRoot")
    );
    assert!(control_bool(
        &bridge,
        "WorkbenchSceneEnvironmentItem",
        "selected"
    ));
    assert_eq!(
        control_integer(&bridge, "WorkbenchSceneEnvironmentItem", "tree_depth"),
        Some(1)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchScenePropsItem"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchInspectorTitle", "text").as_deref(),
        Some("GameplayRoot")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchTransformPosition", "value").as_deref(),
        Some("X 12.0   Y 3.5   Z -8.0")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchTransformPositionX", "value").as_deref(),
        Some("12.0")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchTransformPositionY", "value").as_deref(),
        Some("3.5")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchTransformPositionZ", "value").as_deref(),
        Some("-8.0")
    );
    let position_x = bridge
        .host_projection()
        .node_by_control_id("WorkbenchTransformPositionX")
        .expect("position X field projection");
    assert_eq!(position_x.component, "InputField");
    assert!(control_has_class(
        &bridge,
        "WorkbenchTransformPositionX",
        "workbench-axis-value-field"
    ));
    assert!(position_x.routes.iter().any(|route| {
        route.binding_id == "Inspector/TransformPositionXEdit"
            && route.event_kind == UiEventKind::Change
    }));
    assert!(position_x.routes.iter().any(|route| {
        route.binding_id == "Inspector/TransformPositionXCommit"
            && route.event_kind == UiEventKind::Submit
    }));
    assert_eq!(
        control_string(&bridge, "WorkbenchMeshLabel", "text").as_deref(),
        Some("Transform Component")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMeshRow", "text").as_deref(),
        Some("Visible")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMeshRow", "value_text").as_deref(),
        Some("true")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialRow", "text").as_deref(),
        Some("Cast Shadows")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialRow", "value_text").as_deref(),
        Some("false")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMeshRow", "inspector_property_field_id").as_deref(),
        Some("visible")
    );
    assert_eq!(
        control_string(
            &bridge,
            "WorkbenchMaterialRow",
            "inspector_property_field_id"
        )
        .as_deref(),
        Some("cast_shadows")
    );
    assert_eq!(
        control_string(
            &bridge,
            "WorkbenchMaterialRow",
            "inspector_property_value_kind"
        )
        .as_deref(),
        Some("bool")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialRow", "value").as_deref(),
        Some("false")
    );
    assert!(control_bool(
        &bridge,
        "WorkbenchMaterialRow",
        "inspector_property_editable"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentPropertySlot03Row", "text").as_deref(),
        Some("Receive Shadows")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentPropertySlot03Row", "value_text").as_deref(),
        Some("true")
    );
    assert_eq!(
        template_contract_node(
            &to_host_contract_workbench_window_nodes(Some(bridge.host_projection())),
            "WorkbenchComponentPropertySlot03Row",
        )
        .layout_content_offset_x,
        34.0
    );
    assert_eq!(
        control_string(
            &bridge,
            "WorkbenchComponentPropertySlot04Row",
            "inspector_property_field_id"
        )
        .as_deref(),
        Some("material_slot")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentPropertyVirtualRow05", "text").as_deref(),
        Some("Lightmap")
    );
    assert_eq!(
        control_string(
            &bridge,
            "WorkbenchComponentPropertyVirtualRow05",
            "value_text"
        )
        .as_deref(),
        Some("1")
    );
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchComponentPropertyVirtualRow05")
        .expect("virtual component property row projection")
        .routes
        .iter()
        .any(
            |route| route.binding_id == "Inspector/ComponentProperty04Edit"
                && route.event_kind == UiEventKind::Change
        ));
    let mesh_row = bridge
        .host_projection()
        .node_by_control_id("WorkbenchMeshRow")
        .expect("mesh/property row projection");
    assert_eq!(mesh_row.component, "InputField");
    assert!(control_has_class(
        &bridge,
        "WorkbenchMeshRow",
        "workbench-component-property-row"
    ));
    assert!(mesh_row.routes.iter().any(|route| {
        route.binding_id == "Inspector/ComponentProperty01Edit"
            && route.event_kind == UiEventKind::Change
    }));
    assert!(mesh_row.routes.iter().any(|route| {
        route.binding_id == "Inspector/ComponentProperty01Commit"
            && route.event_kind == UiEventKind::Submit
    }));
    let material_row = bridge
        .host_projection()
        .node_by_control_id("WorkbenchMaterialRow")
        .expect("material/property row projection");
    let material_host_row = template_contract_node(
        &to_host_contract_workbench_window_nodes(Some(bridge.host_projection())),
        "WorkbenchMaterialRow",
    );
    assert_eq!(
        style_color_u8(
            material_host_row
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        Some([40, 46, 50, 255])
    );
    assert_eq!(
        style_color_u8(material_host_row.button_style.element.border_color.as_ref()),
        Some([52, 61, 67, 255])
    );
    assert_eq!(
        material_host_row.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(181, 192, 197)
    );
    assert!(material_row.routes.iter().any(|route| {
        route.binding_id == "Inspector/ComponentProperty02Edit"
            && route.event_kind == UiEventKind::Change
    }));
    assert!(material_row.routes.iter().any(|route| {
        route.binding_id == "Inspector/ComponentProperty02Commit"
            && route.event_kind == UiEventKind::Submit
    }));

    let extended_entries = (0..8)
        .map(|index| SceneEntry {
            id: (index + 1) as u64,
            name: format!("SceneNode_{:02}", index + 1),
            depth: if index == 0 { 0 } else { 1 + index % 3 },
            selected: index == 7,
        })
        .collect::<Vec<_>>();
    bridge
        .sync_scene_and_inspector(&extended_entries, Some(&inspector))
        .unwrap();
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneSlot07Item", "text").as_deref(),
        Some("SceneNode_07")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneSlot08Item", "text").as_deref(),
        Some("SceneNode_08")
    );
    assert!(control_bool(
        &bridge,
        "WorkbenchSceneSlot08Item",
        "selected"
    ));
    assert_eq!(
        control_integer(&bridge, "WorkbenchSceneSlot08Item", "tree_depth"),
        Some(2)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchSceneSlot09Item"),
        Some(UiVisibility::Collapsed)
    );

    bridge.sync_scene_and_inspector(&[], None).unwrap();
    assert_eq!(
        control_visibility(&bridge, "WorkbenchSceneRootItem"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchInspectorTitle", "text").as_deref(),
        Some("No Selection")
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchInspectorTransform"),
        Some(UiVisibility::Collapsed)
    );
    let cleared_material_row = template_contract_node(
        &to_host_contract_workbench_window_nodes(Some(bridge.host_projection())),
        "WorkbenchMaterialRow",
    );
    assert_eq!(
        style_color_u8(
            cleared_material_row
                .button_style
                .element
                .background_color
                .as_ref()
        ),
        None
    );
    assert_eq!(
        style_color_u8(
            cleared_material_row
                .button_style
                .element
                .border_color
                .as_ref()
        ),
        None
    );
    assert_eq!(
        cleared_material_row.value_color,
        crate::ui::retained_host::primitives::Color::from_rgb_u8(216, 227, 231)
    );
}

#[test]
fn componentized_workbench_scene_tree_grows_and_reuses_virtual_rows_for_live_snapshot_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    assert_virtual_row_repeat(
        &bridge,
        "WorkbenchSceneTree",
        "WorkbenchSceneSlot10Item",
        "WorkbenchSceneVirtualItem",
        10,
        "v2",
    );
    let inspector = InspectorSnapshot {
        id: 13,
        name: "SceneNode_13".to_string(),
        parent: "SceneNode_12".to_string(),
        translation: ["0.0".to_string(), "1.0".to_string(), "2.0".to_string()],
        plugin_components: Vec::new(),
    };
    let thirteen_entries = numbered_scene_entries(13, 12);
    bridge
        .sync_scene_and_inspector(&thirteen_entries, Some(&inspector))
        .unwrap();

    assert_eq!(
        control_string(&bridge, "WorkbenchSceneVirtualItem11", "text").as_deref(),
        Some("SceneNode_11")
    );
    assert_eq!(
        control_integer(&bridge, "WorkbenchSceneVirtualItem12", "scene_node_id"),
        Some(12)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneVirtualItem13", "text").as_deref(),
        Some("SceneNode_13")
    );
    assert!(control_bool(
        &bridge,
        "WorkbenchSceneVirtualItem13",
        "selected"
    ));
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchSceneVirtualItem13")
        .is_some());
    let virtual_binding = bridge
        .binding_for_control("WorkbenchSceneVirtualItem13", UiEventKind::Click)
        .expect("virtual scene row binding should resolve through authored prototype route");
    assert!(matches!(
        virtual_binding.payload(),
        EditorUiBindingPayload::SelectionCommand(_)
    ));
    assert!(
        bridge
            .surface()
            .last_rebuild_report
            .control_pool_created_count
            >= 3
    );

    let two_entries = numbered_scene_entries(2, 1);
    bridge
        .sync_scene_and_inspector(&two_entries, Some(&inspector))
        .unwrap();
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchSceneVirtualItem11")
        .is_none());
    assert!(
        bridge
            .surface()
            .last_rebuild_report
            .control_pool_recycled_count
            >= 3
    );

    let twelve_entries = numbered_scene_entries(12, 11);
    bridge
        .sync_scene_and_inspector(&twelve_entries, Some(&inspector))
        .unwrap();
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneVirtualItem12", "text").as_deref(),
        Some("SceneNode_12")
    );
    assert!(bridge
        .host_projection()
        .node_by_control_id("WorkbenchSceneVirtualItem12")
        .is_some());
    assert!(
        bridge
            .surface()
            .last_rebuild_report
            .control_pool_reused_count
            >= 2
    );
}

#[test]
fn componentized_workbench_window_template_bridge_updates_tool_selection_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge.recompute_layout(UiSize::new(1440.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchToolSelect", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchToolMove", "selected"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchToolMove").as_deref(),
        Some("#171c20")
    );

    let binding = bridge
        .dispatch_control_state("WorkbenchToolMove", UiEventKind::Click)
        .unwrap()
        .expect("move tool should have a binding");

    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::ViewportCommand(ViewportCommand::SetTool(SceneViewportTool::Move))
    ));
    assert!(!control_bool(&bridge, "WorkbenchToolSelect", "selected"));
    assert!(control_bool(&bridge, "WorkbenchToolMove", "selected"));
    assert!(control_bool(&bridge, "WorkbenchToolMove", "checked"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchToolMove").as_deref(),
        Some("#12383d")
    );
    assert_eq!(
        bridge.frames().viewport,
        UiFrame::new(404.0, 60.0, 632.0, 428.0)
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchToolMove")
            .expect("move tool projection")
            .frame,
        bridge
            .control_frame("WorkbenchToolMove")
            .expect("move tool frame")
    );
}

#[test]
fn componentized_workbench_control_dispatch_updates_state_and_emits_typed_event() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_tool_dispatch");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    let effects = dispatch_componentized_workbench_control(
        &harness.runtime,
        &mut bridge,
        "WorkbenchToolRotate",
        UiEventKind::Click,
    )
    .expect("rotate tool should dispatch")
    .unwrap();

    assert!(control_bool(&bridge, "WorkbenchToolRotate", "selected"));
    assert!(control_bool(&bridge, "WorkbenchToolRotate", "checked"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchToolRotate").as_deref(),
        Some("#12383d")
    );
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::SetTool {
            tool: SceneViewportTool::Rotate,
        })
    );
    assert!(effects.render_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn componentized_workbench_pointer_dispatch_hits_tool_control_and_emits_typed_event() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_pointer_tool");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let point = control_center(&bridge, "WorkbenchToolScale");

    let press_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, point).with_button(UiPointerButton::Primary),
    )
    .expect("tool pointer press should request paint-only feedback")
    .unwrap();
    assert!(press_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!press_effects.render_dirty);
    assert!(!press_effects.presentation_dirty);

    let effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, point).with_button(UiPointerButton::Primary),
    )
    .expect("tool pointer release should dispatch")
    .unwrap();

    assert!(control_bool(&bridge, "WorkbenchToolScale", "selected"));
    assert!(control_bool(&bridge, "WorkbenchToolScale", "checked"));
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::SetTool {
            tool: SceneViewportTool::Scale,
        })
    );
    assert!(effects.render_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn componentized_workbench_pointer_dispatch_ignores_decorative_viewport_scene_layer() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_pointer_viewport_scene");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .dispatch_control_state("WorkbenchModuleScene", UiEventKind::Click)
        .unwrap()
        .expect("scene module should expose a preview binding");
    assert_eq!(
        control_visibility(&bridge, "WorkbenchSceneWorkspace"),
        Some(UiVisibility::Visible)
    );
    let point = control_center(&bridge, "WorkbenchViewportFloorGrateRight");

    let press = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, point).with_button(UiPointerButton::Primary),
    );
    assert!(
        press.is_none(),
        "decorative viewport scene layer press should not request feedback or dispatch"
    );
    assert!(bridge.pointer_pressed_target().is_none());

    let release = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, point).with_button(UiPointerButton::Primary),
    );
    assert!(
        release.is_none(),
        "decorative viewport scene layer release should not dispatch"
    );
    assert!(harness.runtime.journal().records().is_empty());
}

#[test]
fn componentized_workbench_control_dispatch_records_component_drawer_preview_action() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_preview_dispatch");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    let effects = dispatch_componentized_workbench_control(
        &harness.runtime,
        &mut bridge,
        "WorkbenchCheckboxOff",
        UiEventKind::Toggle,
    )
    .expect("component lab preview binding should dispatch")
    .unwrap();

    assert!(control_bool(&bridge, "WorkbenchCheckboxOff", "checked"));
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Transient(EditorEventTransient::PressNode {
            node_path: "ComponentLab/CheckboxOffToggle".to_string(),
            pressed: false,
        })
    );
    assert_eq!(
        harness
            .runtime
            .journal()
            .records()
            .last()
            .unwrap()
            .operation_group
            .as_deref(),
        Some("ComponentLabPreview")
    );
    assert!(!effects.render_dirty);
    assert!(!effects.presentation_dirty);
}

#[test]
fn componentized_workbench_pointer_dispatch_hits_component_drawer_toggle_once() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_pointer_toggle");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let point = control_center(&bridge, "WorkbenchCheckboxOff");

    assert!(!control_bool(&bridge, "WorkbenchCheckboxOff", "checked"));
    let press_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, point).with_button(UiPointerButton::Primary),
    )
    .expect("checkbox pointer press should request paint-only feedback")
    .unwrap();
    assert!(press_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!press_effects.render_dirty);
    assert!(!press_effects.presentation_dirty);

    let effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, point).with_button(UiPointerButton::Primary),
    )
    .expect("checkbox pointer release should dispatch")
    .unwrap();

    assert!(control_bool(&bridge, "WorkbenchCheckboxOff", "checked"));
    assert_eq!(
        harness
            .runtime
            .journal()
            .records()
            .last()
            .unwrap()
            .operation_group
            .as_deref(),
        Some("ComponentLabPreview")
    );
    assert!(!effects.render_dirty);
    assert!(!effects.presentation_dirty);
}

#[test]
fn componentized_workbench_window_template_bridge_updates_activity_rail_selection_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchRailScene", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchRailImage", "selected"));

    let binding = bridge
        .dispatch_control_state("WorkbenchRailImage", UiEventKind::Click)
        .unwrap()
        .expect("asset rail button should have a binding");

    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::DockCommand(_)
    ));
    assert!(!control_bool(&bridge, "WorkbenchRailScene", "selected"));
    assert!(control_bool(&bridge, "WorkbenchRailImage", "selected"));
    assert!(control_bool(&bridge, "WorkbenchRailImage", "checked"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchRailImage").as_deref(),
        Some("#12383d")
    );
}

#[test]
fn componentized_workbench_window_template_bridge_updates_scene_tree_selection_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchScenePropsItem", "selected"));

    let binding = bridge
        .dispatch_control_state("WorkbenchScenePlayerItem", UiEventKind::Click)
        .unwrap()
        .expect("scene row should have a binding");

    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::SelectionCommand(_)
    ));
    assert!(!control_bool(&bridge, "WorkbenchSceneRootItem", "selected"));
    assert!(!control_bool(
        &bridge,
        "WorkbenchScenePropsItem",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchScenePlayerItem",
        "selected"
    ));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchScenePlayerItem").as_deref(),
        Some("#12383d")
    );
}

#[test]
fn componentized_workbench_window_template_bridge_updates_panel_tab_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchSceneTabScene", "selected"));
    assert!(!control_bool(
        &bridge,
        "WorkbenchSceneTabLayers",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchInspectorTabInspector",
        "selected"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchInspectorTabHistory",
        "selected"
    ));

    let scene_binding = bridge
        .dispatch_control_state("WorkbenchSceneTabLayers", UiEventKind::Click)
        .unwrap()
        .expect("scene layers tab should expose a preview binding");
    assert!(matches!(
        scene_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "scene_tree.layers_tab.select"
    ));
    assert!(!control_bool(&bridge, "WorkbenchSceneTabScene", "selected"));
    assert!(control_bool(&bridge, "WorkbenchSceneTabLayers", "selected"));
    assert!(control_bool(&bridge, "WorkbenchSceneTabLayers", "checked"));
    assert_eq!(
        render_border_for_control(&bridge, "WorkbenchSceneTabLayers").as_deref(),
        Some("#35c7d0")
    );

    let inspector_binding = bridge
        .dispatch_control_state("WorkbenchInspectorTabHistory", UiEventKind::Click)
        .unwrap()
        .expect("inspector history tab should expose a preview binding");
    assert!(matches!(
        inspector_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "inspector.history_tab.select"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchInspectorTabInspector",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchInspectorTabHistory",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchInspectorTabHistory",
        "checked"
    ));
}

#[test]
fn componentized_workbench_window_template_bridge_updates_component_drawer_tab_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(
        &bridge,
        "WorkbenchDrawerTabComponents",
        "selected"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchDrawerTabConsole",
        "selected"
    ));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentDrawerBody"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentDrawerConsoleBody"),
        Some(UiVisibility::Collapsed)
    );

    let binding = bridge
        .dispatch_control_state("WorkbenchDrawerTabConsole", UiEventKind::Click)
        .unwrap()
        .expect("component drawer console tab should expose a preview binding");
    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_drawer.console_tab.select"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchDrawerTabComponents",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchDrawerTabConsole",
        "selected"
    ));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentDrawerBody"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentDrawerConsoleBody"),
        Some(UiVisibility::Visible)
    );
}

#[test]
fn componentized_workbench_pointer_dispatch_hits_component_drawer_tab_visibility() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_pointer_drawer_tab");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let point = control_center(&bridge, "WorkbenchDrawerTabConsole");

    let press_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, point).with_button(UiPointerButton::Primary),
    )
    .expect("drawer tab pointer press should request paint-only feedback")
    .unwrap();
    assert!(press_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!press_effects.render_dirty);
    assert!(!press_effects.presentation_dirty);

    let effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, point).with_button(UiPointerButton::Primary),
    )
    .expect("drawer tab pointer release should dispatch")
    .unwrap();

    assert!(!control_bool(
        &bridge,
        "WorkbenchDrawerTabComponents",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchDrawerTabConsole",
        "selected"
    ));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentDrawerBody"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentDrawerConsoleBody"),
        Some(UiVisibility::Visible)
    );
    assert!(!effects.render_dirty);
    assert!(!effects.presentation_dirty);
}

#[test]
fn componentized_workbench_window_template_bridge_updates_component_drawer_input_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(!control_bool(
        &bridge,
        "WorkbenchInputDropdown",
        "popup_open"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchInputSegmented", "value").as_deref(),
        Some("center")
    );

    let dropdown_binding = bridge
        .dispatch_control_state("WorkbenchInputDropdown", UiEventKind::Click)
        .unwrap()
        .expect("dropdown should expose a preview binding");
    assert!(matches!(
        dropdown_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.input_dropdown.open"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchInputDropdown",
        "popup_open"
    ));
    assert!(control_bool(&bridge, "WorkbenchInputDropdown", "focused"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchInputDropdown").as_deref(),
        Some("#151b1f")
    );

    let segment_binding = bridge
        .dispatch_control_state("WorkbenchInputSegmented", UiEventKind::Change)
        .unwrap()
        .expect("segmented control should expose a preview binding");
    assert!(matches!(
        segment_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.input_segment.select"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchInputSegmented", "value").as_deref(),
        Some("right")
    );
    assert!(control_bool(&bridge, "WorkbenchInputSegmented", "selected"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchInputSegmented").as_deref(),
        Some("#12383d")
    );
}

#[test]
fn componentized_workbench_window_template_bridge_updates_component_drawer_button_icon_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchSecondaryButton").as_deref(),
        Some("#1d2328")
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchDropdownButton",
        "popup_open"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchIconToggleSegmented", "value").as_deref(),
        Some("grid")
    );

    let dropdown_binding = bridge
        .dispatch_control_state("WorkbenchDropdownButton", UiEventKind::Click)
        .unwrap()
        .expect("button dropdown should expose a preview binding");
    assert!(matches!(
        dropdown_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.button_dropdown.open"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchDropdownButton",
        "popup_open"
    ));
    assert!(control_bool(&bridge, "WorkbenchDropdownButton", "focused"));

    let icon_toggle_binding = bridge
        .dispatch_control_state("WorkbenchIconToggleSegmented", UiEventKind::Change)
        .unwrap()
        .expect("icon toggle should expose a preview binding");
    assert!(matches!(
        icon_toggle_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.icon_toggle_segment.select"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchIconToggleSegmented", "value").as_deref(),
        Some("list")
    );
    assert!(control_bool(
        &bridge,
        "WorkbenchIconToggleSegmented",
        "selected"
    ));
}

#[test]
fn componentized_workbench_window_projection_exports_dropdown_and_popup_rows() {
    let _guard = env_lock().lock().unwrap();

    let bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));

    let dropdown = template_contract_node(&nodes, "WorkbenchInputDropdown");
    assert_eq!(dropdown.role.as_str(), "Dropdown");
    assert_eq!(dropdown.layout_offset_x, -4.0);
    assert_eq!(dropdown.layout_offset_y, 8.0);
    assert_eq!(dropdown.frame.height, 30.5);
    assert_eq!(
        dropdown.options_text.as_str(),
        "dropdown, option_a, option_b"
    );
    assert_eq!(dropdown.options.row_count(), 3);
    assert_eq!(dropdown.options.row_data(0).as_deref(), Some("dropdown"));
    assert_eq!(dropdown.structured_options.row_count(), 3);

    let selected = template_contract_option(&dropdown.structured_options, 0);
    assert_eq!(selected.id.as_str(), "dropdown");
    assert_eq!(selected.label.as_str(), "dropdown");
    assert!(selected.selected);
    assert!(selected.special);
    assert!(!selected.disabled);

    let hovered = template_contract_option(&dropdown.structured_options, 1);
    assert_eq!(hovered.id.as_str(), "option_a");
    assert!(hovered.focused);
    assert!(hovered.hovered);
    assert!(!hovered.selected);

    let disabled = template_contract_option(&dropdown.structured_options, 2);
    assert_eq!(disabled.id.as_str(), "option_b");
    assert!(disabled.disabled);

    let stepper = template_contract_node(&nodes, "WorkbenchInputStepper");
    assert_eq!(stepper.role.as_str(), "InputField");
    assert_eq!(stepper.layout_offset_x, -4.0);
    assert_eq!(stepper.layout_offset_y, 8.0);
    assert_eq!(stepper.frame.height, 30.5);

    let popup_menu = template_contract_node(&nodes, "WorkbenchPopupMenu");
    assert_eq!(popup_menu.role.as_str(), "Menu");
    assert!(popup_menu.popup_open);
    assert_eq!(popup_menu.frame.width, 145.0);
    assert_eq!(popup_menu.layout_offset_y, -12.0);
    assert_eq!(popup_menu.structured_menu_items.row_count(), 5);

    let delete = template_contract_menu_item(&popup_menu.structured_menu_items, 3);
    assert_eq!(delete.raw.as_str(), "Delete|danger,hovered,icon=trash");
    assert_eq!(delete.label.as_str(), "Delete");
    assert_eq!(delete.action_id.as_str(), "menu.item.delete");
    assert!(delete.hovered);
    assert!(!delete.disabled);

    let more_tools = template_contract_menu_item(&popup_menu.structured_menu_items, 4);
    assert_eq!(more_tools.raw.as_str(), "More Tools|submenu");
    assert_eq!(more_tools.label.as_str(), "More Tools");
    assert_eq!(more_tools.action_id.as_str(), "menu.item.more_tools");
    assert!(!more_tools.hovered);
    assert!(!more_tools.disabled);
}

#[test]
fn componentized_workbench_dropdown_option_selection_updates_value_and_projection() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_dropdown_select");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    bridge
        .dispatch_control_state("WorkbenchInputDropdown", UiEventKind::Click)
        .unwrap()
        .expect("dropdown should expose an open binding");
    assert!(control_bool(
        &bridge,
        "WorkbenchInputDropdown",
        "popup_open"
    ));

    let effects = dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchInputDropdown",
        "option_a",
    )
    .expect("dropdown option selection should dispatch");

    assert_eq!(
        control_string(&bridge, "WorkbenchInputDropdown", "value").as_deref(),
        Some("option_a")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchInputDropdown", "value_text").as_deref(),
        Some("option_a")
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchInputDropdown",
        "popup_open"
    ));
    assert!(!control_bool(&bridge, "WorkbenchInputDropdown", "focused"));
    assert!(!control_bool(&bridge, "WorkbenchInputDropdown", "selected"));
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Transient(EditorEventTransient::PressNode {
            node_path: "component_lab.input_dropdown.select".to_string(),
            pressed: false,
        })
    );
    assert_eq!(
        harness
            .runtime
            .journal()
            .records()
            .last()
            .unwrap()
            .operation_group
            .as_deref(),
        Some("ComponentLabPreview")
    );

    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let dropdown = template_contract_node(&nodes, "WorkbenchInputDropdown");
    let old_selected = template_contract_option(&dropdown.structured_options, 0);
    assert_eq!(old_selected.id.as_str(), "dropdown");
    assert!(!old_selected.selected);
    assert!(!old_selected.special);

    let option = template_contract_option(&dropdown.structured_options, 1);
    assert_eq!(option.id.as_str(), "option_a");
    assert!(option.selected);
    assert!(option.special);
    assert!(!option.focused);
    assert!(!option.hovered);

    let disabled = template_contract_option(&dropdown.structured_options, 2);
    assert_eq!(disabled.id.as_str(), "option_b");
    assert!(disabled.disabled);

    let no_effects = dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchInputDropdown",
        "option_b",
    )
    .expect("disabled option selection should be swallowed");
    assert_eq!(
        control_string(&bridge, "WorkbenchInputDropdown", "value").as_deref(),
        Some("option_a")
    );
    assert_eq!(harness.runtime.journal().records().len(), 1);
    assert_eq!(no_effects, UiHostEventEffects::default());
}

#[test]
fn componentized_workbench_popup_cancel_closes_dropdown_without_value_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    bridge
        .dispatch_control_state("WorkbenchInputDropdown", UiEventKind::Click)
        .unwrap()
        .expect("dropdown should expose an open binding");
    assert!(control_bool(
        &bridge,
        "WorkbenchInputDropdown",
        "popup_open"
    ));

    let effects = dispatch_componentized_workbench_popup_cancelled(
        &mut bridge,
        "WorkbenchInputDropdown",
        WORKBENCH_POPUP_CANCEL_ACTION_ID,
    )
    .expect("popup cancel action should be routed")
    .expect("popup cancel should close the dropdown");

    assert!(!control_bool(
        &bridge,
        "WorkbenchInputDropdown",
        "popup_open"
    ));
    assert!(!control_bool(&bridge, "WorkbenchInputDropdown", "focused"));
    assert!(!control_bool(&bridge, "WorkbenchInputDropdown", "selected"));
    assert_eq!(
        control_string(&bridge, "WorkbenchInputDropdown", "value").as_deref(),
        Some("dropdown")
    );
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));

    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let dropdown = template_contract_node(&nodes, "WorkbenchInputDropdown");
    let selected = template_contract_option(&dropdown.structured_options, 0);
    assert_eq!(selected.id.as_str(), "dropdown");
    assert!(selected.selected);
    assert!(selected.special);
    assert!(!selected.focused);
    assert!(!selected.hovered);
    assert!(!selected.pressed);
    let next = template_contract_option(&dropdown.structured_options, 1);
    assert!(!next.focused);
    assert!(!next.hovered);
    assert!(!next.pressed);

    let no_effects = dispatch_componentized_workbench_popup_cancelled(
        &mut bridge,
        "WorkbenchInputDropdown",
        WORKBENCH_POPUP_CANCEL_ACTION_ID,
    )
    .expect("popup cancel action should still be routed when closed")
    .expect("closed dropdown cancel should be a no-op");
    assert_eq!(no_effects, UiHostEventEffects::default());
}

#[test]
fn componentized_workbench_popup_menu_item_selection_updates_value_and_projection() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_menu_select");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchPopupMenu", "popup_open"));

    let effects = dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchPopupMenu",
        "menu.item.delete",
    )
    .expect("popup menu item should be handled")
    .expect("popup menu item selection should dispatch");

    assert_eq!(
        control_string(&bridge, "WorkbenchPopupMenu", "value").as_deref(),
        Some("Delete")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchPopupMenu", "value_text").as_deref(),
        Some("Delete")
    );
    assert!(!control_bool(&bridge, "WorkbenchPopupMenu", "popup_open"));
    assert!(!control_bool(&bridge, "WorkbenchPopupMenu", "focused"));
    assert!(!control_bool(&bridge, "WorkbenchPopupMenu", "selected"));
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(harness.runtime.journal().records().is_empty());

    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let popup_menu = template_contract_node(&nodes, "WorkbenchPopupMenu");
    let delete = template_contract_menu_item(&popup_menu.structured_menu_items, 3);
    assert_eq!(delete.raw.as_str(), "Delete|danger,icon=trash");
    assert_eq!(delete.action_id.as_str(), "menu.item.delete");
    assert!(!delete.hovered);
    assert!(!delete.pressed);

    assert!(dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchPopupMenu",
        "Missing"
    )
    .is_none());
}

#[test]
fn componentized_workbench_popup_cancel_closes_menu_without_selecting_item() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchPopupMenu", "popup_open"));

    let effects = dispatch_componentized_workbench_popup_cancelled(
        &mut bridge,
        "WorkbenchPopupMenu",
        WORKBENCH_POPUP_CANCEL_ACTION_ID,
    )
    .expect("popup cancel action should be routed")
    .expect("popup cancel should close the menu");

    assert!(!control_bool(&bridge, "WorkbenchPopupMenu", "popup_open"));
    assert!(!control_bool(&bridge, "WorkbenchPopupMenu", "focused"));
    assert!(!control_bool(&bridge, "WorkbenchPopupMenu", "selected"));
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));

    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let popup_menu = template_contract_node(&nodes, "WorkbenchPopupMenu");
    let delete = template_contract_menu_item(&popup_menu.structured_menu_items, 3);
    assert_eq!(delete.raw.as_str(), "Delete|danger,icon=trash");
    assert_eq!(delete.action_id.as_str(), "menu.item.delete");
    assert!(!delete.focused);
    assert!(!delete.hovered);
    assert!(!delete.pressed);

    let no_route = dispatch_componentized_workbench_popup_cancelled(
        &mut bridge,
        "WorkbenchPopupMenu",
        "WrongAction",
    );
    assert!(no_route.is_none());
}

#[test]
fn componentized_workbench_pointer_focuses_input_fields_without_authored_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_pointer_input_focus");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(!control_component_focused(&bridge, "WorkbenchInputText"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchInputText").as_deref(),
        Some("#101417")
    );

    let text_point = control_center(&bridge, "WorkbenchInputText");
    let effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, text_point)
            .with_button(UiPointerButton::Primary),
    )
    .expect("text input pointer press should request paint-only feedback")
    .unwrap();

    assert!(control_component_focused(&bridge, "WorkbenchInputText"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchInputText").as_deref(),
        Some("#11191c")
    );
    assert_eq!(
        render_border_for_control(&bridge, "WorkbenchInputText").as_deref(),
        Some("#1b98a0")
    );
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!effects.render_dirty);
    assert!(!effects.presentation_dirty);
    assert!(harness.runtime.journal().records().is_empty());

    let stepper_point = control_center(&bridge, "WorkbenchInputStepper");
    let effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, stepper_point)
            .with_button(UiPointerButton::Primary),
    )
    .expect("stepper input pointer press should request paint-only feedback")
    .unwrap();

    assert!(!control_component_focused(&bridge, "WorkbenchInputText"));
    assert!(control_component_focused(&bridge, "WorkbenchInputStepper"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchInputText").as_deref(),
        Some("#101417")
    );
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchInputStepper").as_deref(),
        Some("#11191c")
    );
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!effects.render_dirty);
    assert!(!effects.presentation_dirty);
    assert!(harness.runtime.journal().records().is_empty());
}

#[test]
fn componentized_workbench_window_template_bridge_updates_component_drawer_selection_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchCheckboxOn", "checked"));
    assert!(!control_bool(&bridge, "WorkbenchCheckboxOff", "checked"));
    assert!(control_bool(&bridge, "WorkbenchRadioOn", "checked"));
    assert!(!control_bool(&bridge, "WorkbenchRadioOff", "checked"));
    assert!(control_bool(&bridge, "WorkbenchToggleOn", "checked"));
    assert!(control_bool(&bridge, "WorkbenchListSelected", "selected"));
    assert!(control_bool(&bridge, "WorkbenchLabsTabOne", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchLabsTabTwo", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchTableItem", "selected"));
    assert!(control_bool(&bridge, "WorkbenchTableSelected", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchTableTail", "selected"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchLabsTabTwo", UiEventKind::Click)
            .unwrap()
            .expect("labs tab should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.labs_tab_two.select"
    ));
    assert!(!control_bool(&bridge, "WorkbenchLabsTabOne", "selected"));
    assert!(control_bool(&bridge, "WorkbenchLabsTabTwo", "selected"));
    assert!(control_bool(&bridge, "WorkbenchLabsTabTwo", "checked"));
    assert!(!control_bool(&bridge, "WorkbenchLabsTabThree", "selected"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchCheckboxOff", UiEventKind::Toggle)
            .unwrap()
            .expect("checkbox should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.checkbox_off.toggle"
    ));
    assert!(control_bool(&bridge, "WorkbenchCheckboxOff", "checked"));
    assert!(control_bool(&bridge, "WorkbenchCheckboxOff", "selected"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchCheckboxOff").as_deref(),
        Some("#209fa8")
    );

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchRadioOff", UiEventKind::Change)
            .unwrap()
            .expect("radio should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.radio_off.select"
    ));
    assert!(!control_bool(&bridge, "WorkbenchRadioOn", "checked"));
    assert!(control_bool(&bridge, "WorkbenchRadioOff", "checked"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchToggleOn", UiEventKind::Toggle)
            .unwrap()
            .expect("switch should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.switch.toggle"
    ));
    assert!(!control_bool(&bridge, "WorkbenchToggleOn", "checked"));
    assert!(!control_bool(&bridge, "WorkbenchToggleOn", "selected"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchListItem", UiEventKind::Click)
            .unwrap()
            .expect("list item should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.list_item.select"
    ));
    assert!(control_bool(&bridge, "WorkbenchListItem", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchListSelected", "selected"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchListItem").as_deref(),
        Some("#12383d")
    );

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchTableItem", UiEventKind::Click)
            .unwrap()
            .expect("table item should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.table_item.select"
    ));
    assert!(control_bool(&bridge, "WorkbenchTableItem", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchTableSelected", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchTableTail", "selected"));

    assert!(matches!(
        bridge
            .dispatch_control_state("WorkbenchTableTail", UiEventKind::Click)
            .unwrap()
            .expect("table tail should expose a preview binding")
            .payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.table_tail.select"
    ));
    assert!(!control_bool(&bridge, "WorkbenchTableItem", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchTableSelected", "selected"));
    assert!(control_bool(&bridge, "WorkbenchTableTail", "selected"));
}

#[test]
fn startup_template_runtime_loads_componentized_workbench_window_bridge_source() {
    let _guard = env_lock().lock().unwrap();

    let runtime = Arc::new(load_startup_builtin_template_runtime().unwrap());
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new_with_runtime(
        runtime,
        UiSize::new(1672.0, 941.0),
    )
    .unwrap();

    assert!(bridge
        .host_projection()
        .node_by_control_id(EditorWorkbenchTemplateControlIds::ROOT)
        .is_some());
    assert_eq!(
        bridge.control_frame(EditorWorkbenchTemplateControlIds::STATUS_BAR),
        Some(UiFrame::new(0.0, 895.0, 1672.0, 46.0))
    );
}

fn round_to_layout_pixel(value: f32) -> f32 {
    value.round()
}

fn numbered_scene_entries(count: usize, selected_index: usize) -> Vec<SceneEntry> {
    (0..count)
        .map(|index| SceneEntry {
            id: (index + 1) as u64,
            name: format!("SceneNode_{:02}", index + 1),
            depth: if index == 0 { 0 } else { 1 + index % 3 },
            selected: index == selected_index,
        })
        .collect()
}

fn template_contract_node(
    nodes: &ModelRc<TemplatePaneNodeData>,
    control_id: &str,
) -> TemplatePaneNodeData {
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("{control_id} should project to the host contract"))
}

fn template_contract_option(
    options: &ModelRc<TemplatePaneOptionData>,
    row: usize,
) -> TemplatePaneOptionData {
    options
        .row_data(row)
        .unwrap_or_else(|| panic!("structured option row {row} should be projected"))
}

fn template_contract_menu_item(
    items: &ModelRc<TemplatePaneMenuItemData>,
    row: usize,
) -> TemplatePaneMenuItemData {
    items
        .row_data(row)
        .unwrap_or_else(|| panic!("structured menu item row {row} should be projected"))
}

fn control_bool(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> bool {
    bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
                .and_then(|metadata| metadata.attributes.get(property))
                .and_then(toml::Value::as_bool)
        })
        .unwrap_or(false)
}

fn control_string(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Option<String> {
    bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .and_then(|metadata| metadata.attributes.get(property))
            .and_then(toml::Value::as_str)
            .map(str::to_string)
    })
}

fn control_has_class(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    class_name: &str,
) -> bool {
    bridge.surface().tree.nodes.values().any(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .is_some_and(|metadata| {
                metadata
                    .classes
                    .iter()
                    .any(|class| class.as_str() == class_name)
            })
    })
}

fn control_integer(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Option<i64> {
    bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .and_then(|metadata| metadata.attributes.get(property))
            .and_then(toml::Value::as_integer)
    })
}

fn control_attribute<'a>(
    bridge: &'a BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Option<&'a toml::Value> {
    bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .and_then(|metadata| metadata.attributes.get(property))
    })
}

fn assert_virtual_row_repeat(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    prototype_control_id: &str,
    virtual_control_prefix: &str,
    authored_count: i64,
    node_path_namespace: &str,
) {
    let repeat = control_attribute(bridge, control_id, UI_V2_REPEAT_ATTRIBUTE)
        .and_then(toml::Value::as_table)
        .unwrap_or_else(|| panic!("{control_id} should expose a repeat declaration"));
    assert_eq!(
        repeat
            .get(UI_V2_REPEAT_FIELD_KIND)
            .and_then(toml::Value::as_str),
        Some(UI_V2_REPEAT_KIND_VIRTUAL_ROWS)
    );
    assert_eq!(
        repeat
            .get(UI_V2_REPEAT_FIELD_PROTOTYPE)
            .and_then(toml::Value::as_str),
        Some(prototype_control_id)
    );
    assert_eq!(
        repeat
            .get(UI_V2_REPEAT_FIELD_VIRTUAL_CONTROL_PREFIX)
            .and_then(toml::Value::as_str),
        Some(virtual_control_prefix)
    );
    assert_eq!(
        repeat
            .get(UI_V2_REPEAT_FIELD_AUTHORED_COUNT)
            .and_then(toml::Value::as_integer),
        Some(authored_count)
    );
    assert_eq!(
        repeat
            .get(UI_V2_REPEAT_FIELD_NODE_PATH_NAMESPACE)
            .and_then(toml::Value::as_str),
        Some(node_path_namespace)
    );
}

fn control_center(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> UiPoint {
    let frame = bridge
        .control_frame(control_id)
        .unwrap_or_else(|| panic!("{control_id} should have a frame"));
    UiPoint::new(frame.x + frame.width * 0.5, frame.y + frame.height * 0.5)
}

fn control_component_focused(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> bool {
    let Some(node_id) = bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .map(|_| node.node_id)
    }) else {
        return false;
    };
    bridge.surface().focus.focused == Some(node_id)
        && bridge
            .surface()
            .component_state(node_id)
            .is_some_and(|state| state.flags.focused)
}

fn control_visibility(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> Option<UiVisibility> {
    bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .map(|_| node.visibility)
    })
}

fn render_background_for_control(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> Option<String> {
    let node_id = bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .map(|_| node.node_id)
    })?;
    bridge
        .surface()
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == node_id)
        .and_then(|command| command.style.background_color.clone())
}

fn render_border_for_control(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> Option<String> {
    let node_id = bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .map(|_| node.node_id)
    })?;
    bridge
        .surface()
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == node_id)
        .and_then(|command| command.style.border_color.clone())
}

fn style_color_u8(color: Option<&UiStyleColor>) -> Option<[u8; 4]> {
    match color? {
        UiStyleColor::Rgba(color) => Some(color.to_u8()),
        UiStyleColor::Transparent => Some([0, 0, 0, 0]),
        UiStyleColor::Role(_) | UiStyleColor::Inherit => None,
    }
}
