//! Exercise Canvas source edits through the real v2 preview and undo consumers.

use crate::ui::asset_editor::{UiAssetEditorSession, UiAssetEditorSessionError};
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::layout::UiFrame;

use super::support::open_design_session;

const CANVAS_SOURCE: &str = r#"
[asset]
kind = "view"
id = "ui.test.canvas_anchors"
version = 2

[root]
node = "canvas"

[nodes.canvas]
component = "Panel"
control_id = "CanvasRoot"
layout = { container = { kind = "Canvas" }, padding = { left = 17, top = 11, right = 23, bottom = 19 } }
children = [
  { node = "button", slot = { layout = { anchor = { x = 0.25, y = 0.25 }, anchor_max = { x = 0.25, y = 0.25 }, pivot = { x = 0.2, y = 0.6 }, position = { x = 13, y = -7 }, offset = { left = 25, top = 30, right = 80, bottom = 40 }, auto_size = false, z_order = 7 } } },
  { node = "sibling" },
]

[nodes.button]
component = "Button"
control_id = "AnchorTarget"
props = { text = "锚点事务 / Anchor target" }
events = [{ id = "AnchorTarget/Click", event = "Click", route = "fixture.select" }]

[nodes.sibling]
component = "Label"
control_id = "UntouchedSibling"
props = { text = "Retain sibling identity" }
"#;

fn selected_session(source: &str) -> UiAssetEditorSession {
    let mut session = open_design_session("asset://ui/canvas_anchors.zui", source);
    assert!(
        session.diagnostics().is_empty(),
        "{:?}",
        session.diagnostics()
    );
    session
        .select_hierarchy_index(1)
        .expect("select Canvas child");
    assert_eq!(
        session
            .reflection_model()
            .selection
            .primary_node_id
            .as_deref(),
        Some("button")
    );
    session
}

fn node_frame(session: &UiAssetEditorSession, source_node_id: &str) -> UiFrame {
    session
        .preview_host()
        .surface()
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.source_node_id.as_deref())
                == Some(source_node_id)
        })
        .expect("source-mapped retained node")
        .layout_cache
        .frame
}

fn parent_content(session: &UiAssetEditorSession) -> UiFrame {
    let frame = node_frame(session, "canvas");
    UiFrame::new(
        frame.x + 17.0,
        frame.y + 11.0,
        frame.width - 40.0,
        frame.height - 30.0,
    )
}

fn close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.001,
        "actual {actual}, expected {expected}"
    );
}

fn apply_preset(
    session: &mut UiAssetEditorSession,
    index: usize,
    shift: bool,
    control: bool,
) -> bool {
    session
        .apply_selected_canvas_anchor_preset(index, shift, control)
        .expect("apply Canvas preset")
}

fn same_frame(actual: UiFrame, expected: UiFrame) {
    close(actual.x, expected.x);
    close(actual.y, expected.y);
    close(actual.width, expected.width);
    close(actual.height, expected.height);
}

#[test]
fn all_presets_preserve_the_current_rectangle_from_point_and_stretch_slots() {
    let stretch_source = CANVAS_SOURCE.replace(
        "anchor_max = { x = 0.25, y = 0.25 }",
        "anchor_max = { x = 0.8, y = 0.9 }",
    );
    for source in [CANVAS_SOURCE, stretch_source.as_str()] {
        for index in 0..16 {
            let mut session = selected_session(source);
            let before = node_frame(&session, "button");
            assert!(apply_preset(&mut session, index, false, false));
            assert!(session.diagnostics().is_empty());
            same_frame(node_frame(&session, "button"), before);
        }
    }
}

#[test]
fn shift_changes_point_alignment_after_rebasing_with_the_old_pivot() {
    let mut session = selected_session(CANVAS_SOURCE);
    let before = node_frame(&session, "button");
    assert!(apply_preset(&mut session, 2, true, false));
    let after = node_frame(&session, "button");
    // Right/top point: old pivot (.2,.6), new pivot (1,0).
    close(after.x, before.x + before.width * (0.2 - 1.0));
    close(after.y, before.y + before.height * 0.6);
    close(after.width, before.width);
    close(after.height, before.height);
}

