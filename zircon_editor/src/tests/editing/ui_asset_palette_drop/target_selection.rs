use super::*;

#[test]
fn ui_asset_editor_session_exposes_palette_drag_target_cycle_candidates_for_low_semantic_slots() {
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

    let initial = session.pane_presentation();
    assert_eq!(
        initial.palette_drag_candidate_items,
        vec![
            "Slot A Slot • slot_a".to_string(),
            "Slot B Slot • slot_b".to_string(),
            "Slot C Slot • slot_c".to_string(),
        ]
    );
    assert_eq!(initial.palette_drag_candidate_selected_index, 1);
    assert_eq!(initial.palette_drag_target_label, "Insert Slot B Slot");

    assert!(session
        .cycle_palette_drag_target_candidate_next()
        .expect("cycle palette drag target next"));
    let cycled_next = session.pane_presentation();
    assert_eq!(cycled_next.palette_drag_candidate_selected_index, 2);
    assert_eq!(cycled_next.palette_drag_target_label, "Insert Slot C Slot");

    assert!(session
        .cycle_palette_drag_target_candidate_previous()
        .expect("cycle palette drag target previous"));
    let cycled_previous = session.pane_presentation();
    assert_eq!(cycled_previous.palette_drag_candidate_selected_index, 1);
    assert_eq!(
        cycled_previous.palette_drag_target_label,
        "Insert Slot B Slot"
    );
}

#[test]
fn ui_asset_editor_session_drop_uses_cycled_palette_drag_target_candidate() {
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
    assert!(session
        .cycle_palette_drag_target_candidate_next()
        .expect("cycle palette drag target next"));

    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("drop palette item into cycled low semantic component target"));

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let inserted_mount = document
        .node("host")
        .and_then(|node| node.children.first())
        .expect("inserted low semantic component child mount");
    assert_eq!(inserted_mount.mount.as_deref(), Some("slot_c"));
}

#[test]
fn ui_asset_editor_session_ambiguous_palette_drop_arms_sticky_target_chooser_after_release() {
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

    let original_source = session.source_buffer().text().to_string();
    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("arm sticky chooser instead of committing ambiguous drop"));

    assert_eq!(session.source_buffer().text(), original_source);
    let pane = session.pane_presentation();
    assert!(pane.palette_target_chooser_active);
    assert_eq!(
        pane.palette_drag_candidate_items,
        vec![
            "Slot A Slot • slot_a".to_string(),
            "Slot B Slot • slot_b".to_string(),
            "Slot C Slot • slot_c".to_string(),
        ]
    );
    assert_eq!(pane.palette_drag_candidate_selected_index, 1);
    assert_eq!(pane.palette_drag_target_label, "Insert Slot B Slot");
}

#[test]
fn ui_asset_editor_session_sticky_palette_target_chooser_selects_and_confirms_candidate() {
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
    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("arm sticky chooser"));

    assert!(session
        .select_palette_target_candidate(2)
        .expect("select sticky chooser slot c candidate"));
    let selected = session.pane_presentation();
    assert!(selected.palette_target_chooser_active);
    assert_eq!(selected.palette_drag_candidate_selected_index, 2);
    assert_eq!(selected.palette_drag_target_label, "Insert Slot C Slot");

    assert!(session
        .confirm_palette_target_choice()
        .expect("confirm sticky chooser"));
    let confirmed = session.pane_presentation();
    assert!(!confirmed.palette_target_chooser_active);

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let inserted_mount = document
        .node("host")
        .and_then(|node| node.children.first())
        .expect("inserted low semantic component child mount");
    assert_eq!(inserted_mount.mount.as_deref(), Some("slot_c"));
}

#[test]
fn ui_asset_editor_session_sticky_palette_target_chooser_cancels_without_mutating_source() {
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

    let original_source = session.source_buffer().text().to_string();
    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("arm sticky chooser"));
    assert!(session
        .cancel_palette_target_choice()
        .expect("cancel sticky chooser"));

    assert_eq!(session.source_buffer().text(), original_source);
    assert!(!session.pane_presentation().palette_target_chooser_active);
}

#[test]
fn ui_asset_editor_session_sticky_palette_target_chooser_survives_hover_reconciliation() {
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
    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("arm sticky chooser"));
    assert!(session
        .select_palette_target_candidate(2)
        .expect("select sticky chooser slot c candidate"));

    assert!(!session
        .update_palette_drag_target(
            host_frame.x + host_frame.width * 0.18,
            host_frame.y + host_frame.height * 0.5,
        )
        .expect("sticky chooser should ignore hover-driven target changes"));

    let pane = session.pane_presentation();
    assert!(pane.palette_target_chooser_active);
    assert_eq!(pane.palette_drag_candidate_selected_index, 2);
    assert_eq!(pane.palette_drag_target_label, "Insert Slot C Slot");
}

#[test]
fn ui_asset_editor_session_sticky_palette_target_chooser_survives_hover_loss_until_cancelled() {
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
    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("arm sticky chooser"));
    assert!(session
        .select_palette_target_candidate(2)
        .expect("select sticky chooser slot c candidate"));

    assert!(!session
        .update_palette_drag_target(-64.0, -64.0)
        .expect("ignore hover loss while sticky chooser is armed"));

    let pane = session.pane_presentation();
    assert!(pane.palette_target_chooser_active);
    assert_eq!(pane.palette_drag_candidate_selected_index, 2);
    assert_eq!(pane.palette_drag_target_label, "Insert Slot C Slot");

    assert!(session
        .cancel_palette_target_choice()
        .expect("cancel sticky chooser after hover loss"));
    assert!(!session.pane_presentation().palette_target_chooser_active);
}
