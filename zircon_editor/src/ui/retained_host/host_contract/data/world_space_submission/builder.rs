use crate::ui::retained_host::primitives::ModelRc;

use super::super::{HostWindowSceneData, PaneData, TemplatePaneNodeData};
use super::model::WorldSpaceUiSurfaceSubmission;

pub(crate) fn build_world_space_ui_surface_submissions(
    surface_id: impl Into<String>,
    nodes: &ModelRc<TemplatePaneNodeData>,
) -> Vec<WorldSpaceUiSurfaceSubmission> {
    let surface_id = surface_id.into();
    let mut submissions = (0..nodes.row_count())
        .filter_map(|index| nodes.row_data(index))
        .filter(|node| node.world_space_enabled)
        .filter_map(|node| world_space_submission_for_node(&surface_id, node))
        .collect::<Vec<_>>();

    submissions.sort_by(|left, right| {
        left.render_order
            .cmp(&right.render_order)
            .then_with(|| left.node_id.cmp(&right.node_id))
            .then_with(|| left.control_id.cmp(&right.control_id))
    });
    submissions
}

pub(crate) fn build_world_space_ui_surface_submissions_from_host_scene(
    scene: &HostWindowSceneData,
) -> Vec<WorldSpaceUiSurfaceSubmission> {
    let mut submissions = Vec::new();

    extend_world_space_pane_submissions("left-dock", &scene.left_dock.pane, &mut submissions);
    extend_world_space_pane_submissions(
        "document-dock",
        &scene.document_dock.pane,
        &mut submissions,
    );
    extend_world_space_pane_submissions("right-dock", &scene.right_dock.pane, &mut submissions);
    extend_world_space_pane_submissions("bottom-dock", &scene.bottom_dock.pane, &mut submissions);

    for index in 0..scene.floating_layer.floating_windows.row_count() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(index) else {
            continue;
        };
        let surface_id = format!("floating-window:{}", window.window_id);
        submissions.extend(build_world_space_ui_surface_submissions(
            format!("{surface_id}:header"),
            &window.header_nodes,
        ));
        extend_world_space_pane_submissions(&surface_id, &window.active_pane, &mut submissions);
    }

    submissions.sort_by(|left, right| {
        left.render_order
            .cmp(&right.render_order)
            .then_with(|| left.surface_id.cmp(&right.surface_id))
            .then_with(|| left.node_id.cmp(&right.node_id))
            .then_with(|| left.control_id.cmp(&right.control_id))
    });
    submissions
}

fn extend_world_space_pane_submissions(
    surface_id: &str,
    pane: &PaneData,
    submissions: &mut Vec<WorldSpaceUiSurfaceSubmission>,
) {
    let pane_surface_id = if pane.id.is_empty() {
        surface_id.to_string()
    } else {
        format!("{surface_id}:{}", pane.id)
    };

    for nodes in [
        &pane.hierarchy.nodes,
        &pane.inspector.nodes,
        &pane.console.nodes,
        &pane.assets_activity.nodes,
        &pane.asset_browser.nodes,
        &pane.welcome.nodes,
        &pane.project_overview.nodes,
        &pane.ui_asset.nodes,
        &pane.animation.nodes,
    ] {
        submissions.extend(build_world_space_ui_surface_submissions(
            pane_surface_id.clone(),
            nodes,
        ));
    }
}

fn world_space_submission_for_node(
    surface_id: &str,
    node: TemplatePaneNodeData,
) -> Option<WorldSpaceUiSurfaceSubmission> {
    let pixels_per_meter = node.world_pixels_per_meter.max(0.0);
    let world_width =
        positive_or_projected_world_extent(node.world_width, node.frame.width, pixels_per_meter);
    let world_height =
        positive_or_projected_world_extent(node.world_height, node.frame.height, pixels_per_meter);

    if world_width <= 0.0 || world_height <= 0.0 {
        return None;
    }

    Some(WorldSpaceUiSurfaceSubmission {
        surface_id: surface_id.to_string(),
        node_id: node.node_id.to_string(),
        control_id: node.control_id.to_string(),
        viewport_x: node.frame.x,
        viewport_y: node.frame.y,
        viewport_width: node.frame.width,
        viewport_height: node.frame.height,
        world_position: [
            node.world_position_x,
            node.world_position_y,
            node.world_position_z,
        ],
        world_rotation: [
            node.world_rotation_x,
            node.world_rotation_y,
            node.world_rotation_z,
        ],
        world_scale: [node.world_scale_x, node.world_scale_y, node.world_scale_z],
        world_width,
        world_height,
        pixels_per_meter,
        billboard: node.world_billboard,
        depth_test: node.world_depth_test,
        render_order: node.world_render_order,
        camera_target: node.world_camera_target.to_string(),
    })
}

fn positive_or_projected_world_extent(
    explicit: f32,
    frame_extent: f32,
    pixels_per_meter: f32,
) -> f32 {
    if explicit > 0.0 {
        explicit
    } else if frame_extent > 0.0 && pixels_per_meter > 0.0 {
        frame_extent / pixels_per_meter
    } else {
        0.0
    }
}
