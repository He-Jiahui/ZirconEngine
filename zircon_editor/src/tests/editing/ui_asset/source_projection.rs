use super::support::*;

#[test]
fn ui_asset_editor_session_projects_selection_indices_source_summary_and_canvas_frames() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(pane.hierarchy_selected_index, 1);
    assert!(pane.preview_selected_index >= 0);
    assert_eq!(pane.preview_surface_width, 640.0);
    assert_eq!(pane.preview_surface_height, 360.0);
    let selected_canvas_node = pane
        .preview_canvas_items
        .iter()
        .find(|item| item.selected)
        .expect("selected canvas node");
    assert_eq!(selected_canvas_node.node_id, "button");
    assert_eq!(selected_canvas_node.label, "SaveButton");
    assert!(selected_canvas_node.width > 0.0);
    assert!(selected_canvas_node.height > 0.0);
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
    assert!(pane.source_selected_line > 0);
    assert!(pane.source_selected_excerpt.contains("[nodes.button]"));
    assert!(pane.source_roundtrip_status.contains("line"));
}

#[test]
fn ui_asset_editor_session_selects_same_node_from_preview_canvas_projection() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let selected_preview_index = session.pane_presentation().preview_selected_index;
    assert!(selected_preview_index >= 0);

    session
        .select_preview_index(selected_preview_index as usize)
        .expect("select preview node");
    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.hierarchy_selected_index, 1);
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
}

#[test]
fn ui_asset_editor_session_selects_same_node_from_source_outline_projection() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let pane = session.pane_presentation();
    let outline_index = pane.source_outline_selected_index;
    assert!(outline_index >= 0);
    assert!(pane
        .source_outline_items
        .iter()
        .any(|entry| entry.contains("[nodes.button]")));

    session
        .select_source_outline_index(outline_index as usize)
        .expect("select source outline node");
    let roundtripped = session.pane_presentation();
    assert_eq!(roundtripped.inspector_selected_node_id, "button");
    assert_eq!(
        roundtripped.preview_selected_index,
        pane.preview_selected_index
    );
    assert_eq!(roundtripped.source_selected_block_label, "[nodes.button]");
}

#[test]
fn ui_asset_editor_session_selects_same_node_from_source_line_inside_block() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let selected_line = session.pane_presentation().source_selected_line;
    assert!(selected_line > 0);

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    session
        .select_source_line((selected_line + 1) as usize)
        .expect("select node from source line");

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.hierarchy_selected_index, 1);
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
}

#[test]
fn ui_asset_editor_session_selects_same_node_from_source_byte_offset_inside_block() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let selected_line = session.pane_presentation().source_selected_line;
    assert!(selected_line > 0);
    let byte_offset =
        byte_offset_for_line(session.source_buffer().text(), (selected_line + 1) as usize);

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    assert!(session
        .select_source_byte_offset(byte_offset)
        .expect("select node from source byte offset"));

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.hierarchy_selected_index, 1);
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
}

#[test]
fn ui_asset_editor_session_ignores_source_byte_offset_outside_node_blocks() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    assert!(!session
        .select_source_byte_offset(0)
        .expect("offset outside node block should no-op"));

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.hierarchy_selected_index, 1);
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
}

#[test]
fn ui_asset_editor_session_rejects_source_line_outside_node_blocks() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    let error = session
        .select_source_line(1)
        .expect_err("source line outside any node block should fail");

    assert!(
        matches!(
            error,
            UiAssetEditorSessionError::InvalidSelectionIndex { index: 1 }
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn ui_asset_editor_session_tracks_source_cursor_line_inside_selected_block() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let block_line = session.pane_presentation().source_selected_line as usize;
    let target_line = block_line + 2;
    let byte_offset = byte_offset_for_line(session.source_buffer().text(), target_line);

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    assert!(session
        .select_source_byte_offset(byte_offset)
        .expect("select source byte offset"));

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
    assert_eq!(pane.source_selected_line, target_line as i32);
    assert_eq!(pane.source_cursor_byte_offset, byte_offset as i32);
}

#[test]
fn ui_asset_editor_session_preserves_source_cursor_line_through_valid_source_roundtrip() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let block_line = session.pane_presentation().source_selected_line as usize;
    let target_line = block_line + 2;
    let byte_offset = byte_offset_for_line(session.source_buffer().text(), target_line);

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    session
        .select_source_byte_offset(byte_offset)
        .expect("select source byte offset");

    let valid_source =
        session
            .source_buffer()
            .text()
            .replacen("[nodes.button]\n", "\n[nodes.button]\n", 1);
    session
        .apply_command(UiAssetEditorCommand::edit_source(valid_source))
        .expect("apply valid source edit");

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
    assert_eq!(pane.source_selected_line, target_line as i32 + 1);
    assert_eq!(pane.source_cursor_byte_offset, byte_offset as i32 + 1);
    assert!(pane.source_selected_excerpt.contains("[nodes.button]"));
}

#[test]
fn ui_asset_editor_session_undo_restores_source_cursor_line_within_selected_block() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    let palette_index = session
        .pane_presentation()
        .palette_items
        .iter()
        .position(|item| item == "Native / Button")
        .expect("button palette item");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let block_line = session.pane_presentation().source_selected_line as usize;
    let target_line = block_line + 2;
    let byte_offset = byte_offset_for_line(session.source_buffer().text(), target_line);
    session
        .select_source_byte_offset(byte_offset)
        .expect("select source byte offset");
    assert_eq!(
        session.pane_presentation().source_selected_line,
        target_line as i32
    );

    assert!(session
        .select_palette_index(palette_index)
        .expect("select palette item"));
    assert!(session
        .insert_selected_palette_item_after_selection()
        .expect("insert button after selection"));
    assert_eq!(
        session.pane_presentation().inspector_selected_node_id,
        "button_2"
    );

    assert!(session.undo().expect("undo tree edit"));
    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
    assert_eq!(pane.source_selected_line, target_line as i32);
    assert_eq!(pane.source_cursor_byte_offset, byte_offset as i32);
}

#[test]
fn ui_asset_editor_session_falls_back_to_last_valid_source_selection_when_source_is_invalid() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let block_line = session.pane_presentation().source_selected_line as usize;
    let target_line = block_line + 2;
    let byte_offset = byte_offset_for_line(session.source_buffer().text(), target_line);

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    session
        .select_source_byte_offset(byte_offset)
        .expect("select source byte offset");

    let invalid_source =
        session
            .source_buffer()
            .text()
            .replacen("[nodes.button]", "[nodes.button", 1);
    session
        .apply_command(UiAssetEditorCommand::edit_source(invalid_source))
        .expect("apply invalid source edit");

    let pane = session.pane_presentation();
    let expected_invalid_cursor_offset =
        byte_offset_for_line(session.source_buffer().text(), target_line);
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
    assert_eq!(pane.source_selected_line, target_line as i32);
    assert_eq!(
        pane.source_cursor_byte_offset,
        expected_invalid_cursor_offset as i32
    );
    assert!(pane.source_selected_excerpt.contains("[nodes.button]"));
    assert!(pane.source_roundtrip_status.contains("last valid snapshot"));
    assert!(pane
        .source_outline_items
        .iter()
        .any(|entry| entry.contains("[nodes.button]")));
    assert!(pane.source_outline_selected_index >= 0);
}

