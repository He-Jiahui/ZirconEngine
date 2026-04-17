use crate::editing::ui_asset::UiAssetEditorSession;
use crate::{UiAssetEditorMode, UiAssetEditorRoute, UiSize};
use zircon_ui::{UiAssetKind, UiAssetLoader};

const GRID_DROP_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.grid_drop"
version = 1
display_name = "Grid Drop Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "GridBox"
control_id = "Root"
children = [{ child = "button", slot = { row = 1, column = 2, row_span = 2, column_span = 3 } }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "Button"
props = { text = "Grid" }
"##;

const OVERLAY_DROP_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.overlay_drop"
version = 1
display_name = "Overlay Drop Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Overlay"
control_id = "Root"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "Overlay" } }
"##;

const FLOW_DROP_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.flow_drop"
version = 1
display_name = "Flow Drop Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "FlowBox"
control_id = "Root"
children = [{ child = "button", slot = { break_before = true, alignment = "Center" } }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "Button"
props = { text = "Flow" }
"##;

const LOCAL_COMPONENT_SLOT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.local_component_drop"
version = 1
display_name = "Local Component Drop Layout"

[root]
node = "card"

[components.CardShell]
root = "card_root"

[components.CardShell.slots.header]
required = false
multiple = true

[components.CardShell.slots.body]
required = false
multiple = true

[nodes.card]
kind = "component"
component = "CardShell"
control_id = "CardHost"

[nodes.card_root]
kind = "native"
type = "VerticalBox"
control_id = "CardRoot"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 8.0 } }
children = [{ child = "header_slot" }, { child = "body_slot" }]

[nodes.header_slot]
kind = "slot"
slot_name = "header"

[nodes.body_slot]
kind = "slot"
slot_name = "body"
"##;

const EXTERNAL_WIDGET_SLOT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.external_widget_drop"
version = 1
display_name = "External Widget Drop Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 8.0 } }
children = [{ child = "toolbar" }]

[nodes.toolbar]
kind = "reference"
component_ref = "asset://ui/common/toolbar_shell.ui#ToolbarShell"
control_id = "ToolbarHost"
"##;

const IMPORTED_TOOLBAR_SHELL_WIDGET_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.common.toolbar_shell"
version = 1
display_name = "Toolbar Shell"

[root]
node = "toolbar_root"

[components.ToolbarShell]
root = "toolbar_root"

[components.ToolbarShell.slots.leading]
required = false
multiple = true

[components.ToolbarShell.slots.trailing]
required = false
multiple = true

[nodes.toolbar_root]
kind = "native"
type = "HorizontalBox"
control_id = "ToolbarRoot"
layout = { width = { stretch = "Stretch" }, height = { min = 48.0, preferred = 48.0, max = 48.0, stretch = "Fixed" }, container = { kind = "HorizontalBox", gap = 8.0 } }
children = [{ child = "leading_slot" }, { child = "trailing_slot" }]

[nodes.leading_slot]
kind = "slot"
slot_name = "leading"

[nodes.trailing_slot]
kind = "slot"
slot_name = "trailing"
"##;

const LOW_SEMANTIC_COMPONENT_SLOT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.low_semantic_component_drop"
version = 1
display_name = "Low Semantic Component Drop Layout"

[root]
node = "host"

[components.ThreeSlotShell]
root = "three_slot_root"

[components.ThreeSlotShell.slots.slot_a]
required = false
multiple = true

[components.ThreeSlotShell.slots.slot_b]
required = false
multiple = true

[components.ThreeSlotShell.slots.slot_c]
required = false
multiple = true

[nodes.host]
kind = "component"
component = "ThreeSlotShell"
control_id = "ThreeSlotHost"

[nodes.three_slot_root]
kind = "native"
type = "HorizontalBox"
control_id = "ThreeSlotRoot"
layout = { width = { stretch = "Stretch" }, height = { min = 48.0, preferred = 48.0, max = 48.0, stretch = "Fixed" }, container = { kind = "HorizontalBox", gap = 6.0 } }
children = [{ child = "slot_a_mount" }, { child = "slot_b_mount" }, { child = "slot_c_mount" }]

[nodes.slot_a_mount]
kind = "slot"
slot_name = "slot_a"

[nodes.slot_b_mount]
kind = "slot"
slot_name = "slot_b"

[nodes.slot_c_mount]
kind = "slot"
slot_name = "slot_c"
"##;