#[test]
fn control_resets_both_position_terms_and_only_stretched_trailing_offsets() {
    let mut session = selected_session(CANVAS_SOURCE);
    let parent = parent_content(&session);
    let before = node_frame(&session, "button");
    // Center/fill: X remains point-sized, Y stretches to both parent edges.
    assert!(apply_preset(&mut session, 13, true, true));
    let after = node_frame(&session, "button");
    same_frame(
        after,
        UiFrame::new(
            parent.x + parent.width * 0.5 - before.width * 0.5,
            parent.y,
            before.width,
            parent.height,
        ),
    );
    let source: toml::Value = toml::from_str(session.source_buffer().text()).unwrap();
    let layout = &source["nodes"]["canvas"]["children"][0]["slot"]["layout"];
    assert_eq!(layout["position"]["x"].as_integer(), Some(0));
    assert_eq!(layout["position"]["y"].as_integer(), Some(0));
    assert_eq!(layout["offset"]["left"].as_integer(), Some(0));
    assert_eq!(layout["offset"]["top"].as_integer(), Some(0));
    assert_eq!(layout["offset"]["bottom"].as_integer(), Some(0));
    assert_eq!(layout["offset"]["right"].as_integer(), Some(80));
}

#[test]
fn preset_source_roundtrip_preserves_unrelated_nodes_and_has_one_undo_record() {
    let mut session = selected_session(CANVAS_SOURCE);
    let original_source = session.source_buffer().text().to_string();
    let mut expected = UiV2AssetLoader::load_toml_str(&original_source).unwrap();
    assert!(apply_preset(&mut session, 15, false, false));
    let changed_source = session.source_buffer().text().to_string();
    let changed = UiV2AssetLoader::load_toml_str(&changed_source).unwrap();
    expected.nodes.get_mut("canvas").unwrap().children[0].slot =
        changed.nodes["canvas"].children[0].slot.clone();
    assert_eq!(changed, expected);
    assert_eq!(
        session.next_undo_label().as_deref(),
        Some("Changed Anchors")
    );

    let reopened = selected_session(&changed_source);
    same_frame(
        node_frame(&reopened, "button"),
        node_frame(&session, "button"),
    );
    let undo = session.undo_replay().unwrap();
    assert!(undo.changed);
    assert_eq!(undo.label, "Changed Anchors");
    assert_eq!(session.source_buffer().text(), original_source);
    assert!(
        !session.undo().unwrap(),
        "one preset must be one undo entry"
    );
    assert!(session.redo().unwrap());
    assert_eq!(session.source_buffer().text(), changed_source);
}

#[test]
fn repeating_a_preset_does_not_add_an_undo_entry_for_a_noop() {
    let mut session = selected_session(CANVAS_SOURCE);
    assert!(apply_preset(&mut session, 15, false, false));
    let changed = session.source_buffer().text().to_string();
    let revision = session.source_revision();
    assert!(!apply_preset(&mut session, 15, false, false));
    assert_eq!(session.source_buffer().text(), changed);
    assert_eq!(session.source_revision(), revision);
    assert!(session.undo().unwrap());
    assert!(!session.can_undo());
}

#[test]
fn invalid_preset_root_and_non_canvas_edges_preserve_source_and_history() {
    let mut session = selected_session(CANVAS_SOURCE);
    let original = session.source_buffer().text().to_string();
    assert!(matches!(
        session.apply_selected_canvas_anchor_preset(16, false, false),
        Err(UiAssetEditorSessionError::InvalidCanvasAnchorPreset { index: 16 })
    ));
    session.select_hierarchy_index(0).unwrap();
    assert!(!apply_preset(&mut session, 0, false, false));
    assert_eq!(session.source_buffer().text(), original);
    assert!(!session.can_undo());

    for kind in ["Overlay", "Free"] {
        let source = CANVAS_SOURCE.replace("kind = \"Canvas\"", &format!("kind = \"{kind}\""));
        let mut session = selected_session(&source);
        let original = session.source_buffer().text().to_string();
        assert!(matches!(
            session.apply_selected_canvas_anchor_preset(15, false, false),
            Err(UiAssetEditorSessionError::CanvasAnchorGeometryUnavailable { .. })
        ));
        assert_eq!(session.source_buffer().text(), original);
        assert!(!session.can_undo());
    }
}

