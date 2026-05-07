use slint::{Model, ModelRc};

use super::{HostWindowSceneData, PaneData, TemplatePaneNodeData};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WorldSpaceUiSurfaceSubmission {
    pub surface_id: String,
    pub node_id: String,
    pub control_id: String,
    pub viewport_x: f32,
    pub viewport_y: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub world_position: [f32; 3],
    pub world_rotation: [f32; 3],
    pub world_scale: [f32; 3],
    pub world_width: f32,
    pub world_height: f32,
    pub pixels_per_meter: f32,
    pub billboard: bool,
    pub depth_test: bool,
    pub render_order: i32,
    pub camera_target: String,
}

impl WorldSpaceUiSurfaceSubmission {
    pub(crate) fn contains_viewport_point(&self, x: f32, y: f32) -> bool {
        x >= self.viewport_x
            && y >= self.viewport_y
            && x <= self.viewport_x + self.viewport_width.max(0.0)
            && y <= self.viewport_y + self.viewport_height.max(0.0)
    }
}

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

#[cfg(test)]
mod tests {
    use crate::ui::layouts::common::model_rc;
    use slint::SharedString;

    use super::*;
    use crate::ui::slint_host::host_contract::TemplateNodeFrameData;

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
}