#[test]
fn ui_asset_editor_session_synthesizes_grid_slot_from_palette_drag_drop() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/grid-drop.ui.toml",
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

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let inserted_mount = document
        .nodes
        .get("root")
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
        "asset://ui/tests/grid-drop.ui.toml",
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
        "asset://ui/tests/overlay-drop.ui.toml",
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

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let inserted_mount = document
        .nodes
        .get("root")
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
        "asset://ui/tests/overlay-drop.ui.toml",
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
        "asset://ui/tests/flow-drop.ui.toml",
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

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let inserted_mount = document
        .nodes
        .get("root")
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
        "asset://ui/tests/local-component-drop.ui.toml",
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
        "asset://ui/tests/external-widget-drop.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut external_session = UiAssetEditorSession::from_source(
        external_route,
        EXTERNAL_WIDGET_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("external widget drop session");
    let imported_widget = UiAssetLoader::load_toml_str(IMPORTED_TOOLBAR_SHELL_WIDGET_ASSET_TOML)
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

#[test]
fn ui_asset_editor_session_routes_palette_drop_into_local_component_mounts() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/local-component-drop.ui.toml",
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

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let inserted_mount = document
        .nodes
        .get("card")
        .and_then(|node| node.children.first())
        .expect("inserted local component child mount");
    assert_eq!(inserted_mount.mount.as_deref(), Some("body"));
}

#[test]
fn ui_asset_editor_session_routes_palette_drop_into_external_widget_named_slots() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/external-widget-drop.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        EXTERNAL_WIDGET_SLOT_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("external widget drop session");
    let imported_widget = UiAssetLoader::load_toml_str(IMPORTED_TOOLBAR_SHELL_WIDGET_ASSET_TOML)
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

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let inserted_mount = document
        .nodes
        .get("toolbar")
        .and_then(|node| node.children.first())
        .expect("inserted external widget child mount");
    assert_eq!(inserted_mount.mount.as_deref(), Some("trailing"));
}

#[test]
fn ui_asset_editor_session_uses_explicit_slot_overlay_regions_for_low_semantic_component_mounts() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/low-semantic-component-drop.ui.toml",
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

    assert!(session
        .drop_selected_palette_item_at_palette_drag_target()
        .expect("drop palette item into low semantic component"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let inserted_mount = document
        .nodes
        .get("host")
        .and_then(|node| node.children.first())
        .expect("inserted low semantic component child mount");
    assert_eq!(inserted_mount.mount.as_deref(), Some("slot_b"));
}

#[test]
fn ui_asset_editor_session_exposes_palette_drag_target_cycle_candidates_for_low_semantic_slots() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/low-semantic-component-drop.ui.toml",
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
    assert_eq!(cycled_previous.palette_drag_target_label, "Insert Slot B Slot");
}

#[test]
fn ui_asset_editor_session_drop_uses_cycled_palette_drag_target_candidate() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/low-semantic-component-drop.ui.toml",
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

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let inserted_mount = document
        .nodes
        .get("host")
        .and_then(|node| node.children.first())
        .expect("inserted low semantic component child mount");
    assert_eq!(inserted_mount.mount.as_deref(), Some("slot_c"));
}

#[test]
fn ui_asset_editor_session_ambiguous_palette_drop_arms_sticky_target_chooser_after_release() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/low-semantic-component-drop.ui.toml",
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
        "asset://ui/tests/low-semantic-component-drop.ui.toml",
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

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let inserted_mount = document
        .nodes
        .get("host")
        .and_then(|node| node.children.first())
        .expect("inserted low semantic component child mount");
    assert_eq!(inserted_mount.mount.as_deref(), Some("slot_c"));
}

#[test]
fn ui_asset_editor_session_sticky_palette_target_chooser_cancels_without_mutating_source() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/low-semantic-component-drop.ui.toml",
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

fn select_palette_entry(session: &mut UiAssetEditorSession, label: &str) {
    let palette_index = session
        .pane_presentation()
        .palette_items
        .iter()
        .position(|item| item == label)
        .unwrap_or_else(|| panic!("palette item {label}"));
    session
        .select_palette_index(palette_index)
        .expect("select palette item");
}

fn preview_frame(
    session: &UiAssetEditorSession,
    node_id: &str,
) -> crate::editing::ui_asset::UiAssetEditorPreviewCanvasNode {
    session
        .pane_presentation()
        .preview_canvas_items
        .into_iter()
        .find(|item| item.node_id == node_id)
        .unwrap_or_else(|| panic!("preview frame {node_id}"))
}

fn numeric_slot_value(
    slot: &std::collections::BTreeMap<String, toml::Value>,
    path: &[&str],
) -> Option<f64> {
    let mut current = slot.get(path.first().copied()?)?;
    for segment in &path[1..] {
        current = current.as_table()?.get(*segment)?;
    }
    current
        .as_float()
        .or_else(|| current.as_integer().map(|value| value as f64))
}