#[test]
fn auto_size_and_zero_parent_content_are_rejected_without_source_changes() {
    let auto_size = CANVAS_SOURCE.replace("auto_size = false", "auto_size = true");
    let zero_parent =
        CANVAS_SOURCE.replace("right = 23, bottom = 19", "right = 2000, bottom = 2000");
    for source in [auto_size, zero_parent] {
        let mut session = selected_session(&source);
        let original = session.source_buffer().text().to_string();
        let revision = session.source_revision();
        assert!(matches!(
            session.apply_selected_canvas_anchor_preset(15, false, false),
            Err(UiAssetEditorSessionError::CanvasAnchorGeometryUnavailable { .. })
        ));
        assert_eq!(session.source_buffer().text(), original);
        assert_eq!(session.source_revision(), revision);
        assert!(!session.can_undo());
    }
}

#[test]
fn a_canvas_child_without_placement_rebases_its_effective_node_layout() {
    let mut value: toml::Value = toml::from_str(CANVAS_SOURCE).unwrap();
    value["nodes"]["canvas"]["children"][0]
        .as_table_mut()
        .unwrap()
        .remove("slot");
    let layout = toml::from_str(
        "width = { preferred = 80 }\nheight = { preferred = 40 }\n\
         anchor = { x = 0.25, y = 0.25 }\npivot = { x = 0.2, y = 0.6 }\n\
         position = { x = 13, y = -7 }",
    )
    .unwrap();
    value["nodes"]["button"]
        .as_table_mut()
        .unwrap()
        .insert("layout".into(), layout);
    let source = toml::to_string_pretty(&value).unwrap();
    let mut session = selected_session(&source);
    let before = node_frame(&session, "button");
    assert!(apply_preset(&mut session, 5, false, false));
    same_frame(node_frame(&session, "button"), before);
    assert!(session.undo().unwrap());
    assert_eq!(session.source_buffer().text(), source);
}

#[test]
fn canvas_slot_semantics_expose_authored_data_without_editing_or_overlay_aliases() {
    let mut session = selected_session(CANVAS_SOURCE);
    let original = session.source_buffer().text().to_string();
    let revision = session.source_revision();
    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_slot_kind, "Canvas");
    assert_eq!(pane.inspector_slot_semantic_title, "Canvas Slot");
    assert_eq!(pane.inspector_slot_semantic_items.len(), 14);
    for label in [
        "layout.anchor_max.x = 0.25",
        "layout.pivot.y = 0.6",
        "layout.offset.right = 80",
        "layout.auto_size = false",
        "layout.z_order = 7",
    ] {
        assert!(pane
            .inspector_slot_semantic_items
            .iter()
            .any(|row| row == label));
    }
    assert!(pane.inspector_slot_overlay_anchor_x.is_empty());
    assert_eq!(session.source_buffer().text(), original);
    assert_eq!(session.source_revision(), revision);
    assert!(!session.can_undo());

    session.select_hierarchy_index(0).unwrap();
    assert!(session
        .pane_presentation()
        .inspector_slot_semantic_items
        .is_empty());
    for kind in ["Overlay", "Free"] {
        let source = CANVAS_SOURCE.replace("kind = \"Canvas\"", &format!("kind = \"{kind}\""));
        let pane = selected_session(&source).pane_presentation();
        assert_ne!(pane.inspector_slot_kind, "Canvas");
        assert_ne!(pane.inspector_slot_semantic_title, "Canvas Slot");
        assert!(!pane
            .inspector_slot_semantic_items
            .iter()
            .any(|row| row.starts_with("layout.anchor_max.")));
    }
}

