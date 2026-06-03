use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_viewport_scene_architecture::{
    push_back_door, push_door_core, push_side_panel_detail, push_side_stairs, push_wall_column,
    push_wall_detail_lines,
};
use super::template_viewport_scene_floor::{
    push_floor_grid_line, push_floor_panel_detail, push_floor_seam,
};
use super::template_viewport_scene_light::{
    push_beacon, push_floor_reflection, push_soft_light, push_soft_shadow, push_wall_light,
};
use super::template_viewport_scene_structure::{
    push_axis_line, push_axis_origin, push_base_surface, push_cargo_detail, push_cargo_inner_frame,
    push_floor_grate_slots, push_gizmo_center, push_handrail, push_prop_body_detail,
    push_prop_top_detail, push_rack_detail, push_selection_glow,
};
use super::template_viewport_scene_surfaces::{
    push_back_wall_surface, push_backdrop_surface, push_ceiling_surface, push_floor_surface,
};

const VIEWPORT_CONTROL_PREFIX: &str = "WorkbenchViewport";

pub(super) fn push_viewport_scene_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let Some(kind) = viewport_scene_kind(node) else {
        return false;
    };

    let rect = pixel_aligned_rect(rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    match kind {
        ViewportSceneKind::Container => {}
        ViewportSceneKind::SelectionEdge => {
            push_selection_glow(commands, &rect, clip, order, opacity);
            push_base_surface(commands, node, &rect, clip, order + 1, opacity);
        }
        ViewportSceneKind::AxisLine => {
            push_axis_line(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::AxisOrigin => {
            push_axis_origin(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorGrate => {
            push_base_surface(commands, node, &rect, clip, order, opacity);
            push_floor_grate_slots(commands, &rect, clip, order + 1, opacity);
        }
        ViewportSceneKind::Backdrop => {
            push_backdrop_surface(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::Ceiling => {
            push_ceiling_surface(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::BackWall => {
            push_back_wall_surface(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorSurface => {
            push_floor_surface(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorGrid => {
            push_floor_grid_line(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorPanel => {
            push_floor_panel_detail(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorSeam => {
            push_floor_seam(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::Cargo => {
            push_base_surface(commands, node, &rect, clip, order, opacity);
            push_cargo_detail(commands, &rect, clip, order + 1, opacity);
        }
        ViewportSceneKind::PropBody => {
            push_prop_body_detail(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::PropTop => {
            push_prop_top_detail(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::CargoInner => {
            push_cargo_inner_frame(commands, &rect, clip, order, opacity);
        }
        ViewportSceneKind::Rack => {
            push_base_surface(commands, node, &rect, clip, order, opacity);
            push_rack_detail(commands, &rect, clip, order + 1, opacity);
        }
        ViewportSceneKind::SidePanel => {
            push_side_panel_detail(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::SideStairs => {
            push_side_stairs(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::WallDetail => {
            push_wall_detail_lines(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::BackDoor => {
            push_back_door(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::DoorCore => {
            push_door_core(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::WallColumn => {
            push_wall_column(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::Handrail => {
            push_handrail(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::SoftLight => {
            push_soft_light(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::SoftShadow => {
            push_soft_shadow(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::FloorReflection => {
            push_floor_reflection(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::WallLight => {
            push_wall_light(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::Beacon => {
            push_beacon(commands, node, &rect, clip, order, opacity);
        }
        ViewportSceneKind::GizmoCenter => {
            push_gizmo_center(commands, &rect, clip, order, opacity);
        }
        ViewportSceneKind::SceneLayer => {
            push_base_surface(commands, node, &rect, clip, order, opacity);
        }
    }
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewportSceneKind {
    Container,
    SceneLayer,
    Backdrop,
    Ceiling,
    BackWall,
    FloorSurface,
    FloorGrate,
    FloorGrid,
    FloorPanel,
    FloorSeam,
    Cargo,
    PropBody,
    PropTop,
    CargoInner,
    Rack,
    SidePanel,
    SideStairs,
    WallDetail,
    BackDoor,
    DoorCore,
    WallColumn,
    Handrail,
    SoftLight,
    SoftShadow,
    FloorReflection,
    WallLight,
    Beacon,
    SelectionEdge,
    AxisLine,
    AxisOrigin,
    GizmoCenter,
}

fn viewport_scene_kind(node: &TemplatePaneNodeData) -> Option<ViewportSceneKind> {
    let id = node.control_id.as_str();
    if !id.starts_with(VIEWPORT_CONTROL_PREFIX) || is_viewport_chrome_node(id) {
        return None;
    }

    if matches!(
        id,
        "WorkbenchViewportAxisXLabel"
            | "WorkbenchViewportAxisYLabel"
            | "WorkbenchViewportGizmoX"
            | "WorkbenchViewportGizmoY"
            | "WorkbenchViewportGizmoZ"
    ) {
        return None;
    }

    if id.contains("Lightwash") {
        Some(ViewportSceneKind::SoftLight)
    } else if id.contains("Shadow") {
        Some(ViewportSceneKind::SoftShadow)
    } else if id.contains("FloorReflection") {
        Some(ViewportSceneKind::FloorReflection)
    } else if id.contains("WallLight") {
        Some(ViewportSceneKind::WallLight)
    } else if id.contains("Beacon") {
        Some(ViewportSceneKind::Beacon)
    } else if id.contains("Selection") {
        Some(ViewportSceneKind::SelectionEdge)
    } else if id == "WorkbenchViewportAxisOrigin" {
        Some(ViewportSceneKind::AxisOrigin)
    } else if id.contains("AxisX") || id.contains("AxisY") || id.contains("AxisZ") {
        Some(ViewportSceneKind::AxisLine)
    } else if id == "WorkbenchViewportSurface" || id == "WorkbenchViewportGizmoPanel" {
        Some(ViewportSceneKind::Container)
    } else if id == "WorkbenchViewportBackdrop" {
        Some(ViewportSceneKind::Backdrop)
    } else if id == "WorkbenchViewportCeiling" {
        Some(ViewportSceneKind::Ceiling)
    } else if id == "WorkbenchViewportBackWall" {
        Some(ViewportSceneKind::BackWall)
    } else if id == "WorkbenchViewportFloor" {
        Some(ViewportSceneKind::FloorSurface)
    } else if id.contains("Grid") {
        Some(ViewportSceneKind::FloorGrid)
    } else if id.contains("FloorPanel") {
        Some(ViewportSceneKind::FloorPanel)
    } else if id.contains("FloorSeam") {
        Some(ViewportSceneKind::FloorSeam)
    } else if id.contains("FloorGrate") {
        Some(ViewportSceneKind::FloorGrate)
    } else if id == "WorkbenchViewportPropBody" {
        Some(ViewportSceneKind::PropBody)
    } else if id == "WorkbenchViewportPropTop" {
        Some(ViewportSceneKind::PropTop)
    } else if id.contains("Cargo") && id.contains("Inner") {
        Some(ViewportSceneKind::CargoInner)
    } else if id.contains("Cargo") {
        Some(ViewportSceneKind::Cargo)
    } else if id.contains("Rack") {
        Some(ViewportSceneKind::Rack)
    } else if id.contains("SideLeftStairs") {
        Some(ViewportSceneKind::SideStairs)
    } else if id == "WorkbenchViewportSideLeft" || id == "WorkbenchViewportSideRight" {
        Some(ViewportSceneKind::SidePanel)
    } else if id.contains("WallDetail") {
        Some(ViewportSceneKind::WallDetail)
    } else if id == "WorkbenchViewportBackDoor" {
        Some(ViewportSceneKind::BackDoor)
    } else if id == "WorkbenchViewportDoorCore" {
        Some(ViewportSceneKind::DoorCore)
    } else if id.contains("WallColumn") {
        Some(ViewportSceneKind::WallColumn)
    } else if id.contains("Handrail") {
        Some(ViewportSceneKind::Handrail)
    } else if id == "WorkbenchViewportGizmoCenter" {
        Some(ViewportSceneKind::GizmoCenter)
    } else {
        Some(ViewportSceneKind::SceneLayer)
    }
}

fn is_viewport_chrome_node(id: &str) -> bool {
    matches!(
        id,
        "WorkbenchViewportPanel"
            | "WorkbenchViewportToolbar"
            | "WorkbenchViewportToolbarFill"
            | "WorkbenchViewportMode"
            | "WorkbenchViewportLit"
            | "WorkbenchViewportAngle"
            | "WorkbenchViewportSpeed"
    )
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(0.0),
        height: rect.height.round().max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::data::TemplateNodeFrameData;
    use super::super::template_nodes::{
        paint_template_nodes_for_test, paint_template_nodes_for_test_with_background,
    };
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn viewport_floor_grate_paints_repeating_native_slots() {
        let bytes = paint_template_nodes_for_test(
            80,
            80,
            model_rc(vec![styled_node(
                "WorkbenchViewportFloorGrateRight",
                10.0,
                8.0,
                42.0,
                48.0,
                [0, 0, 0, 140],
            )]),
        );

        assert_ne!(pixel_at(&bytes, 80, 16, 20), [0, 0, 0, 255]);
        assert_ne!(pixel_at(&bytes, 80, 20, 20), pixel_at(&bytes, 80, 16, 20));
    }

    #[test]
    fn viewport_handrail_paints_posts_beyond_authored_rail_rect() {
        let bytes = paint_template_nodes_for_test(
            140,
            80,
            model_rc(vec![styled_node(
                "WorkbenchViewportHandrailLeft",
                10.0,
                10.0,
                100.0,
                4.0,
                [179, 113, 48, 122],
            )]),
        );

        assert_ne!(pixel_at(&bytes, 140, 47, 36), [0, 0, 0, 255]);
        assert_eq!(pixel_at(&bytes, 140, 20, 36), [0, 0, 0, 255]);
    }

    #[test]
    fn viewport_gizmo_center_paints_native_axis_rod_and_facets() {
        let bytes = paint_template_nodes_for_test(
            96,
            96,
            model_rc(vec![styled_node(
                "WorkbenchViewportGizmoCenter",
                36.0,
                42.0,
                36.0,
                31.0,
                [49, 93, 159, 255],
            )]),
        );

        assert_ne!(pixel_at(&bytes, 96, 54, 24), [0, 0, 0, 255]);
        assert_ne!(pixel_at(&bytes, 96, 42, 46), pixel_at(&bytes, 96, 42, 68));
        assert_ne!(pixel_at(&bytes, 96, 68, 58), pixel_at(&bytes, 96, 42, 58));
    }

    #[test]
    fn viewport_axis_line_uses_authored_background_color_before_axis_fallback() {
        let bytes = paint_template_nodes_for_test(
            96,
            48,
            model_rc(vec![styled_node(
                "WorkbenchViewportAxisX",
                12.0,
                20.0,
                60.0,
                4.0,
                [10, 80, 190, 255],
            )]),
        );

        let line = pixel_at(&bytes, 96, 34, 22);
        assert!(line[2] > line[0]);
    }

    #[test]
    fn viewport_soft_light_paints_layered_center_intensity() {
        let bytes = paint_template_nodes_for_test(
            140,
            120,
            model_rc(vec![styled_node(
                "WorkbenchViewportLightwashCenter",
                20.0,
                20.0,
                90.0,
                60.0,
                [174, 198, 211, 96],
            )]),
        );

        assert!(luma(pixel_at(&bytes, 140, 65, 50)) > luma(pixel_at(&bytes, 140, 22, 22)));
    }

    #[test]
    fn viewport_soft_shadow_darkens_toward_center() {
        let bytes = paint_template_nodes_for_test_with_background(
            140,
            120,
            [48, 54, 58, 255],
            model_rc(vec![styled_node(
                "WorkbenchViewportShadowTopBay",
                20.0,
                20.0,
                90.0,
                60.0,
                [0, 0, 0, 128],
            )]),
        );

        let center = pixel_at(&bytes, 140, 65, 50);
        assert!(luma(center) < luma([48, 54, 58, 255]));
        assert!(luma(center) < luma(pixel_at(&bytes, 140, 22, 22)));
    }

    #[test]
    fn viewport_wall_light_paints_hot_core_over_soft_strip() {
        let bytes = paint_template_nodes_for_test(
            120,
            72,
            model_rc(vec![styled_node(
                "WorkbenchViewportWallLightFarRight",
                30.0,
                30.0,
                56.0,
                8.0,
                [217, 230, 233, 144],
            )]),
        );

        assert!(luma(pixel_at(&bytes, 120, 58, 31)) > luma(pixel_at(&bytes, 120, 58, 36)));
    }

    #[test]
    fn viewport_beacon_paints_hot_inner_strip() {
        let bytes = paint_template_nodes_for_test(
            80,
            96,
            model_rc(vec![styled_node(
                "WorkbenchViewportWallBeaconLeft",
                36.0,
                18.0,
                8.0,
                56.0,
                [225, 148, 80, 168],
            )]),
        );

        assert!(luma(pixel_at(&bytes, 80, 39, 46)) > luma(pixel_at(&bytes, 80, 36, 46)));
    }

    #[test]
    fn viewport_grid_line_paints_native_glow_band() {
        let bytes = paint_template_nodes_for_test(
            120,
            80,
            model_rc(vec![styled_node(
                "WorkbenchViewportGridH2",
                20.0,
                40.0,
                80.0,
                1.0,
                [145, 155, 157, 87],
            )]),
        );

        assert!(luma(pixel_at(&bytes, 120, 48, 40)) > luma(pixel_at(&bytes, 120, 48, 39)));
        assert_ne!(pixel_at(&bytes, 120, 48, 39), [0, 0, 0, 255]);
    }

    #[test]
    fn viewport_back_door_paints_inset_panel_lines() {
        let bytes = paint_template_nodes_for_test(
            120,
            96,
            model_rc(vec![styled_node(
                "WorkbenchViewportBackDoor",
                20.0,
                18.0,
                80.0,
                50.0,
                [75, 85, 89, 214],
            )]),
        );

        assert!(luma(pixel_at(&bytes, 120, 32, 26)) > luma(pixel_at(&bytes, 120, 32, 32)));
        assert_ne!(pixel_at(&bytes, 120, 60, 42), pixel_at(&bytes, 120, 32, 42));
    }

    #[test]
    fn viewport_wall_detail_paints_internal_line_grid() {
        let bytes = paint_template_nodes_for_test(
            140,
            120,
            model_rc(vec![styled_node(
                "WorkbenchViewportWallDetailCenterLines",
                20.0,
                18.0,
                100.0,
                80.0,
                [170, 190, 199, 51],
            )]),
        );

        assert!(luma(pixel_at(&bytes, 140, 48, 49)) > luma(pixel_at(&bytes, 140, 48, 24)));
        assert_ne!(pixel_at(&bytes, 140, 70, 28), pixel_at(&bytes, 140, 48, 24));
    }

    #[test]
    fn viewport_side_stairs_paints_step_lines() {
        let bytes = paint_template_nodes_for_test(
            120,
            96,
            model_rc(vec![styled_node(
                "WorkbenchViewportSideLeftStairs",
                20.0,
                18.0,
                90.0,
                60.0,
                [184, 194, 194, 56],
            )]),
        );

        assert!(luma(pixel_at(&bytes, 120, 34, 28)) > luma(pixel_at(&bytes, 120, 34, 24)));
    }

    #[test]
    fn viewport_cargo_inner_paints_frame_without_cargo_body_fill() {
        let bytes = paint_template_nodes_for_test(
            120,
            96,
            model_rc(vec![styled_node(
                "WorkbenchViewportCargoRightInner",
                20.0,
                18.0,
                80.0,
                44.0,
                [0, 0, 0, 0],
            )]),
        );

        assert_ne!(pixel_at(&bytes, 120, 20, 18), [0, 0, 0, 255]);
        assert_ne!(pixel_at(&bytes, 120, 47, 28), [0, 0, 0, 255]);
        assert_eq!(pixel_at(&bytes, 120, 58, 30), [0, 0, 0, 255]);
    }

    #[test]
    fn viewport_selected_prop_paints_box_facets_instead_of_cargo_slots() {
        let bytes = paint_template_nodes_for_test(
            120,
            96,
            model_rc(vec![
                styled_node(
                    "WorkbenchViewportPropTop",
                    24.0,
                    18.0,
                    64.0,
                    16.0,
                    [42, 48, 53, 255],
                ),
                styled_node(
                    "WorkbenchViewportPropBody",
                    24.0,
                    34.0,
                    64.0,
                    44.0,
                    [32, 37, 40, 255],
                ),
            ]),
        );

        assert!(luma(pixel_at(&bytes, 120, 44, 37)) > luma(pixel_at(&bytes, 120, 44, 50)));
        assert!(luma(pixel_at(&bytes, 120, 85, 50)) < luma(pixel_at(&bytes, 120, 44, 50)));
        assert!(luma(pixel_at(&bytes, 120, 44, 21)) > luma(pixel_at(&bytes, 120, 44, 30)));
    }

    #[test]
    fn viewport_ceiling_surface_paints_ribs_and_lower_shadow() {
        let bytes = paint_template_nodes_for_test(
            120,
            80,
            model_rc(vec![styled_node(
                "WorkbenchViewportCeiling",
                10.0,
                10.0,
                100.0,
                40.0,
                [21, 26, 29, 255],
            )]),
        );

        assert_ne!(pixel_at(&bytes, 120, 52, 20), pixel_at(&bytes, 120, 50, 20));
        assert!(luma(pixel_at(&bytes, 120, 60, 48)) < luma(pixel_at(&bytes, 120, 60, 30)));
    }

    #[test]
    fn viewport_back_wall_surface_paints_panel_grid() {
        let bytes = paint_template_nodes_for_test(
            140,
            120,
            model_rc(vec![styled_node(
                "WorkbenchViewportBackWall",
                20.0,
                18.0,
                100.0,
                80.0,
                [27, 29, 31, 255],
            )]),
        );

        assert_ne!(pixel_at(&bytes, 140, 70, 50), pixel_at(&bytes, 140, 58, 50));
        assert_ne!(pixel_at(&bytes, 140, 42, 45), pixel_at(&bytes, 140, 42, 44));
    }

    #[test]
    fn viewport_floor_surface_paints_depth_lines_and_bottom_shadow() {
        let bytes = paint_template_nodes_for_test(
            140,
            100,
            model_rc(vec![styled_node(
                "WorkbenchViewportFloor",
                10.0,
                10.0,
                110.0,
                64.0,
                [35, 38, 37, 255],
            )]),
        );

        assert_ne!(pixel_at(&bytes, 140, 58, 46), pixel_at(&bytes, 140, 58, 45));
        assert!(luma(pixel_at(&bytes, 140, 60, 70)) < luma(pixel_at(&bytes, 140, 60, 34)));
    }

    #[test]
    fn viewport_layout_containers_do_not_paint_fallback_surface() {
        let bytes = paint_template_nodes_for_test_with_background(
            90,
            72,
            [9, 11, 13, 255],
            model_rc(vec![
                styled_node(
                    "WorkbenchViewportSurface",
                    8.0,
                    8.0,
                    72.0,
                    48.0,
                    [0, 0, 0, 0],
                ),
                styled_node(
                    "WorkbenchViewportGizmoPanel",
                    18.0,
                    18.0,
                    32.0,
                    24.0,
                    [0, 0, 0, 0],
                ),
            ]),
        );

        assert_eq!(pixel_at(&bytes, 90, 20, 20), [9, 11, 13, 255]);
        assert_eq!(pixel_at(&bytes, 90, 70, 50), [9, 11, 13, 255]);
    }

    fn styled_node(
        control_id: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        background: [u8; 4],
    ) -> TemplatePaneNodeData {
        let mut node = TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "Pane".into(),
            frame: TemplateNodeFrameData {
                x,
                y,
                width,
                height,
            },
            ..TemplatePaneNodeData::default()
        };
        node.button_style.element.background_color = Some(UiStyleColor::Rgba(
            UiRgbaColor::from_u8(background[0], background[1], background[2], background[3]),
        ));
        node
    }

    fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
        let index = ((y as usize * frame_width as usize) + x as usize) * 4;
        [
            bytes[index],
            bytes[index + 1],
            bytes[index + 2],
            bytes[index + 3],
        ]
    }

    fn luma(pixel: [u8; 4]) -> u16 {
        pixel[0] as u16 + pixel[1] as u16 + pixel[2] as u16
    }
}
