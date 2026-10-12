use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::{TemplateNodeFrameData, TemplatePaneNodeData};
use crate::ui::retained_host::primitives::SharedString;

use super::*;

#[test]
fn world_space_ui_surface_submissions_collect_enabled_render_candidates() {
    let nodes = model_rc(vec![
        world_node("late", "LateSurface", 20, 2.0, 1.0),
        screen_node("screen-only"),
        world_node("early", "EarlySurface", 4, 4.0, 2.0),
    ]);

    let submissions = build_world_space_ui_surface_submissions("viewport-main", &nodes);

    assert_eq!(submissions.len(), 2);
    assert_eq!(submissions[0].node_id, "early");
    assert_eq!(submissions[0].surface_id, "viewport-main");
    assert_eq!(submissions[0].control_id, "EarlySurface");
    assert_eq!(submissions[0].world_width, 4.0);
    assert_eq!(submissions[0].world_height, 2.0);
    assert_eq!(submissions[1].node_id, "late");
}

#[test]
fn world_space_ui_surface_submissions_project_size_from_frame_when_world_size_missing() {
    let mut node = world_node("projected", "ProjectedSurface", 0, 0.0, 0.0);
    node.frame.width = 256.0;
    node.frame.height = 128.0;
    node.world_pixels_per_meter = 64.0;
    let nodes = model_rc(vec![node]);

    let submissions = build_world_space_ui_surface_submissions("viewport-main", &nodes);

    assert_eq!(submissions.len(), 1);
    assert_eq!(submissions[0].world_width, 4.0);
    assert_eq!(submissions[0].world_height, 2.0);
}

#[test]
fn world_space_ui_surface_submission_exposes_viewport_hit_bounds_without_rhi() {
    let nodes = model_rc(vec![world_node("hit", "HitSurface", 0, 2.0, 1.0)]);
    let submissions = build_world_space_ui_surface_submissions("viewport-main", &nodes);

    assert!(submissions[0].contains_viewport_point(16.0, 20.0));
    assert!(!submissions[0].contains_viewport_point(400.0, 20.0));
}

fn world_node(
    node_id: &'static str,
    control_id: &'static str,
    render_order: i32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: SharedString::from(node_id),
        control_id: SharedString::from(control_id),
        world_space_enabled: true,
        world_position_x: 1.0,
        world_position_y: 2.0,
        world_position_z: 3.0,
        world_rotation_x: 10.0,
        world_rotation_y: 20.0,
        world_rotation_z: 30.0,
        world_scale_x: 1.0,
        world_scale_y: 1.0,
        world_scale_z: 1.0,
        world_width: width,
        world_height: height,
        world_pixels_per_meter: 128.0,
        world_billboard: true,
        world_depth_test: true,
        world_render_order: render_order,
        world_camera_target: SharedString::from("viewport-main"),
        frame: TemplateNodeFrameData {
            x: 8.0,
            y: 16.0,
            width: 320.0,
            height: 180.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn screen_node(node_id: &'static str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: SharedString::from(node_id),
        control_id: SharedString::from("ScreenSurface"),
        world_space_enabled: false,
        ..TemplatePaneNodeData::default()
    }
}