#[test]
fn canvas_slot_semantic_edit_updates_real_geometry_roundtrip_and_one_undo() {
    let mut session = selected_session(CANVAS_SOURCE);
    let original = session.source_buffer().text().to_string();
    let before = node_frame(&session, "button");
    let index = session
        .pane_presentation()
        .inspector_slot_semantic_items
        .iter()
        .position(|row| row.starts_with("layout.offset.left = "))
        .unwrap();
    assert!(session.select_slot_semantic(index).unwrap());
    assert_eq!(session.source_buffer().text(), original);
    assert!(session.set_selected_slot_semantic_value("125").unwrap());
    let changed_source = session.source_buffer().text().to_string();
    let mut expected: toml::Value = toml::from_str(&original).unwrap();
    expected["nodes"]["canvas"]["children"][0]["slot"]["layout"]["offset"]["left"] =
        toml::Value::Integer(125);
    let changed: toml::Value = toml::from_str(&changed_source).unwrap();
    assert_eq!(changed, expected);
    same_frame(
        node_frame(&session, "button"),
        UiFrame::new(before.x + 100.0, before.y, before.width, before.height),
    );
    let reopened = selected_session(&changed_source);
    same_frame(
        node_frame(&reopened, "button"),
        node_frame(&session, "button"),
    );
    assert_eq!(
        session.pane_presentation().inspector_slot_semantic_path,
        "layout.offset.left"
    );
    let revision = session.source_revision();
    assert!(!session.set_selected_slot_semantic_value("125").unwrap());
    assert_eq!(session.source_revision(), revision);
    assert_eq!(
        session.next_undo_label().as_deref(),
        Some("Slot Semantic Edit")
    );
    assert!(session.undo().unwrap());
    assert_eq!(session.source_buffer().text(), original);
    same_frame(node_frame(&session, "button"), before);
    assert!(!session.can_undo());
    assert!(session.redo().unwrap());
    assert_eq!(session.source_buffer().text(), changed_source);
}

#[test]
fn canvas_slot_semantic_delete_preserves_absence_and_replays_original_history() {
    let mut session = selected_session(CANVAS_SOURCE);
    let original = session.source_buffer().text().to_string();
    let index = session
        .pane_presentation()
        .inspector_slot_semantic_items
        .iter()
        .position(|row| row.starts_with("layout.z_order = "))
        .unwrap();
    assert!(session.select_slot_semantic(index).unwrap());
    assert!(session.delete_selected_slot_semantic().unwrap());
    let changed_source = session.source_buffer().text().to_string();
    let mut expected: toml::Value = toml::from_str(&original).unwrap();
    expected["nodes"]["canvas"]["children"][0]["slot"]["layout"]
        .as_table_mut()
        .unwrap()
        .remove("z_order");
    let changed: toml::Value = toml::from_str(&changed_source).unwrap();
    assert_eq!(changed, expected);
    let reopened = selected_session(&changed_source).pane_presentation();
    assert_eq!(reopened.inspector_slot_kind, "Canvas");
    assert_eq!(reopened.inspector_slot_semantic_items.len(), 13);
    assert!(!reopened
        .inspector_slot_semantic_items
        .iter()
        .any(|row| row.starts_with("layout.z_order = ")));
    let pane = session.pane_presentation();
    assert!(pane.inspector_slot_semantic_path.is_empty());
    assert_eq!(pane.inspector_slot_semantic_selected_index, -1);
    assert!(!session.delete_selected_slot_semantic().unwrap());
    assert_eq!(
        session.next_undo_label().as_deref(),
        Some("Slot Semantic Delete")
    );
    assert!(session.undo().unwrap());
    assert_eq!(session.source_buffer().text(), original);
    assert!(!session.can_undo());
    assert!(session.redo().unwrap());
    assert_eq!(session.source_buffer().text(), changed_source);
}

fn popup_ticket(
    session: &mut UiAssetEditorSession,
) -> crate::ui::asset_editor::UiAssetCanvasAnchorPresetTicket {
    session
        .begin_canvas_anchor_preset()
        .expect("current Canvas geometry")
        .expect("selected native Canvas child")
}

fn assert_retired_popup(
    session: &mut UiAssetEditorSession,
    ticket: &crate::ui::asset_editor::UiAssetCanvasAnchorPresetTicket,
) {
    let source = session.source_buffer().text().to_string();
    let revision = session.source_revision();
    let undo = session.next_undo_label();
    let redo = session.next_redo_label();
    assert!(matches!(
        session.apply_canvas_anchor_preset(ticket, 15, false, false),
        Err(UiAssetEditorSessionError::RetiredCanvasAnchorPreset)
    ));
    assert_eq!(session.source_buffer().text(), source);
    assert_eq!(session.source_revision(), revision);
    assert_eq!(session.next_undo_label(), undo);
    assert_eq!(session.next_redo_label(), redo);
}

