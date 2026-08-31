use super::*;

#[test]
fn ui_asset_editor_session_synthesizes_grid_slot_from_palette_drag_drop() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/grid-drop.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        GRID_DROP_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("grid drop session");

    select_palette_entry(&mut session, "Native / Button");
    let root_frame = preview_frame(&session, "root");
    assert!(session
        .update_palette_drag_target(
            root_frame.x + root_frame.width * 0.75,
            root_frame.y + root_frame.height * 0.75,
        )
        .expect("hover grid root"));

    let targeted = session.pane_presentation();
    assert_eq!(targeted.palette_drag_target_label, "Insert Grid Child");

    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("drop palette item into grid"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let inserted_mount = document
        .node("root")
        .and_then(|node| node.children.last())
        .expect("inserted grid child mount");
    assert_eq!(
        numeric_slot_value(&inserted_mount.slot, &["row"]),
        Some(2.0)
    );
    assert_eq!(
        numeric_slot_value(&inserted_mount.slot, &["column"]),
        Some(3.0)
    );
}

#[test]
fn ui_asset_editor_session_projects_explicit_grid_slot_target_overlays_for_palette_drag() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/grid-drop.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        GRID_DROP_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("grid drop session");

    select_palette_entry(&mut session, "Native / Button");
    let root_frame = preview_frame(&session, "root");
    assert!(session
        .update_palette_drag_target(
            root_frame.x + root_frame.width * 0.75,
            root_frame.y + root_frame.height * 0.75,
        )
        .expect("hover grid root"));

    let targeted = session.pane_presentation();
    assert_eq!(targeted.palette_drag_slot_target_items.len(), 15);
    assert!(targeted
        .palette_drag_slot_target_items
        .iter()
        .any(|item| item.label == "R0 C0"));
    let selected = targeted
        .palette_drag_slot_target_items
        .iter()
        .find(|item| item.selected)
        .expect("selected grid slot overlay");
    assert_eq!(selected.label, "R2 C3");
    assert!(selected.width > 0.0);
    assert!(selected.height > 0.0);
}

#[test]
fn ui_asset_editor_session_synthesizes_overlay_slot_from_palette_drag_drop() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/overlay-drop.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        OVERLAY_DROP_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("overlay drop session");

    select_palette_entry(&mut session, "Native / Button");
    let root_frame = preview_frame(&session, "root");
    assert!(session
        .update_palette_drag_target(root_frame.x + root_frame.width - 16.0, root_frame.y + 12.0,)
        .expect("hover overlay root"));

    let targeted = session.pane_presentation();
    assert_eq!(targeted.palette_drag_target_label, "Insert Overlay Child");

    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("drop palette item into overlay"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let inserted_mount = document
        .node("root")
        .and_then(|node| node.children.first())
        .expect("inserted overlay child mount");
    assert_eq!(
        numeric_slot_value(&inserted_mount.slot, &["layout", "anchor", "x"]),
        Some(1.0)
    );
    assert_eq!(
        numeric_slot_value(&inserted_mount.slot, &["layout", "anchor", "y"]),
        Some(0.0)
    );
    assert_eq!(
        numeric_slot_value(&inserted_mount.slot, &["layout", "pivot", "x"]),
        Some(1.0)
    );
    assert_eq!(
        numeric_slot_value(&inserted_mount.slot, &["layout", "pivot", "y"]),
        Some(0.0)
    );
    assert_eq!(
        numeric_slot_value(&inserted_mount.slot, &["layout", "position", "x"]),
        Some(-16.0)
    );
    assert_eq!(
        numeric_slot_value(&inserted_mount.slot, &["layout", "position", "y"]),
        Some(12.0)
    );
}

