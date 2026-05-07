use super::support::*;

#[test]
fn ui_asset_editor_session_projects_and_switches_designer_tool_modes() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/slot-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        SLOT_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    let root_pane = session.pane_presentation();
    assert_eq!(root_pane.designer_tool_mode, "Select");
    assert!(root_pane.can_designer_select);
    assert!(!root_pane.can_designer_resize_slot);
    assert!(root_pane.can_designer_preview_interact);

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session.set_designer_tool_mode(UiDesignerToolMode::PreviewInteract));
    assert!(!session.set_designer_tool_mode(UiDesignerToolMode::PreviewInteract));

    let button_pane = session.pane_presentation();
    assert_eq!(button_pane.designer_tool_mode, "Preview Interact");
    assert!(button_pane.can_designer_resize_slot);
    assert!(button_pane.can_designer_preview_interact);
}

#[test]
fn ui_asset_editor_session_resizes_selected_slot_as_single_undoable_transaction() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/slot-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        SLOT_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let initial = session.pane_presentation();
    assert_eq!(initial.inspector_slot_width_preferred, "180");
    assert_eq!(initial.inspector_slot_height_preferred, "32");

    assert!(session
        .resize_selected_slot_preferred_size(260.5, 48.0)
        .expect("resize selected slot"));
    let resized = session.pane_presentation();
    assert_eq!(resized.designer_tool_mode, "Resize Slot");
    assert_eq!(resized.inspector_slot_width_preferred, "260.5");
    assert_eq!(resized.inspector_slot_height_preferred, "48");
    assert!(session.can_undo());

    assert!(session.undo().expect("undo resize"));
    let undone = session.pane_presentation();
    assert_eq!(undone.inspector_slot_width_preferred, "180");
    assert_eq!(undone.inspector_slot_height_preferred, "32");
    assert!(session.can_redo());

    assert!(session.redo().expect("redo resize"));
    let redone = session.pane_presentation();
    assert_eq!(redone.inspector_slot_width_preferred, "260.5");
    assert_eq!(redone.inspector_slot_height_preferred, "48");
}

#[test]
fn ui_asset_editor_session_dispatches_preview_interact_binding_from_canvas_node() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/structured-binding.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STRUCTURED_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let preview_index = session.pane_presentation().preview_selected_index;
    assert!(preview_index >= 0);

    let dispatch = session
        .dispatch_preview_interact_at_preview_index(preview_index as usize, UiEventKind::Click)
        .expect("dispatch preview interaction")
        .expect("click binding dispatch");
    assert_eq!(dispatch.node_id, "button");
    assert_eq!(dispatch.control_id, "SaveButton");
    assert_eq!(dispatch.event, UiEventKind::Click);
    assert_eq!(dispatch.binding_id, "SaveButton/onClick");
    assert_eq!(dispatch.route, "MenuAction.SaveProject");
    assert_eq!(dispatch.side_effect_class, UiActionSideEffectClass::AssetIo);
    assert_eq!(
        dispatch.payload_items,
        vec!["confirm = true".to_string(), "mode = \"full\"".to_string()]
    );
    assert_eq!(session.last_preview_interact_dispatch(), Some(&dispatch));

    let pane = session.pane_presentation();
    assert_eq!(pane.designer_tool_mode, "Preview Interact");
    assert_eq!(pane.preview_interact_node_id, "button");
    assert_eq!(pane.preview_interact_event, "onClick");
    assert_eq!(pane.preview_interact_route, "MenuAction.SaveProject");
    assert_eq!(pane.preview_interact_side_effect, "AssetIo");
    assert_eq!(pane.preview_interact_payload_items, dispatch.payload_items);
}