#[test]
fn canvas_popup_commit_is_single_use_and_preserves_one_undo_transaction() {
    let mut session = selected_session(CANVAS_SOURCE);
    let source = session.source_buffer().text().to_string();
    let before = node_frame(&session, "button");
    let ticket = popup_ticket(&mut session);
    assert_eq!(session.source_buffer().text(), source);
    assert!(!session.can_undo());
    assert!(session
        .apply_canvas_anchor_preset(&ticket, 15, false, false)
        .unwrap());
    same_frame(node_frame(&session, "button"), before);
    let changed = session.source_buffer().text().to_string();
    assert_retired_popup(&mut session, &ticket.clone());
    let reopened = selected_session(&changed);
    same_frame(node_frame(&reopened, "button"), before);
    assert!(session.undo().unwrap());
    assert_eq!(session.source_buffer().text(), source);
    assert!(!session.can_undo());
    assert!(session.redo().unwrap());
    assert_eq!(session.source_buffer().text(), changed);
}

#[test]
fn canvas_popup_selection_navigation_retires_even_after_return_to_same_node() {
    let mut session = selected_session(CANVAS_SOURCE);
    let ticket = popup_ticket(&mut session);
    session.select_hierarchy_index(2).unwrap();
    session.select_hierarchy_index(1).unwrap();
    assert_retired_popup(&mut session, &ticket);
    assert!(!session.can_undo());
    let fresh = popup_ticket(&mut session);
    assert!(session
        .apply_canvas_anchor_preset(&fresh, 15, false, false)
        .unwrap());
}

#[test]
fn canvas_popup_source_change_and_undo_do_not_revive_old_ticket() {
    use crate::ui::asset_editor::UiAssetEditorCommand;
    let mut session = selected_session(CANVAS_SOURCE);
    let ticket = popup_ticket(&mut session);
    let source = session.source_buffer().text().to_string();
    session
        .apply_command(UiAssetEditorCommand::edit_source(format!(
            "{source}\n# current revision"
        )))
        .unwrap();
    assert_retired_popup(&mut session, &ticket);
    assert!(session.undo().unwrap());
    assert_eq!(session.source_buffer().text(), source);
    session.select_hierarchy_index(1).unwrap();
    assert_retired_popup(&mut session, &ticket);
    assert!(!session.can_undo());
}

#[test]
fn canvas_popup_current_invalid_source_never_edits_last_valid_document() {
    use crate::ui::asset_editor::UiAssetEditorCommand;
    let mut session = selected_session(CANVAS_SOURCE);
    let ticket = popup_ticket(&mut session);
    session
        .apply_command(UiAssetEditorCommand::edit_source(format!(
            "{CANVAS_SOURCE}\n[broken"
        )))
        .unwrap();
    assert!(!session.diagnostics().is_empty());
    assert_retired_popup(&mut session, &ticket);
    assert!(matches!(
        session.begin_canvas_anchor_preset(),
        Err(UiAssetEditorSessionError::InvalidSourceBuffer)
    ));
}

#[test]
fn canvas_popup_preview_resize_or_mode_return_rejects_old_geometry() {
    use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetPreviewPreset};
    let mut session = selected_session(CANVAS_SOURCE);
    let ticket = popup_ticket(&mut session);
    assert!(session
        .set_preview_preset(UiAssetPreviewPreset::EditorFloating)
        .unwrap());
    assert_retired_popup(&mut session, &ticket);
    assert!(!session.can_undo());
    let ticket = popup_ticket(&mut session);
    session.set_mode(UiAssetEditorMode::Source).unwrap();
    assert!(session.begin_canvas_anchor_preset().unwrap().is_none());
    session.set_mode(UiAssetEditorMode::Design).unwrap();
    session.select_hierarchy_index(1).unwrap();
    assert_retired_popup(&mut session, &ticket);
    assert!(!session.can_undo());
}

#[test]
fn canvas_popup_old_cancel_and_submit_do_not_retire_a_newer_popup() {
    let mut session = selected_session(CANVAS_SOURCE);
    let old = popup_ticket(&mut session);
    let current = popup_ticket(&mut session);
    assert!(!session.cancel_canvas_anchor_preset(&old));
    assert_retired_popup(&mut session, &old);
    assert!(session
        .apply_canvas_anchor_preset(&current, 15, false, false)
        .unwrap());
    let current = popup_ticket(&mut session);
    assert!(session.cancel_canvas_anchor_preset(&current));
    assert!(!session.cancel_canvas_anchor_preset(&current));
    assert_retired_popup(&mut session, &current);
}

