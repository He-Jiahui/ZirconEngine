use super::*;

#[test]
fn ui_asset_editor_session_routes_palette_drop_into_local_component_mounts() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/local-component-drop.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LOCAL_COMPONENT_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("local component drop session");

    select_palette_entry(&mut session, "Native / Button");
    let root_frame = preview_frame(&session, "card");
    assert!(session
        .update_palette_drag_target(
            root_frame.x + root_frame.width * 0.5,
            root_frame.y + root_frame.height * 0.8,
        )
        .expect("hover local component root"));

    let targeted = session.pane_presentation();
    assert_eq!(targeted.palette_drag_target_label, "Insert Body Slot");

    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("drop palette item into local component"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let inserted_mount = document
        .node("card")
        .and_then(|node| node.children.first())
        .expect("inserted local component child mount");
    assert_eq!(inserted_mount.mount.as_deref(), Some("body"));
}

#[test]
fn ui_asset_editor_session_routes_palette_drop_into_external_widget_named_slots() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/external-widget-drop.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        EXTERNAL_WIDGET_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("external widget drop session");
    let imported_widget =
        crate::tests::support::load_test_ui_asset(IMPORTED_TOOLBAR_SHELL_WIDGET_ASSET_TOML)
            .expect("imported toolbar shell");
    session
        .register_widget_import(
            "asset://ui/common/toolbar_shell.ui#ToolbarShell",
            imported_widget,
        )
        .expect("register imported toolbar shell");

    select_palette_entry(&mut session, "Native / Button");
    let toolbar_frame = preview_frame(&session, "toolbar");
    assert!(session
        .update_palette_drag_target(
            toolbar_frame.x + toolbar_frame.width * 0.85,
            toolbar_frame.y + toolbar_frame.height * 0.5,
        )
        .expect("hover toolbar reference"));

    let targeted = session.pane_presentation();
    assert_eq!(targeted.palette_drag_target_label, "Insert Trailing Slot");

    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("drop palette item into external widget reference"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let inserted_mount = document
        .node("toolbar")
        .and_then(|node| node.children.first())
        .expect("inserted external widget child mount");
    assert_eq!(inserted_mount.mount.as_deref(), Some("trailing"));
}

#[test]
fn ui_asset_editor_session_uses_explicit_slot_overlay_regions_for_low_semantic_component_mounts() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/low-semantic-component-drop.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LOW_SEMANTIC_COMPONENT_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("low semantic component drop session");

    select_palette_entry(&mut session, "Native / Button");
    let host_frame = preview_frame(&session, "host");
    assert!(session
        .update_palette_drag_target(
            host_frame.x + host_frame.width * 0.51,
            host_frame.y + host_frame.height * 0.5,
        )
        .expect("hover low semantic component middle slot overlay"));

    let targeted = session.pane_presentation();
    assert_eq!(
        targeted
            .palette_drag_slot_target_items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Slot A Slot", "Slot B Slot", "Slot C Slot"]
    );
    assert_eq!(
        targeted
            .palette_drag_slot_target_items
            .iter()
            .find(|item| item.selected)
            .map(|item| item.label.as_str()),
        Some("Slot B Slot")
    );
    assert_eq!(targeted.palette_drag_target_label, "Insert Slot B Slot");

    let original_source = session.source_buffer().text().to_string();
    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("arm sticky chooser for low semantic component"));
    assert_eq!(session.source_buffer().text(), original_source);
    assert!(session.pane_presentation().palette_target_chooser_active);
    assert!(session
        .confirm_palette_target_choice()
        .expect("confirm low semantic component target"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let inserted_mount = document
        .node("host")
        .and_then(|node| node.children.first())
        .expect("inserted low semantic component child mount");
    assert_eq!(inserted_mount.mount.as_deref(), Some("slot_b"));
}
