use zircon_runtime::ui::{template::UiAssetDocumentRuntimeExt, v2::UiV2AssetLoader};
use zircon_runtime_interface::ui::{
    layout::{UiAlignment, UiAlignment2D, UiMargin, UiSize, UiSlot, UiSlotKind},
    template::UiAssetKind,
    v2::{UiV2AssetDocument, UiV2ChildMount},
};

use super::*;
use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute};

const SLOT_MIGRATION_V2: &str = r#"
[asset]
kind = "view"
id = "editor.test.slot_migration"
version = 2
[root]
node = "root"
[components.Badge]
root = "badge_root"
[nodes.root]
component = "Container"
control_id = "Root"
children = [
  { node = "subject", slot = { name = "content", layout = { padding = { left = 7, top = 3, right = 9, bottom = 5 }, alignment = { horizontal = "Center", vertical = "End" }, order = 4, anchor = { x = 0.2, y = 0.25 }, anchor_max = { x = 0.7, y = 0.75 }, pivot = { x = 0.1, y = 0.2 }, position = { x = 13, y = -7 }, offset = { left = 25, top = 30, right = 80, bottom = 40 }, auto_size = false, z_order = 7, linear_size = { rule = "Stretch", value = 2.5, stretch_value = 2.5 }, row = 1, column = 2, row_span = 2, column_span = 3 } } },
  { node = "target", slot = { name = "content" } },
  { node = "untouched", slot = { name = "content" } }
]
[nodes.root.layout.container]
kind = "ROOT_CONTAINER_KIND"
[nodes.subject]
component = "Button"
control_id = "Subject"
props = { text = "保留控件、事件和源身份" }
events = [{ id = "Widget/SlotActivate", event = "Click", route = "widget.activate" }]
[nodes.target]
component = "Container"
control_id = "Target"
[nodes.target.layout.container]
kind = "TARGET_CONTAINER_KIND"
[nodes.untouched]
component = "Badge"
control_id = "Untouched"
[nodes.badge_root]
component = "Text"
control_id = "BadgeRoot"
props = { text = "Existing local component" }
"#;

fn slot_migration_source(root_kind: &str, target_kind: &str) -> String {
    SLOT_MIGRATION_V2
        .replace("ROOT_CONTAINER_KIND", root_kind)
        .replace("TARGET_CONTAINER_KIND", target_kind)
}

fn slot_migration_session(source: &str) -> UiAssetEditorSession {
    let session = UiAssetEditorSession::from_v2_source(
        UiAssetEditorRoute::new(
            "editor.test.slot_migration",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        ),
        source,
        UiSize::new(640.0, 520.0),
    )
    .expect("real v2 session");
    assert!(
        session.diagnostics().is_empty(),
        "{:?}",
        session.diagnostics()
    );
    session
}

fn parsed(session: &UiAssetEditorSession) -> UiV2AssetDocument {
    UiV2AssetLoader::load_toml_str(session.source_buffer().text()).expect("saved v2 source")
}

fn select_subject(session: &mut UiAssetEditorSession) {
    session.selection = selection_for_node(&session.last_valid_document, "subject");
}

fn moved_slot<'a>(
    document: &'a UiV2AssetDocument,
    parent: &str,
) -> &'a std::collections::BTreeMap<String, toml::Value> {
    &document.nodes[parent]
        .children
        .iter()
        .find(|child| child.node == "subject")
        .expect("moved original subject")
        .slot
}

// Read the actual v2-built preview tree; source IDs locate the new runtime IDs.
fn preview_subject_slot(session: &UiAssetEditorSession, parent_source: &str) -> UiSlot {
    let tree = &session.preview_host().surface().tree;
    let mapped = |source: &str| {
        let mut nodes = tree.nodes.iter().filter(|(_, node)| {
            node.template_metadata
                .as_ref()
                .and_then(|m| m.source_node_id.as_deref())
                == Some(source)
        });
        let (id, _) = nodes.next().expect("source-mapped native node");
        assert!(
            nodes.next().is_none(),
            "fixture source mapping must be unambiguous"
        );
        *id
    };
    let parent = mapped(parent_source);
    let child = mapped("subject");
    let mut slots = tree
        .layout_slots()
        .iter()
        .filter(|slot| slot.parent_id == parent && slot.child_id == child);
    let slot = slots.next().expect("real preview edge");
    assert!(
        slots.next().is_none(),
        "fixture edge has one native layout slot"
    );
    slot.clone()
}

fn assert_preview_portable(slot: &UiSlot) {
    assert_eq!(slot.padding, UiMargin::new(7.0, 3.0, 9.0, 5.0));
    assert_eq!(
        slot.alignment,
        UiAlignment2D {
            horizontal: UiAlignment::Center,
            vertical: UiAlignment::End
        }
    );
    assert_eq!(slot.order, 4);
}

