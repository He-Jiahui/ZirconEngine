use super::support::*;

#[test]
fn ui_asset_editor_session_projects_structured_widget_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
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
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.inspector_parent_node_id, "root");
    assert_eq!(pane.inspector_mount, "");
    assert_eq!(pane.inspector_widget_kind, "Native");
    assert_eq!(pane.inspector_widget_label, "Button");
    assert_eq!(pane.inspector_control_id, "SaveButton");
    assert_eq!(pane.inspector_text_prop, "Save");
    assert!(pane.inspector_can_edit_control_id);
    assert!(pane.inspector_can_edit_text_prop);
}

#[test]
fn ui_asset_editor_session_updates_selected_widget_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
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
    assert!(session
        .set_selected_widget_control_id("ConfirmButton")
        .expect("set selected control id"));
    assert!(session
        .set_selected_widget_text_property("Confirm")
        .expect("set selected text property"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_control_id, "ConfirmButton");
    assert_eq!(updated.inspector_text_prop, "Confirm");

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let button = document.node("button").expect("button node");
    assert_eq!(button.control_id.as_deref(), Some("ConfirmButton"));
    assert_eq!(
        button.props.get("text").and_then(toml::Value::as_str),
        Some("Confirm")
    );
    assert!(preview_has_control_id(
        session.preview_host().surface(),
        "ConfirmButton"
    ));
}

#[test]
fn ui_asset_editor_session_projects_structured_slot_inspector_fields() {
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

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_mount, "actions");
    assert_eq!(pane.inspector_slot_padding, "8");
    assert_eq!(pane.inspector_slot_width_preferred, "180");
    assert_eq!(pane.inspector_slot_height_preferred, "32");
}

#[test]
fn ui_asset_editor_session_updates_selected_slot_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
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
    assert!(session
        .set_selected_slot_mount("footer")
        .expect("set selected mount"));
    assert!(session
        .set_selected_slot_padding("12")
        .expect("set selected slot padding"));
    assert!(session
        .set_selected_slot_width_preferred("240")
        .expect("set selected slot width preferred"));
    assert!(session
        .set_selected_slot_height_preferred("44")
        .expect("set selected slot height preferred"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_mount, "footer");
    assert_eq!(updated.inspector_slot_padding, "12");
    assert_eq!(updated.inspector_slot_width_preferred, "240");
    assert_eq!(updated.inspector_slot_height_preferred, "44");

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let child_mount = document
        .node("root")
        .expect("root node")
        .children
        .iter()
        .find(|child_mount| child_mount.node.node_id == "button")
        .expect("button child mount");
    assert_eq!(child_mount.mount.as_deref(), Some("footer"));
    assert_eq!(
        slot_value(&child_mount.slot, &["padding"]).and_then(toml::Value::as_integer),
        Some(12)
    );
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "width", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(240)
    );
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "height", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(44)
    );
}

#[test]
fn ui_asset_editor_session_projects_structured_layout_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/layout-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LAYOUT_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_layout_width_preferred, "220");
    assert_eq!(pane.inspector_layout_height_preferred, "48");
}