#[test]
fn ui_asset_editor_session_projects_explicit_overlay_slot_target_overlays_for_palette_drag() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/overlay-drop.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        OVERLAY_DROP_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("overlay drop session");

    select_palette_entry(&mut session, "Native / Button");
    let root_frame = preview_frame(&session, "root");
    assert!(session
        .update_palette_drag_target(root_frame.x + root_frame.width - 16.0, root_frame.y + 12.0,)
        .expect("hover overlay root"));

    let targeted = session.pane_presentation();
    assert_eq!(targeted.palette_drag_slot_target_items.len(), 9);
    assert!(targeted
        .palette_drag_slot_target_items
        .iter()
        .any(|item| item.label == "Center"));
    let selected = targeted
        .palette_drag_slot_target_items
        .iter()
        .find(|item| item.selected)
        .expect("selected overlay slot overlay");
    assert_eq!(selected.label, "Top Right");
}

#[test]
fn ui_asset_editor_session_synthesizes_flow_slot_from_palette_drag_drop() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/flow-drop.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        FLOW_DROP_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("flow drop session");

    select_palette_entry(&mut session, "Native / Button");
    let root_frame = preview_frame(&session, "root");
    assert!(session
        .update_palette_drag_target(
            root_frame.x + root_frame.width - 1.0,
            root_frame.y + root_frame.height * 0.75,
        )
        .expect("hover flow root"));

    let targeted = session.pane_presentation();
    assert_eq!(targeted.palette_drag_target_label, "Insert Flow Child");

    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("drop palette item into flow"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let inserted_mount = document
        .node("root")
        .and_then(|node| node.children.last())
        .expect("inserted flow child mount");
    assert_eq!(
        inserted_mount
            .slot
            .get("break_before")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        inserted_mount
            .slot
            .get("alignment")
            .and_then(toml::Value::as_str),
        Some("End")
    );
}

#[test]
fn ui_asset_editor_session_projects_explicit_named_slot_target_overlays_for_palette_drag() {
    let local_route = UiAssetEditorRoute::new(
        "asset://ui/tests/local-component-drop.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut local_session = UiAssetEditorSession::from_source(
        local_route,
        LOCAL_COMPONENT_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("local component drop session");

    select_palette_entry(&mut local_session, "Native / Button");
    let card_frame = preview_frame(&local_session, "card");
    assert!(local_session
        .update_palette_drag_target(
            card_frame.x + card_frame.width * 0.5,
            card_frame.y + card_frame.height * 0.8,
        )
        .expect("hover local component root"));

    let local_targeted = local_session.pane_presentation();
    assert_eq!(
        local_targeted
            .palette_drag_slot_target_items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Header Slot", "Body Slot"]
    );
    assert_eq!(
        local_targeted
            .palette_drag_slot_target_items
            .iter()
            .find(|item| item.selected)
            .map(|item| item.label.as_str()),
        Some("Body Slot")
    );

    let external_route = UiAssetEditorRoute::new(
        "asset://ui/tests/external-widget-drop.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut external_session = UiAssetEditorSession::from_source(
        external_route,
        EXTERNAL_WIDGET_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("external widget drop session");
    let imported_widget =
        crate::tests::support::load_test_ui_asset(IMPORTED_TOOLBAR_SHELL_WIDGET_ASSET_TOML)
            .expect("imported toolbar shell");
    external_session
        .register_widget_import(
            "asset://ui/common/toolbar_shell.ui#ToolbarShell",
            imported_widget,
        )
        .expect("register imported toolbar shell");

    select_palette_entry(&mut external_session, "Native / Button");
    let toolbar_frame = preview_frame(&external_session, "toolbar");
    assert!(external_session
        .update_palette_drag_target(
            toolbar_frame.x + toolbar_frame.width * 0.85,
            toolbar_frame.y + toolbar_frame.height * 0.5,
        )
        .expect("hover toolbar reference"));

    let external_targeted = external_session.pane_presentation();
    assert_eq!(
        external_targeted
            .palette_drag_slot_target_items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Leading Slot", "Trailing Slot"]
    );
    assert_eq!(
        external_targeted
            .palette_drag_slot_target_items
            .iter()
            .find(|item| item.selected)
            .map(|item| item.label.as_str()),
        Some("Trailing Slot")
    );
}
