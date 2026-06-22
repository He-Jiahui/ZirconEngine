use super::super::super::HostWindowSceneData;
use super::super::model::WorldSpaceUiSurfaceSubmission;
use super::node::build_world_space_ui_surface_submissions;
use super::pane::extend_world_space_pane_submissions;

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