#[test]
fn canvas_popup_reopened_document_rejects_other_session_ticket() {
    let mut original = selected_session(CANVAS_SOURCE);
    let ticket = popup_ticket(&mut original);
    let mut reopened = selected_session(CANVAS_SOURCE);
    let current = popup_ticket(&mut reopened);
    assert_retired_popup(&mut reopened, &ticket);
    assert!(!reopened.cancel_canvas_anchor_preset(&ticket));
    assert!(reopened
        .apply_canvas_anchor_preset(&current, 15, false, false)
        .unwrap());
    assert!(original.cancel_canvas_anchor_preset(&ticket));
    assert!(!original.can_undo());
}

#[test]
fn canvas_popup_invalid_index_is_retryable_but_noop_and_direct_command_retire_ticket() {
    let mut session = selected_session(CANVAS_SOURCE);
    let ticket = popup_ticket(&mut session);
    assert!(matches!(
        session.apply_canvas_anchor_preset(&ticket, 16, false, false),
        Err(UiAssetEditorSessionError::InvalidCanvasAnchorPreset { index: 16 })
    ));
    assert!(session
        .apply_canvas_anchor_preset(&ticket, 15, false, false)
        .unwrap());
    let changed = session.source_buffer().text().to_string();
    let ticket = popup_ticket(&mut session);
    assert!(!session
        .apply_canvas_anchor_preset(&ticket, 15, false, false)
        .unwrap());
    assert_retired_popup(&mut session, &ticket);
    let ticket = popup_ticket(&mut session);
    assert!(!apply_preset(&mut session, 15, false, false));
    assert_retired_popup(&mut session, &ticket);
    assert_eq!(session.source_buffer().text(), changed);
    assert!(session.undo().unwrap());
    assert!(!session.can_undo());
}

fn default_canvas_slot_source() -> String {
    let mut source: toml::Value = toml::from_str(CANVAS_SOURCE).unwrap();
    assert!(source["nodes"]["canvas"]["children"][0]
        .as_table_mut()
        .unwrap()
        .remove("slot")
        .is_some());
    toml::to_string(&source).unwrap()
}

#[test]
fn canvas_current_default_preset_keeps_implicit_source_and_history_unchanged() {
    let source = default_canvas_slot_source();
    for (shift, control) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut session = selected_session(&source);
        let before = node_frame(&session, "button");
        assert!(before.width > 0.0 && before.height > 0.0);
        let revision = session.source_revision();
        let ticket = popup_ticket(&mut session);
        assert!(!session
            .apply_canvas_anchor_preset(&ticket, 0, shift, control)
            .unwrap());
        assert_eq!(session.source_buffer().text(), source);
        assert_eq!(session.source_revision(), revision);
        assert!(!session.can_undo());
        same_frame(node_frame(&session, "button"), before);
        assert_retired_popup(&mut session, &ticket);
    }
}

#[test]
fn canvas_point_anchor_move_preserves_intrinsic_sizing_and_implicit_maximum() {
    let source = default_canvas_slot_source();
    let mut session = selected_session(&source);
    let before = node_frame(&session, "button");
    assert!(apply_preset(&mut session, 5, false, false));
    let changed = session.source_buffer().text().to_string();
    let parsed: toml::Value = toml::from_str(&changed).unwrap();
    let layout = &parsed["nodes"]["canvas"]["children"][0]["slot"]["layout"];
    assert!(layout.get("anchor_max").is_none());
    assert!(layout.get("pivot").is_none());
    assert!(layout.get("position").is_none());
    let offset = &layout["offset"];
    assert!(offset.get("right").is_none());
    assert!(offset.get("bottom").is_none());
    same_frame(node_frame(&session, "button"), before);
    same_frame(node_frame(&selected_session(&changed), "button"), before);
    let mut expected = UiV2AssetLoader::load_toml_str(&source).unwrap();
    let actual = UiV2AssetLoader::load_toml_str(&changed).unwrap();
    expected.nodes.get_mut("canvas").unwrap().children[0].slot =
        actual.nodes["canvas"].children[0].slot.clone();
    assert_eq!(actual, expected);
    assert_eq!(session.undo_replay().unwrap().label, "Changed Anchors");
    assert_eq!(session.source_buffer().text(), source);
    assert!(!session.can_undo());
    assert!(session.redo().unwrap());
    assert_eq!(session.source_buffer().text(), changed);
}

