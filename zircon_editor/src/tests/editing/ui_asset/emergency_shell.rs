use super::support::*;

#[test]
fn ui_asset_editor_session_projects_emergency_shell_for_invalid_source() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        SIMPLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    let edited = SIMPLE_LAYOUT_ASSET_TOML.replace("Ready", "Edited");

    session
        .apply_command(UiAssetEditorCommand::edit_source(edited.clone()))
        .expect("valid edit");
    session
        .apply_command(UiAssetEditorCommand::edit_source("not valid toml"))
        .expect("invalid source edit");

    let pane = session.pane_presentation();
    assert_eq!(pane.shell_state, "Emergency");
    assert!(!pane.emergency_summary.is_empty());
    assert!(pane.can_emergency_reload);
    assert!(pane.can_emergency_revert);
    assert!(pane.can_emergency_open_asset_browser);
    assert_eq!(pane.source_text, "not valid toml");
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Edited")
    );
}

#[test]
fn ui_asset_editor_session_reverts_emergency_source_to_last_valid_document() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        SIMPLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    let edited = SIMPLE_LAYOUT_ASSET_TOML.replace("Ready", "Edited");

    session
        .apply_command(UiAssetEditorCommand::edit_source(edited.clone()))
        .expect("valid edit");
    session
        .apply_command(UiAssetEditorCommand::edit_source("not valid toml"))
        .expect("invalid source edit");
    assert_eq!(session.pane_presentation().shell_state, "Emergency");

    assert!(session
        .revert_source_to_last_valid()
        .expect("revert to last valid"));
    let pane = session.pane_presentation();
    assert_eq!(pane.shell_state, "Valid");
    assert!(pane.emergency_summary.is_empty());
    assert_eq!(pane.source_text, edited);
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Edited")
    );
    assert!(session.can_undo());
}
