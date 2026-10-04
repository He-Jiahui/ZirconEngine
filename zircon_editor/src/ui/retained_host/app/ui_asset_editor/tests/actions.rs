use super::ui_asset_action_requires_global_presentation;

#[test]
fn ui_asset_global_presentation_actions_cover_workspace_and_cross_view_routes() {
    for action_id in [
        "save",
        "workspace.keep_local_and_save",
        "canvas.promote.widget",
        "theme.local.promote",
        "theme.source.open",
        "reference.open",
        "emergency.open_asset_browser",
    ] {
        assert!(ui_asset_action_requires_global_presentation(action_id));
    }
    assert!(!ui_asset_action_requires_global_presentation(
        "mode.preview"
    ));
    assert!(!ui_asset_action_requires_global_presentation(
        "canvas.move.up"
    ));
}