#[test]
fn hierarchy_slot_migration_cross_family_keeps_portable_values_and_original_v2_nodes() {
    let mut session = slot_migration_session(&slot_migration_source("Canvas", "VerticalBox"));
    let before = parsed(&session);
    select_subject(&mut session);
    assert!(session.pane_presentation().can_reparent_into_next);
    assert!(session
        .reparent_selected_node_into_next()
        .expect("reparent"));
    let after = parsed(&session);
    let original = before.nodes["root"].children[0].slot["layout"]
        .as_table()
        .expect("original layout");
    let moved = moved_slot(&after, "target");
    assert_eq!(moved["name"].as_str(), Some("content"));
    let layout = moved["layout"].as_table().expect("portable layout");
    assert_eq!(layout.len(), 3);
    for key in ["padding", "alignment", "order"] {
        assert_eq!(layout.get(key), original.get(key));
    }
    for (id, node) in &before.nodes {
        if id != "root" && id != "target" {
            assert_eq!(after.nodes.get(id), Some(node), "preserve {id}");
        }
    }
    assert_eq!(after.components, before.components);
    assert_eq!(after.imports, before.imports);
    assert_eq!(after.tokens, before.tokens);
    assert_eq!(after.stylesheets, before.stylesheets);
    assert_eq!(
        session.selection.primary_node_id.as_deref(),
        Some("subject")
    );
    assert_eq!(session.selection.parent_node_id.as_deref(), Some("target"));
    let reopened = slot_migration_session(session.source_buffer().text());
    assert_eq!(parsed(&reopened), after);
    assert!(reopened.preview_host.is_some());
    let slot = preview_subject_slot(&reopened, "target");
    assert_eq!(slot.kind, UiSlotKind::Linear);
    assert_preview_portable(&slot);
    assert!(slot.canvas_placement.is_none());
    assert!(slot.grid_placement.is_none());
    assert!(slot.linear_sizing.is_none());
    assert_eq!(slot.z_order, 0);
}

#[test]
fn hierarchy_slot_migration_linear_orientation_keeps_the_original_slot_contract() {
    let mut session =
        slot_migration_session(&slot_migration_source("HorizontalBox", "VerticalBox"));
    let before = parsed(&session);
    select_subject(&mut session);
    assert!(session
        .reparent_selected_node_into_next()
        .expect("linear reparent"));
    assert_eq!(
        moved_slot(&parsed(&session), "target"),
        &before.nodes["root"].children[0].slot
    );
    let slot = preview_subject_slot(&session, "target");
    assert_eq!(slot.kind, UiSlotKind::Linear);
    assert_preview_portable(&slot);
    assert_eq!(
        slot.linear_sizing
            .expect("consumed canonical linear size")
            .value,
        2.5
    );
}

#[test]
fn hierarchy_slot_migration_grid_alias_keeps_authored_row_column_and_spans() {
    let mut session = slot_migration_session(&slot_migration_source("GridGroup", "GridBox"));
    let before = parsed(&session);
    select_subject(&mut session);
    assert!(session
        .reparent_selected_node_into_next()
        .expect("grid reparent"));
    assert_eq!(
        moved_slot(&parsed(&session), "target"),
        &before.nodes["root"].children[0].slot
    );
    let slot = preview_subject_slot(&session, "target");
    assert_eq!(slot.kind, UiSlotKind::Grid);
    assert_preview_portable(&slot);
    let grid = slot.grid_placement.expect("consumed grid placement");
    assert_eq!(
        (grid.row, grid.column, grid.row_span, grid.column_span),
        (1, 2, 2, 3)
    );
}

#[test]
fn hierarchy_slot_migration_canvas_free_and_overlay_copy_only_shared_placement_fields() {
    for (target, kept, removed) in [
        (
            "Free",
            vec![
                "anchor",
                "anchor_max",
                "pivot",
                "position",
                "offset",
                "auto_size",
            ],
            vec!["z_order", "linear_size", "row"],
        ),
        (
            "Overlay",
            vec!["z_order"],
            vec![
                "anchor",
                "anchor_max",
                "pivot",
                "position",
                "offset",
                "auto_size",
                "linear_size",
                "row",
            ],
        ),
    ] {
        let mut session = slot_migration_session(&slot_migration_source("Canvas", target));
        let before = parsed(&session);
        let original_runtime = preview_subject_slot(&session, "root");
        select_subject(&mut session);
        assert!(session
            .reparent_selected_node_into_next()
            .expect("placement reparent"));
        let after = parsed(&session);
        let layout = moved_slot(&after, "target")["layout"]
            .as_table()
            .expect("migrated layout");
        let original = before.nodes["root"].children[0].slot["layout"]
            .as_table()
            .expect("old layout");
        for key in kept {
            assert_eq!(layout.get(key), original.get(key), "{target}: {key}");
        }
        for key in removed {
            assert!(!layout.contains_key(key), "{target}: {key}");
        }
        let runtime = preview_subject_slot(&session, "target");
        assert_preview_portable(&runtime);
        if target == "Free" {
            assert_eq!(runtime.kind, UiSlotKind::Free);
            assert!(original_runtime.canvas_placement.is_some());
            assert_eq!(runtime.canvas_placement, original_runtime.canvas_placement);
            assert_eq!(runtime.z_order, 0);
        } else {
            assert_eq!(runtime.kind, UiSlotKind::Overlay);
            assert!(runtime.canvas_placement.is_none());
            assert_eq!(runtime.z_order, 7);
        }
    }
}