#[test]
fn canvas_first_stretch_materializes_complete_maximum_and_reopens_same_rectangle() {
    let source = default_canvas_slot_source();
    let mut session = selected_session(&source);
    let before = node_frame(&session, "button");
    assert!(apply_preset(&mut session, 3, false, false));
    let changed = session.source_buffer().text().to_string();
    let parsed: toml::Value = toml::from_str(&changed).unwrap();
    let layout = &parsed["nodes"]["canvas"]["children"][0]["slot"]["layout"];
    let maximum = layout["anchor_max"].as_table().unwrap();
    assert_eq!(maximum.len(), 2);
    assert_eq!(maximum["x"].as_integer(), Some(1));
    assert_eq!(maximum["y"].as_integer(), Some(0));
    assert!(layout["offset"].get("bottom").is_none());
    same_frame(node_frame(&session, "button"), before);
    same_frame(node_frame(&selected_session(&changed), "button"), before);
    assert_eq!(
        session.next_undo_label().as_deref(),
        Some("Changed Anchors")
    );
    assert!(session.undo().unwrap());
    assert_eq!(session.source_buffer().text(), source);
    assert!(!session.can_undo());
}

fn node_placement_canvas_source() -> String {
    let mut value: toml::Value = toml::from_str(CANVAS_SOURCE).unwrap();
    value["nodes"]["canvas"]["children"][0]
        .as_table_mut()
        .unwrap()
        .remove("slot");
    value["nodes"]["button"].as_table_mut().unwrap().insert(
        "layout".into(),
        toml::from_str(
            "width = { preferred = 80 }\nheight = { preferred = 40 }\n\
             anchor = { x = 0.25, y = 0.5 }\npivot = { x = 0.2, y = 0.6 }\n\
             position = { x = 13, y = -7 }",
        )
        .unwrap(),
    );
    toml::to_string_pretty(&value).unwrap()
}

#[test]
fn canvas_node_placement_materialization_keeps_unchanged_axis_pivot_and_position() {
    let source = node_placement_canvas_source();
    let mut session = selected_session(&source);
    let before = node_frame(&session, "button");
    let ticket = popup_ticket(&mut session);
    // Centre/centre changes X only; Y, pivot and position must survive the
    // switch from node placement to an explicit slot, not just the rebase math.
    assert!(session
        .apply_canvas_anchor_preset(&ticket, 5, false, false)
        .unwrap());
    same_frame(node_frame(&session, "button"), before);
    assert_retired_popup(&mut session, &ticket);
    let changed = session.source_buffer().text().to_string();
    let parsed: toml::Value = toml::from_str(&changed).unwrap();
    let layout = &parsed["nodes"]["canvas"]["children"][0]["slot"]["layout"];
    close(layout["anchor"]["y"].as_float().unwrap() as f32, 0.5);
    close(layout["pivot"]["x"].as_float().unwrap() as f32, 0.2);
    close(layout["pivot"]["y"].as_float().unwrap() as f32, 0.6);
    assert_eq!(layout["position"]["x"].as_integer(), Some(13));
    assert_eq!(layout["position"]["y"].as_integer(), Some(-7));
    assert!(layout.get("anchor_max").is_none());
    assert!(layout["offset"].get("right").is_none());
    assert!(layout["offset"].get("bottom").is_none());
    let actual = UiV2AssetLoader::load_toml_str(&changed).unwrap();
    let mut expected = UiV2AssetLoader::load_toml_str(&source).unwrap();
    expected.nodes.get_mut("canvas").unwrap().children[0].slot =
        actual.nodes["canvas"].children[0].slot.clone();
    assert_eq!(actual, expected, "only the selected source edge changes");
    same_frame(node_frame(&selected_session(&changed), "button"), before);
    assert_eq!(session.undo_replay().unwrap().label, "Changed Anchors");
    assert_eq!(session.source_buffer().text(), source);
    assert!(!session.can_undo());
    assert!(session.redo().unwrap());
    assert_eq!(session.source_buffer().text(), changed);
}