#[test]
fn ui_asset_editor_session_updates_selected_layout_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
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
    assert!(session
        .set_selected_layout_width_preferred("220")
        .expect("set selected layout width preferred"));
    assert!(session
        .set_selected_layout_height_preferred("48")
        .expect("set selected layout height preferred"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_layout_width_preferred, "220");
    assert_eq!(updated.inspector_layout_height_preferred, "48");

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let button = document.node("button").expect("button node");
    assert_eq!(
        layout_value(button.layout.as_ref(), &["width", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(220)
    );
    assert_eq!(
        layout_value(button.layout.as_ref(), &["height", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(48)
    );
}

#[test]
fn ui_asset_editor_session_projects_parent_specific_slot_and_layout_semantics() {
    let overlay_route = UiAssetEditorRoute::new(
        "asset://ui/tests/overlay-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut overlay_session = UiAssetEditorSession::from_source(
        overlay_route,
        OVERLAY_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("overlay session");
    overlay_session
        .select_hierarchy_index(1)
        .expect("select overlay child");
    let overlay = overlay_session.pane_presentation();
    assert_eq!(overlay.inspector_slot_kind, "Overlay");
    assert_eq!(overlay.inspector_slot_semantic_title, "Overlay Slot");
    assert_eq!(overlay.inspector_slot_overlay_anchor_x, "1");
    assert_eq!(overlay.inspector_slot_overlay_anchor_y, "0");
    assert_eq!(overlay.inspector_slot_overlay_pivot_x, "1");
    assert_eq!(overlay.inspector_slot_overlay_pivot_y, "0");
    assert_eq!(overlay.inspector_slot_overlay_position_x, "-16");
    assert_eq!(overlay.inspector_slot_overlay_position_y, "12");
    assert_eq!(overlay.inspector_slot_overlay_z_index, "4");
    assert_eq!(
        overlay.inspector_slot_semantic_items,
        vec![
            "layout.anchor.x = 1".to_string(),
            "layout.anchor.y = 0".to_string(),
            "layout.pivot.x = 1".to_string(),
            "layout.pivot.y = 0".to_string(),
            "layout.position.x = -16".to_string(),
            "layout.position.y = 12".to_string(),
            "layout.z_index = 4".to_string()
        ]
    );

    let grid_route = UiAssetEditorRoute::new(
        "asset://ui/tests/grid-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut grid_session = UiAssetEditorSession::from_source(
        grid_route,
        GRID_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("grid session");
    grid_session
        .select_hierarchy_index(1)
        .expect("select grid child");
    let grid = grid_session.pane_presentation();
    assert_eq!(grid.inspector_slot_kind, "Grid");
    assert_eq!(grid.inspector_slot_semantic_title, "Grid Slot");
    assert_eq!(grid.inspector_slot_grid_row, "1");
    assert_eq!(grid.inspector_slot_grid_column, "2");
    assert_eq!(grid.inspector_slot_grid_row_span, "2");
    assert_eq!(grid.inspector_slot_grid_column_span, "3");
    assert_eq!(
        grid.inspector_slot_semantic_items,
        vec![
            "row = 1".to_string(),
            "column = 2".to_string(),
            "row_span = 2".to_string(),
            "column_span = 3".to_string()
        ]
    );

    let flow_route = UiAssetEditorRoute::new(
        "asset://ui/tests/flow-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut flow_session = UiAssetEditorSession::from_source(
        flow_route,
        FLOW_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("flow session");
    flow_session
        .select_hierarchy_index(1)
        .expect("select flow child");
    let flow = flow_session.pane_presentation();
    assert_eq!(flow.inspector_slot_kind, "Flow");
    assert_eq!(flow.inspector_slot_semantic_title, "Flow Slot");
    assert_eq!(flow.inspector_slot_flow_break_before, "true");
    assert_eq!(flow.inspector_slot_flow_alignment, "\"Center\"");
    assert_eq!(
        flow.inspector_slot_semantic_items,
        vec![
            "break_before = true".to_string(),
            "alignment = \"Center\"".to_string()
        ]
    );

    let scroll_route = UiAssetEditorRoute::new(
        "asset://ui/tests/scrollable-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let scroll_session = UiAssetEditorSession::from_source(
        scroll_route,
        SCROLLABLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("scroll session");
    let scroll = scroll_session.pane_presentation();
    assert_eq!(scroll.inspector_layout_kind, "ScrollableBox");
    assert_eq!(scroll.inspector_layout_semantic_title, "Scrollable Layout");
    assert_eq!(scroll.inspector_layout_scroll_axis, "\"Vertical\"");
    assert_eq!(scroll.inspector_layout_scroll_gap, "6");
    assert_eq!(scroll.inspector_layout_scrollbar_visibility, "\"Always\"");
    assert_eq!(scroll.inspector_layout_virtualization_item_extent, "28");
    assert_eq!(scroll.inspector_layout_virtualization_overscan, "2");
    assert_eq!(scroll.inspector_layout_clip, "true");
    assert_eq!(
        scroll.inspector_layout_semantic_items,
        vec![
            "container.axis = \"Vertical\"".to_string(),
            "container.gap = 6".to_string(),
            "container.scrollbar_visibility = \"Always\"".to_string(),
            "container.virtualization.item_extent = 28".to_string(),
            "container.virtualization.overscan = 2".to_string(),
            "clip = true".to_string()
        ]
    );

    let horizontal_route = UiAssetEditorRoute::new(
        "asset://ui/tests/horizontal-box-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let horizontal_session = UiAssetEditorSession::from_source(
        horizontal_route,
        HORIZONTAL_BOX_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("horizontal session");
    let horizontal = horizontal_session.pane_presentation();
    assert_eq!(horizontal.inspector_layout_kind, "HorizontalBox");
    assert_eq!(horizontal.inspector_layout_semantic_title, "Linear Layout");
    assert_eq!(horizontal.inspector_layout_box_gap, "10");
    assert_eq!(
        horizontal.inspector_layout_semantic_items,
        vec!["container.gap = 10".to_string()]
    );

    let vertical_route = UiAssetEditorRoute::new(
        "asset://ui/tests/vertical-box-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let vertical_session = UiAssetEditorSession::from_source(
        vertical_route,
        VERTICAL_BOX_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("vertical session");
    let vertical = vertical_session.pane_presentation();
    assert_eq!(vertical.inspector_layout_kind, "VerticalBox");
    assert_eq!(vertical.inspector_layout_semantic_title, "Linear Layout");
    assert_eq!(vertical.inspector_layout_box_gap, "12");
    assert_eq!(
        vertical.inspector_layout_semantic_items,
        vec!["container.gap = 12".to_string()]
    );
}

#[test]
fn ui_asset_editor_session_updates_parent_specific_slot_and_layout_semantics() {
    let overlay_route = UiAssetEditorRoute::new(
        "asset://ui/tests/overlay-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut overlay_session = UiAssetEditorSession::from_source(
        overlay_route,
        OVERLAY_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("overlay session");
    overlay_session
        .select_hierarchy_index(1)
        .expect("select overlay child");
    assert!(overlay_session
        .set_selected_slot_semantic_field("layout.pivot.x", "0.5")
        .expect("update overlay pivot x"));

    let overlay_document =
        crate::tests::support::load_test_ui_asset(overlay_session.source_buffer().text())
            .expect("document");
    let overlay_mount = overlay_document
        .node("root")
        .expect("root node")
        .children
        .iter()
        .find(|child_mount| child_mount.node.node_id == "badge")
        .expect("overlay child mount");
    assert_eq!(
        slot_value(&overlay_mount.slot, &["layout", "pivot", "x"]).and_then(toml::Value::as_float),
        Some(0.5)
    );

    let grid_route = UiAssetEditorRoute::new(
        "asset://ui/tests/grid-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut grid_session = UiAssetEditorSession::from_source(
        grid_route,
        GRID_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("grid session");
    grid_session
        .select_hierarchy_index(1)
        .expect("select grid child");
    assert!(grid_session
        .set_selected_slot_semantic_field("column_span", "4")
        .expect("update grid column span"));
    let grid_document =
        crate::tests::support::load_test_ui_asset(grid_session.source_buffer().text())
            .expect("document");
    let grid_mount = grid_document
        .node("root")
        .expect("root node")
        .children
        .iter()
        .find(|child_mount| child_mount.node.node_id == "button")
        .expect("grid child mount");
    assert_eq!(
        slot_value(&grid_mount.slot, &["column_span"]).and_then(toml::Value::as_integer),
        Some(4)
    );

    let flow_route = UiAssetEditorRoute::new(
        "asset://ui/tests/flow-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut flow_session = UiAssetEditorSession::from_source(
        flow_route,
        FLOW_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("flow session");
    flow_session
        .select_hierarchy_index(1)
        .expect("select flow child");
    assert!(flow_session
        .set_selected_slot_semantic_field("break_before", "false")
        .expect("update flow break before"));
    let flow_document =
        crate::tests::support::load_test_ui_asset(flow_session.source_buffer().text())
            .expect("document");
    let flow_mount = flow_document
        .node("root")
        .expect("root node")
        .children
        .iter()
        .find(|child_mount| child_mount.node.node_id == "button")
        .expect("flow child mount");
    assert_eq!(
        slot_value(&flow_mount.slot, &["break_before"]).and_then(toml::Value::as_bool),
        Some(false)
    );

    let scroll_route = UiAssetEditorRoute::new(
        "asset://ui/tests/scrollable-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut scroll_session = UiAssetEditorSession::from_source(
        scroll_route,
        SCROLLABLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("scroll session");
    assert!(scroll_session
        .set_selected_layout_semantic_field("container.scrollbar_visibility", "\"Auto\"")
        .expect("update scroll scrollbar visibility"));
    let scroll_document =
        crate::tests::support::load_test_ui_asset(scroll_session.source_buffer().text())
            .expect("document");
    let scroll_root = scroll_document.node("root").expect("scroll root");
    assert_eq!(
        layout_value(
            scroll_root.layout.as_ref(),
            &["container", "scrollbar_visibility"]
        )
        .and_then(toml::Value::as_str),
        Some("Auto")
    );

    let horizontal_route = UiAssetEditorRoute::new(
        "asset://ui/tests/horizontal-box-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut horizontal_session = UiAssetEditorSession::from_source(
        horizontal_route,
        HORIZONTAL_BOX_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("horizontal session");
    assert!(horizontal_session
        .set_selected_layout_semantic_field("container.gap", "18")
        .expect("update horizontal box gap"));
    let horizontal_document =
        crate::tests::support::load_test_ui_asset(horizontal_session.source_buffer().text())
            .expect("document");
    let horizontal_root = horizontal_document.node("root").expect("horizontal root");
    assert_eq!(
        layout_value(horizontal_root.layout.as_ref(), &["container", "gap"])
            .and_then(toml::Value::as_integer),
        Some(18)
    );
    assert_eq!(
        horizontal_session
            .pane_presentation()
            .inspector_layout_box_gap,
        "18"
    );

    let vertical_route = UiAssetEditorRoute::new(
        "asset://ui/tests/vertical-box-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut vertical_session = UiAssetEditorSession::from_source(
        vertical_route,
        VERTICAL_BOX_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("vertical session");
    assert!(vertical_session
        .set_selected_layout_semantic_field("container.gap", "20")
        .expect("update vertical box gap"));
    let vertical_document =
        crate::tests::support::load_test_ui_asset(vertical_session.source_buffer().text())
            .expect("document");
    let vertical_root = vertical_document.node("root").expect("vertical root");
    assert_eq!(
        layout_value(vertical_root.layout.as_ref(), &["container", "gap"])
            .and_then(toml::Value::as_integer),
        Some(20)
    );
    assert_eq!(
        vertical_session
            .pane_presentation()
            .inspector_layout_box_gap,
        "20"
    );
}

#[test]
fn ui_asset_editor_session_projects_linear_slot_typed_fields() {
    let horizontal_route = UiAssetEditorRoute::new(
        "asset://ui/tests/horizontal-linear-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut horizontal_session = UiAssetEditorSession::from_source(
        horizontal_route,
        HORIZONTAL_LINEAR_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("horizontal session");
    horizontal_session
        .select_hierarchy_index(1)
        .expect("select horizontal linear child");

    let horizontal = horizontal_session.pane_presentation();
    assert_eq!(horizontal.inspector_slot_kind, "HorizontalBox");
    assert_eq!(horizontal.inspector_slot_semantic_title, "Linear Slot");
    assert_eq!(horizontal.inspector_slot_linear_main_weight, "3");
    assert_eq!(horizontal.inspector_slot_linear_main_stretch, "Stretch");
    assert_eq!(horizontal.inspector_slot_linear_cross_weight, "2");
    assert_eq!(horizontal.inspector_slot_linear_cross_stretch, "Fixed");

    let vertical_route = UiAssetEditorRoute::new(
        "asset://ui/tests/vertical-linear-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut vertical_session = UiAssetEditorSession::from_source(
        vertical_route,
        VERTICAL_LINEAR_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("vertical session");
    vertical_session
        .select_hierarchy_index(1)
        .expect("select vertical linear child");

    let vertical = vertical_session.pane_presentation();
    assert_eq!(vertical.inspector_slot_kind, "VerticalBox");
    assert_eq!(vertical.inspector_slot_semantic_title, "Linear Slot");
    assert_eq!(vertical.inspector_slot_linear_main_weight, "5");
    assert_eq!(vertical.inspector_slot_linear_main_stretch, "Fixed");
    assert_eq!(vertical.inspector_slot_linear_cross_weight, "4");
    assert_eq!(vertical.inspector_slot_linear_cross_stretch, "Stretch");
}

#[test]
fn ui_asset_editor_session_updates_linear_slot_typed_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/horizontal-linear-slot.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        HORIZONTAL_LINEAR_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    session
        .select_hierarchy_index(1)
        .expect("select horizontal linear child");

    assert!(session
        .set_selected_slot_semantic_field("layout.width.weight", "6")
        .expect("update main-axis weight"));
    assert!(session
        .set_selected_slot_semantic_field("layout.height.stretch", "\"Stretch\"")
        .expect("update cross-axis stretch"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_slot_linear_main_weight, "6");
    assert_eq!(updated.inspector_slot_linear_cross_stretch, "Stretch");

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let child_mount = document
        .node("root")
        .expect("root node")
        .children
        .iter()
        .find(|child_mount| child_mount.node.node_id == "fill")
        .expect("fill child mount");
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "width", "weight"])
            .and_then(toml::Value::as_integer),
        Some(6)
    );
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "height", "stretch"])
            .and_then(toml::Value::as_str),
        Some("Stretch")
    );
}

#[test]
fn ui_asset_editor_session_projects_structured_binding_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/binding-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        BINDING_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(
        pane.inspector_binding_items,
        vec!["onClick | SaveButton/onClick -> MenuAction.SaveProject".to_string()]
    );
    assert_eq!(pane.inspector_binding_selected_index, 0);
    assert_eq!(pane.inspector_binding_id, "SaveButton/onClick");
    assert_eq!(pane.inspector_binding_event, "onClick");
    assert_eq!(pane.inspector_binding_route, "MenuAction.SaveProject");
    assert_eq!(
        pane.inspector_binding_route_target,
        "MenuAction.SaveProject"
    );
    assert_eq!(pane.inspector_binding_action_target, "");
}

#[test]
fn ui_asset_editor_session_updates_selected_binding_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
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
    assert!(session.add_binding().expect("add binding"));
    assert!(session
        .set_selected_binding_id("SaveButton/onHover")
        .expect("set selected binding id"));
    assert!(session
        .set_selected_binding_event("onHover")
        .expect("set selected binding event"));
    assert!(session
        .set_selected_binding_route("MenuAction.HighlightSave")
        .expect("set selected binding route"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_binding_selected_index, 0);
    assert_eq!(updated.inspector_binding_id, "SaveButton/onHover");
    assert_eq!(updated.inspector_binding_event, "onHover");
    assert_eq!(updated.inspector_binding_route, "MenuAction.HighlightSave");
    assert_eq!(
        updated.inspector_binding_route_target,
        "MenuAction.HighlightSave"
    );
    assert_eq!(updated.inspector_binding_action_target, "");

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let button = document.node("button").expect("button node");
    assert_eq!(button.bindings.len(), 1);
    assert_eq!(button.bindings[0].id, "SaveButton/onHover");
    assert_eq!(button.bindings[0].event.to_string(), "onHover");
    assert_eq!(
        button.bindings[0].route.as_deref(),
        Some("MenuAction.HighlightSave")
    );
}

#[test]
fn ui_asset_editor_session_projects_structured_binding_action_and_payload_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/structured-binding-authoring.ui.toml",
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

    let pane = session.pane_presentation();
    assert_eq!(
        pane.inspector_binding_items,
        vec!["onClick | SaveButton/onClick -> MenuAction.SaveProject (+2 payload)".to_string()]
    );
    assert_eq!(pane.inspector_binding_event_selected_index, 0);
    assert_eq!(pane.inspector_binding_action_kind_selected_index, 1);
    assert_eq!(
        pane.inspector_binding_action_kind_items,
        vec![
            "None".to_string(),
            "Route".to_string(),
            "Action".to_string()
        ]
    );
    assert_eq!(pane.inspector_binding_route, "MenuAction.SaveProject");
    assert_eq!(
        pane.inspector_binding_route_target,
        "MenuAction.SaveProject"
    );
    assert_eq!(pane.inspector_binding_action_target, "");
    assert_eq!(
        pane.inspector_binding_payload_items,
        vec!["confirm = true".to_string(), "mode = \"full\"".to_string()]
    );
    assert_eq!(pane.inspector_binding_payload_selected_index, 0);
    assert_eq!(pane.inspector_binding_payload_key, "confirm");
    assert_eq!(pane.inspector_binding_payload_value, "true");
}

#[test]
fn ui_asset_editor_session_updates_structured_binding_action_and_payload_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/structured-binding-authoring.ui.toml",
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
    assert!(session
        .select_binding_event_option(1)
        .expect("select double click event"));
    assert!(session
        .select_binding_action_kind(2)
        .expect("select action kind"));
    assert!(session
        .set_selected_binding_action_target("EditorActions.SaveProject")
        .expect("set action target"));
    assert!(session
        .select_binding_payload(1)
        .expect("select mode payload"));
    assert!(session
        .upsert_selected_binding_payload("mode", "\"compact\"")
        .expect("update payload"));
    assert!(session
        .upsert_selected_binding_payload("channel", "\"toolbar\"")
        .expect("add payload"));
    assert!(session
        .delete_selected_binding_payload()
        .expect("delete selected payload"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_binding_event, "onDoubleClick");
    assert_eq!(updated.inspector_binding_event_selected_index, 1);
    assert_eq!(updated.inspector_binding_action_kind_selected_index, 2);
    assert_eq!(updated.inspector_binding_route, "EditorActions.SaveProject");
    assert_eq!(updated.inspector_binding_route_target, "");
    assert_eq!(
        updated.inspector_binding_action_target,
        "EditorActions.SaveProject"
    );
    assert_eq!(
        updated.inspector_binding_payload_items,
        vec![
            "confirm = true".to_string(),
            "mode = \"compact\"".to_string()
        ]
    );

    let document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("document");
    let button = document.node("button").expect("button node");
    assert_eq!(button.bindings[0].event.to_string(), "onDoubleClick");
    assert!(button.bindings[0].route.is_none());
    let action = button.bindings[0].action.as_ref().expect("binding action");
    assert_eq!(action.action.as_deref(), Some("EditorActions.SaveProject"));
    assert_eq!(
        action.payload.get("mode").and_then(toml::Value::as_str),
        Some("compact")
    );
    assert!(action.payload.get("channel").is_none());
}

#[test]
fn ui_asset_editor_session_projects_binding_payload_schema_suggestions_and_applies_them() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/structured-binding-authoring.ui.toml",
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

    let initial = session.pane_presentation();
    assert_eq!(
        initial.inspector_binding_payload_suggestion_items,
        vec![
            "confirm = true".to_string(),
            "channel = \"toolbar\"".to_string(),
            "source = \"ui.click\"".to_string(),
        ]
    );

    assert!(session
        .apply_selected_binding_payload_suggestion(2)
        .expect("apply binding payload suggestion"));

    let updated = session.pane_presentation();
    assert_eq!(
        updated.inspector_binding_payload_items,
        vec![
            "confirm = true".to_string(),
            "mode = \"full\"".to_string(),
            "source = \"ui.click\"".to_string(),
        ]
    );

    assert!(session
        .select_binding_event_option(10)
        .expect("select scroll event"));
    let scroll = session.pane_presentation();
    assert_eq!(
        scroll.inspector_binding_payload_suggestion_items,
        vec![
            "axis = \"vertical\"".to_string(),
            "delta = 1".to_string(),
            "source = \"ui.scroll\"".to_string(),
        ]
    );
}