#[test]
fn hierarchy_slot_migration_uses_one_existing_history_step_and_restores_old_edge() {
    let mut session = slot_migration_session(&slot_migration_source("Canvas", "VerticalBox"));
    select_subject(&mut session);
    let before = session.source_buffer().text().to_string();
    assert!(!session.can_undo());
    assert!(session
        .reparent_selected_node_into_next()
        .expect("reparent"));
    let moved = session.source_buffer().text().to_string();
    assert_ne!(moved, before);
    assert!(session.undo().expect("one undo"));
    assert_eq!(session.source_buffer().text(), before);
    assert_eq!(session.selection.parent_node_id.as_deref(), Some("root"));
    assert!(!session.can_undo());
    assert!(session.redo().expect("one redo"));
    assert_eq!(session.source_buffer().text(), moved);
    assert_eq!(session.selection.parent_node_id.as_deref(), Some("target"));
    assert!(!session.can_redo());
}

fn named_slot_session(outdent: bool, occupied: bool) -> UiAssetEditorSession {
    let mut document = UiV2AssetLoader::load_toml_str(&slot_migration_source("Canvas", "Canvas"))
        .expect("named fixture source");
    let named = |node: &str, name: &str| UiV2ChildMount {
        node: node.to_string(),
        slot: std::collections::BTreeMap::from([(
            "name".to_string(),
            toml::Value::String(name.to_string()),
        )]),
    };
    let mut subject = document.nodes["root"].children[0].clone();
    document.nodes.get_mut("root").expect("root").component = "PropertyRow".to_string();
    if outdent {
        subject
            .slot
            .insert("slot_name".to_string(), "content".into());
        document.nodes.get_mut("root").expect("root").children = vec![named("target", "label")];
        document.nodes.get_mut("target").expect("target").children = vec![subject];
        if occupied {
            document
                .nodes
                .get_mut("root")
                .expect("root")
                .children
                .push(named("untouched", "field"));
        }
    } else {
        subject.slot.insert("name".to_string(), "label".into());
        subject.slot.insert("slot_name".to_string(), "label".into());
        document.nodes.get_mut("root").expect("root").children =
            vec![subject, named("target", "field")];
        let target = document.nodes.get_mut("target").expect("target");
        target.component = "PropertyRow".to_string();
        target.children = vec![named("untouched", "label")];
        if occupied {
            target.children.push(named("blocker", "field"));
            let mut blocker = document.nodes["untouched"].clone();
            blocker.control_id = Some("Blocker".to_string());
            document.nodes.insert("blocker".to_string(), blocker);
        }
    }
    slot_migration_session(&toml::to_string_pretty(&document).expect("named v2 source"))
}

#[test]
fn hierarchy_slot_migration_named_destination_replaces_both_old_v2_mount_aliases() {
    for outdent in [false, true] {
        let mut session = named_slot_session(outdent, false);
        select_subject(&mut session);
        if outdent {
            assert!(session.pane_presentation().can_reparent_outdent);
            assert!(session.reparent_selected_node_outdent().expect("outdent"));
        } else {
            assert!(session.pane_presentation().can_reparent_into_next);
            assert!(session
                .reparent_selected_node_into_next()
                .expect("into named sibling"));
        }
        let after = parsed(&session);
        let slot = moved_slot(&after, if outdent { "root" } else { "target" });
        assert_eq!(slot["name"].as_str(), Some("field"));
        assert!(!slot.contains_key("slot_name"));
        assert_eq!(
            session
                .last_valid_document
                .child_mount("subject")
                .expect("new edge")
                .mount
                .as_deref(),
            Some("field")
        );
        assert_eq!(
            parsed(&slot_migration_session(session.source_buffer().text())),
            after
        );
    }
}

#[test]
fn hierarchy_slot_migration_full_named_parent_rejects_before_removal_or_history() {
    for outdent in [false, true] {
        let mut session = named_slot_session(outdent, true);
        select_subject(&mut session);
        let source = session.source_buffer().text().to_string();
        let before = session.last_valid_document.clone();
        let direction = if outdent {
            UiTreeReparentDirection::Outdent
        } else {
            UiTreeReparentDirection::IntoNext
        };
        let mut candidate = before.clone();
        assert!(
            tree_reparent_selected_node(&mut candidate, &session.selection, direction).is_none()
        );
        assert_eq!(candidate, before);
        if outdent {
            assert!(!session.pane_presentation().can_reparent_outdent);
            assert!(!session
                .reparent_selected_node_outdent()
                .expect("reject full root"));
        } else {
            assert!(!session.pane_presentation().can_reparent_into_next);
            assert!(!session
                .reparent_selected_node_into_next()
                .expect("reject full sibling"));
        }
        assert_eq!(session.source_buffer().text(), source);
        assert_eq!(session.last_valid_document, before);
        assert_eq!(
            session.selection.primary_node_id.as_deref(),
            Some("subject")
        );
        assert!(!session.can_undo());
    }
}