#[test]
fn canvas_node_placement_noop_and_cancel_keep_the_slot_absent() {
    let mut value: toml::Value = toml::from_str(&node_placement_canvas_source()).unwrap();
    value["nodes"]["button"]["layout"]["anchor"]["x"] = toml::Value::Float(0.5);
    value["nodes"]["button"]["layout"]["pivot"]["x"] = toml::Value::Float(0.5);
    value["nodes"]["button"]["layout"]["pivot"]["y"] = toml::Value::Float(0.5);
    value["nodes"]["button"]["layout"]["position"]["x"] = toml::Value::Integer(0);
    value["nodes"]["button"]["layout"]["position"]["y"] = toml::Value::Integer(0);
    let source = toml::to_string_pretty(&value).unwrap();
    for (shift, control) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut session = selected_session(&source);
        let before = node_frame(&session, "button");
        let revision = session.source_revision();
        let cancelled = popup_ticket(&mut session);
        assert!(session.cancel_canvas_anchor_preset(&cancelled));
        assert_retired_popup(&mut session, &cancelled);
        let ticket = popup_ticket(&mut session);
        assert!(!session
            .apply_canvas_anchor_preset(&ticket, 5, shift, control)
            .unwrap());
        assert_retired_popup(&mut session, &ticket);
        assert_eq!(session.source_buffer().text(), source);
        assert_eq!(session.source_revision(), revision);
        assert!(!session.can_undo());
        assert!(!session.can_redo());
        same_frame(node_frame(&session, "button"), before);
        assert!(
            UiV2AssetLoader::load_toml_str(session.source_buffer().text())
                .unwrap()
                .nodes["canvas"]
                .children[0]
                .slot
                .is_empty()
        );
    }
}

#[test]
fn canvas_node_placement_modifier_matrix_uses_one_original_ticket_and_undo() {
    const POINTS: [(f32, f32); 4] = [(0.0, 0.0), (0.5, 0.5), (1.0, 1.0), (0.0, 1.0)];
    let source = node_placement_canvas_source();
    for index in 0..16 {
        let (min_x, max_x) = POINTS[index % 4];
        let (min_y, max_y) = POINTS[index / 4];
        for (shift, control) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut session = selected_session(&source);
            let before = node_frame(&session, "button");
            let parent = parent_content(&session);
            let ticket = popup_ticket(&mut session);
            assert!(session
                .apply_canvas_anchor_preset(&ticket, index, shift, control)
                .unwrap());
            assert_retired_popup(&mut session, &ticket);
            let pivot_x = if shift { min_x } else { 0.2 };
            let pivot_y = if shift { min_y } else { 0.6 };
            let expected = if control {
                UiFrame::new(
                    parent.x + parent.width * min_x
                        - if min_x == max_x {
                            before.width * pivot_x
                        } else {
                            0.0
                        },
                    parent.y + parent.height * min_y
                        - if min_y == max_y {
                            before.height * pivot_y
                        } else {
                            0.0
                        },
                    if min_x == max_x {
                        before.width
                    } else {
                        parent.width
                    },
                    if min_y == max_y {
                        before.height
                    } else {
                        parent.height
                    },
                )
            } else {
                UiFrame::new(
                    before.x
                        + if shift && min_x == max_x {
                            before.width * (0.2 - pivot_x)
                        } else {
                            0.0
                        },
                    before.y
                        + if shift && min_y == max_y {
                            before.height * (0.6 - pivot_y)
                        } else {
                            0.0
                        },
                    before.width,
                    before.height,
                )
            };
            assert!(session.diagnostics().is_empty());
            same_frame(node_frame(&session, "button"), expected);
            let changed = session.source_buffer().text().to_string();
            same_frame(node_frame(&selected_session(&changed), "button"), expected);
            assert_eq!(session.undo_replay().unwrap().label, "Changed Anchors");
            assert_eq!(session.source_buffer().text(), source);
            assert!(!session.can_undo());
            assert!(session.redo().unwrap());
            assert_eq!(session.source_buffer().text(), changed);
        }
    }
}
