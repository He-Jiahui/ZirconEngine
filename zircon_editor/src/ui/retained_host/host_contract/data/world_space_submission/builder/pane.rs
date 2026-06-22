use super::super::super::PaneData;
use super::super::model::WorldSpaceUiSurfaceSubmission;
use super::node::build_world_space_ui_surface_submissions;

pub(super) fn extend_world_space_pane_submissions(
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
