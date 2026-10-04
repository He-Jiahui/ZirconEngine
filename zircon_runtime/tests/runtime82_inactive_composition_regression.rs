#![cfg(feature = "ui")]

use zircon_runtime::ui::{
    component::UiComponentDescriptorRegistry,
    dispatch::UiInputManager,
    surface::{UiPropertyMutationRequest, UiSurface},
    template::{UiAssetLoader, UiDocumentCompiler, UiTemplateSurfaceBuilder},
};
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::{
        UiDispatchHostRequestKind, UiImeInputEvent, UiImeInputEventKind, UiInputDispatchResult,
        UiInputEvent, UiInputEventMetadata, UiInputMethodRequestKind, UiInputSequence,
        UiInputTimestamp, UiKeyboardInputEvent, UiKeyboardInputState, UiTextByteRange,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiSize},
    surface::{UiEditableTextState, UiRenderCommandKind},
    text::UiTextEditReceipt,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract, UiWidgetEvent},
};

const INPUT: UiNodeId = UiNodeId::new(2);

#[test]
fn consecutive_backspaces_and_undos_keep_one_retained_document_and_inactive_composition() {
    let mut surface = editable_surface("abc", 3, None);
    let mut manager = UiInputManager::default();

    let first = key(&mut surface, &mut manager, "Backspace", 8, false);
    let first = edit_receipt(&first).clone();
    assert_eq!(body(&surface), "ab");
    assert_eq!(first.previous_revision.get(), 0);
    assert_eq!(first.revision.get(), 1);

    let second = key(&mut surface, &mut manager, "Backspace", 8, false);
    let second = edit_receipt(&second).clone();
    assert_eq!(body(&surface), "a");
    assert_eq!(second.document_id, first.document_id);
    assert_eq!(second.previous_revision, first.revision);
    assert_eq!(second.revision.get(), 2);
    assert!(render_editable(&mut surface).composition.is_none());

    let undo_second = key(&mut surface, &mut manager, "z", 90, true);
    let undo_second = edit_receipt(&undo_second).clone();
    assert_eq!(body(&surface), "ab");
    assert_eq!(undo_second.document_id, first.document_id);
    assert_eq!(undo_second.previous_revision, second.revision);
    assert_eq!(undo_second.revision.get(), 3);

    let undo_first = key(&mut surface, &mut manager, "z", 90, true);
    let undo_first = edit_receipt(&undo_first);
    assert_eq!(body(&surface), "abc");
    assert_eq!(undo_first.document_id, first.document_id);
    assert_eq!(undo_first.previous_revision, undo_second.revision);
    assert_eq!(undo_first.revision.get(), 4);
    assert!(render_editable(&mut surface).composition.is_none());
}

#[test]
fn empty_preedit_over_selection_stays_active_and_cancel_restores_the_source() {
    let mut cancelled = editable_surface("abcd", 3, Some((1, 3)));
    let mut manager = UiInputManager::default();
    cancelled.input.input_method_owner = Some(INPUT);
    let preedit = ime(
        &mut cancelled,
        &mut manager,
        UiImeInputEventKind::Preedit,
        "",
    );
    assert!(preedit.widget_events.is_empty());
    assert_eq!(body(&cancelled), "ad");
    let composition = render_editable(&mut cancelled)
        .composition
        .expect("empty preedit replacing selected text remains active");
    assert_eq!((composition.range.start, composition.range.end), (1, 1));
    assert_eq!(composition.restore_text.as_deref(), Some("bc"));

    let cancel = ime(
        &mut cancelled,
        &mut manager,
        UiImeInputEventKind::Cancel,
        "",
    );
    assert!(cancel.widget_events.is_empty());
    assert_eq!(body(&cancelled), "abcd");
    assert!(render_editable(&mut cancelled).composition.is_none());
    let edit = key(&mut cancelled, &mut manager, "Backspace", 8, false);
    assert_eq!(body(&cancelled), "abd");
    assert_eq!(edit_receipt(&edit).previous_revision.get(), 0);
    assert_eq!(edit_receipt(&edit).revision.get(), 1);
}

#[test]
fn empty_preedit_over_selection_commits_one_undoable_deletion() {
    let mut committed = editable_surface("abcd", 3, Some((1, 3)));
    let mut manager = UiInputManager::default();
    committed.input.input_method_owner = Some(INPUT);
    ime(
        &mut committed,
        &mut manager,
        UiImeInputEventKind::Preedit,
        "",
    );
    assert_eq!(body(&committed), "ad");
    assert!(render_editable(&mut committed).composition.is_some());
    let commit = ime(
        &mut committed,
        &mut manager,
        UiImeInputEventKind::Commit,
        "",
    );
    let receipt = edit_receipt(&commit).clone();
    assert_eq!(body(&committed), "ad");
    assert_eq!(
        (receipt.changed.old.start_byte, receipt.changed.old.end_byte),
        (1, 3)
    );
    assert_eq!(
        (receipt.changed.new.start_byte, receipt.changed.new.end_byte),
        (1, 1)
    );
    assert_eq!(receipt.previous_revision.get(), 0);
    assert_eq!(receipt.revision.get(), 1);
    let undo = key(&mut committed, &mut manager, "z", 90, true);
    assert_eq!(edit_receipt(&undo).document_id, receipt.document_id);
    assert_eq!(edit_receipt(&undo).previous_revision, receipt.revision);
    assert_eq!(edit_receipt(&undo).revision.get(), 2);
    assert_eq!(body(&committed), "abcd");
    assert!(render_editable(&mut committed).composition.is_none());
}

#[test]
fn cancelled_nonempty_preedit_keeps_prior_document_revision_and_undo_history() {
    let mut surface = editable_surface("abcd", 4, None);
    let mut manager = UiInputManager::default();
    let first = key(&mut surface, &mut manager, "Backspace", 8, false);
    let first = edit_receipt(&first).clone();
    assert_eq!(body(&surface), "abc");
    surface.input.input_method_owner = Some(INPUT);
    let preedit = ime(
        &mut surface,
        &mut manager,
        UiImeInputEventKind::Preedit,
        "X",
    );
    assert!(preedit.widget_events.is_empty());
    assert_eq!(body(&surface), "abcX");
    let cancel = ime(&mut surface, &mut manager, UiImeInputEventKind::Cancel, "");
    assert!(cancel.widget_events.is_empty());
    assert_eq!(body(&surface), "abc");

    let second = key(&mut surface, &mut manager, "Backspace", 8, false);
    let second = edit_receipt(&second).clone();
    assert_eq!(body(&surface), "ab");
    assert_eq!(second.document_id, first.document_id);
    assert_eq!(second.previous_revision, first.revision);
    assert_eq!(second.revision.get(), 2);
    let undo_second = key(&mut surface, &mut manager, "z", 90, true);
    assert_eq!(edit_receipt(&undo_second).revision.get(), 3);
    assert_eq!(body(&surface), "abc");
    let undo_first = key(&mut surface, &mut manager, "z", 90, true);
    assert_eq!(edit_receipt(&undo_first).revision.get(), 4);
    assert_eq!(body(&surface), "abcd");
}

#[test]
fn escape_cancelled_preedit_keeps_the_previous_document_and_undo_history() {
    let mut surface = editable_surface("abcd", 4, None);
    let mut manager = UiInputManager::default();
    let first = key(&mut surface, &mut manager, "Backspace", 8, false);
    let first = edit_receipt(&first).clone();
    assert_eq!(body(&surface), "abc");
    surface.input.input_method_owner = Some(INPUT);
    ime(
        &mut surface,
        &mut manager,
        UiImeInputEventKind::Preedit,
        "X",
    );
    assert_eq!(body(&surface), "abcX");

    let cancelled = key(&mut surface, &mut manager, "Escape", 27, false);
    assert!(cancelled.widget_events.is_empty());
    assert_eq!(body(&surface), "abc");
    assert!(render_editable(&mut surface).composition.is_none());
    let undo = key(&mut surface, &mut manager, "z", 90, true);
    let undo = edit_receipt(&undo);
    assert_eq!(undo.document_id, first.document_id);
    assert_eq!(undo.previous_revision, first.revision);
    assert_eq!(undo.revision.get(), 2);
    assert_eq!(body(&surface), "abcd");
}

#[test]
fn matched_ime_commit_preserves_history_when_visible_text_returns_to_the_source() {
    let mut surface = editable_surface("abcd", 4, None);
    let mut manager = UiInputManager::default();
    let first = key(&mut surface, &mut manager, "Backspace", 8, false);
    let first = edit_receipt(&first).clone();
    assert_eq!(body(&surface), "abc");
    surface.input.input_method_owner = Some(INPUT);
    // The selected source is "b". Commit restores it after a provisional "X".
    set_selection(&mut surface, 1, 2);
    ime(
        &mut surface,
        &mut manager,
        UiImeInputEventKind::Preedit,
        "X",
    );
    assert_eq!(body(&surface), "aXc");
    let commit = ime(&mut surface, &mut manager, UiImeInputEventKind::Commit, "b");
    assert!(commit.widget_events.is_empty());
    assert_eq!(body(&surface), "abc");

    let undo = key(&mut surface, &mut manager, "z", 90, true);
    let undo = edit_receipt(&undo);
    assert_eq!(undo.document_id, first.document_id);
    assert_eq!(undo.previous_revision, first.revision);
    assert_eq!(undo.revision.get(), 2);
    assert_eq!(body(&surface), "abcd");
}

#[test]
fn backspace_removing_only_the_preedit_preserves_prior_undo_history() {
    let mut surface = editable_surface("abcd", 4, None);
    let mut manager = UiInputManager::default();
    let first = key(&mut surface, &mut manager, "Backspace", 8, false);
    let first = edit_receipt(&first).clone();
    assert_eq!(body(&surface), "abc");
    surface.input.input_method_owner = Some(INPUT);
    ime(
        &mut surface,
        &mut manager,
        UiImeInputEventKind::Preedit,
        "X",
    );
    assert_eq!(body(&surface), "abcX");
    let removed = key(&mut surface, &mut manager, "Backspace", 8, false);
    assert!(removed.widget_events.is_empty());
    assert_eq!(body(&surface), "abc");
    assert!(render_editable(&mut surface).composition.is_none());

    let undo = key(&mut surface, &mut manager, "z", 90, true);
    let undo = edit_receipt(&undo);
    assert_eq!(undo.document_id, first.document_id);
    assert_eq!(undo.previous_revision, first.revision);
    assert_eq!(undo.revision.get(), 2);
    assert_eq!(body(&surface), "abcd");
}

#[test]
fn backspace_leaving_changed_source_rebinds_instead_of_preserving_old_history() {
    let mut surface = editable_surface("abcd", 4, None);
    let mut manager = UiInputManager::default();
    let first = key(&mut surface, &mut manager, "Backspace", 8, false);
    let first = edit_receipt(&first).clone();
    assert_eq!(body(&surface), "abc");
    surface.input.input_method_owner = Some(INPUT);
    ime(
        &mut surface,
        &mut manager,
        UiImeInputEventKind::Preedit,
        "XY",
    );
    assert_eq!(body(&surface), "abcXY");
    let partial = key(&mut surface, &mut manager, "Backspace", 8, false);
    assert!(partial.widget_events.is_empty());
    assert_eq!(body(&surface), "abcX");
    assert!(render_editable(&mut surface).composition.is_none());

    let next = key(&mut surface, &mut manager, "Backspace", 8, false);
    let next = edit_receipt(&next);
    assert_eq!(body(&surface), "abc");
    assert_ne!(next.document_id, first.document_id);
    assert_eq!(next.previous_revision.get(), 0);
    assert_eq!(next.revision.get(), 1);
}

#[test]
fn active_selected_empty_preedit_snapshot_commits_original_source_and_undoes_it() {
    let mut surface = editable_surface("abcd", 3, Some((1, 3)));
    let mut manager = UiInputManager::default();
    surface.input.input_method_owner = Some(INPUT);
    ime(&mut surface, &mut manager, UiImeInputEventKind::Preedit, "");
    assert_eq!(body(&surface), "ad");
    let encoded = serde_json::to_string(&surface).expect("serialize selected empty preedit");
    let mut restored: UiSurface = serde_json::from_str(&encoded).expect("restore selected preedit");
    let mut manager = UiInputManager::default();

    let committed = ime(&mut restored, &mut manager, UiImeInputEventKind::Commit, "");
    let committed = edit_receipt(&committed).clone();
    assert_eq!(committed.previous_revision.get(), 0);
    assert_eq!(committed.revision.get(), 1);
    assert_eq!(body(&restored), "ad");
    let undo = key(&mut restored, &mut manager, "z", 90, true);
    assert_eq!(edit_receipt(&undo).document_id, committed.document_id);
    assert_eq!(edit_receipt(&undo).revision.get(), 2);
    assert_eq!(body(&restored), "abcd");
    assert!(render_editable(&mut restored).composition.is_none());
}

#[test]
fn zero_range_empty_preedit_survives_materialization_then_cancel_restores_normal_editing() {
    assert_zero_range_empty_preedit_lifecycle(UiImeInputEventKind::Cancel);
}

#[test]
fn zero_range_empty_preedit_survives_materialization_then_empty_commit_restores_normal_editing() {
    assert_zero_range_empty_preedit_lifecycle(UiImeInputEventKind::Commit);
}

fn assert_zero_range_empty_preedit_lifecycle(finish: UiImeInputEventKind) {
    let mut surface = editable_surface("abc", 2, None);
    let mut manager = UiInputManager::default();
    surface.input.input_method_owner = Some(INPUT);
    let preedit = ime(&mut surface, &mut manager, UiImeInputEventKind::Preedit, "");
    assert!(preedit.widget_events.is_empty());
    assert_eq!(body(&surface), "abc");
    let surrounding = preedit
        .host_requests
        .iter()
        .find_map(|host| match &host.request {
            UiDispatchHostRequestKind::InputMethod(request)
                if request.kind == UiInputMethodRequestKind::UpdateCursor =>
            {
                request.surrounding_text.as_ref()
            }
            _ => None,
        })
        .expect("active empty preedit publishes IME surrounding text");
    assert_eq!(
        surrounding.composition_range,
        Some(UiTextByteRange::new(2, 2))
    );
    let composition = render_editable(&mut surface)
        .composition
        .expect("zero-length empty preedit remains active");
    assert_eq!((composition.range.start, composition.range.end), (2, 2));
    assert_eq!(composition.restore_text.as_deref(), Some(""));

    let finished = ime(&mut surface, &mut manager, finish, "");
    assert!(finished.widget_events.is_empty());
    assert!(render_editable(&mut surface).composition.is_none());
    let edit = key(&mut surface, &mut manager, "Backspace", 8, false);
    assert_eq!(body(&surface), "ac");
    assert_eq!(edit_receipt(&edit).previous_revision.get(), 0);
    assert_eq!(edit_receipt(&edit).revision.get(), 1);
}

#[test]
fn compiled_material_text_field_starts_without_composition_and_commits_first_edit() {
    let source = r#"
[asset]
kind = "layout"
id = "runtime82.inactive_composition.catalog"
version = 3

[root]
node_id = "field"
kind = "native"
type = "TextField"
control_id = "Field"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" } }

[root.props]
value_text = "abc"
caret_offset = 3
"#;
    let document = UiAssetLoader::load_toml_str(source).expect("material field document");
    let compiled = UiDocumentCompiler::default()
        .with_component_registry(UiComponentDescriptorRegistry::material_editor_foundation())
        .compile(&document)
        .expect("compile material field with descriptor defaults");
    let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime82.inactive_composition.catalog"),
        &compiled,
    )
    .expect("build material field surface");
    let field = UiNodeId::new(1);
    surface
        .compute_layout(UiSize::new(220.0, 80.0))
        .expect("layout catalog text field");
    surface
        .focus_node(field)
        .expect("focus material text field");
    assert!(render_editable_for(&mut surface, field)
        .composition
        .is_none());

    let mut manager = UiInputManager::default();
    let edit = key(&mut surface, &mut manager, "Backspace", 8, false);
    assert_eq!(text_attribute(&surface, field, "value_text"), "ab");
    assert_eq!(edit_receipt(&edit).revision.get(), 1);
    assert!(render_editable_for(&mut surface, field)
        .composition
        .is_none());
}

#[test]
fn programmatic_text_reset_clears_preedit_without_blocking_the_next_committed_edit() {
    let mut surface = editable_surface("abcd", 3, Some((1, 3)));
    let mut manager = UiInputManager::default();
    surface.input.input_method_owner = Some(INPUT);
    ime(&mut surface, &mut manager, UiImeInputEventKind::Preedit, "");
    assert_eq!(body(&surface), "ad");
    surface
        .mutate_property(UiPropertyMutationRequest::new(
            INPUT,
            "content",
            UiValue::String("xyz".to_string()),
        ))
        .expect("programmatic text source reset");
    assert_eq!(body(&surface), "xyz");
    assert!(render_editable(&mut surface).composition.is_none());
    let edit = key(&mut surface, &mut manager, "Backspace", 8, false);
    assert_eq!(body(&surface), "yz");
    assert_eq!(edit_receipt(&edit).previous_revision.get(), 0);
    assert_eq!(edit_receipt(&edit).revision.get(), 1);
}

#[test]
fn completed_edit_stays_inactive_after_surface_snapshot_restore() {
    let mut surface = editable_surface("abc", 3, None);
    let mut manager = UiInputManager::default();
    let edit = key(&mut surface, &mut manager, "Backspace", 8, false);
    assert_eq!(edit_receipt(&edit).revision.get(), 1);
    let encoded = serde_json::to_string(&surface).expect("serialize edited surface");
    let mut restored: UiSurface = serde_json::from_str(&encoded).expect("restore edited surface");
    assert_eq!(body(&restored), "ab");
    assert!(render_editable(&mut restored).composition.is_none());
    let mut manager = UiInputManager::default();
    let edit = key(&mut restored, &mut manager, "Backspace", 8, false);
    assert_eq!(body(&restored), "a");
    assert_eq!(edit_receipt(&edit).previous_revision.get(), 0);
    assert_eq!(edit_receipt(&edit).revision.get(), 1);
}

#[test]
fn active_zero_range_empty_preedit_survives_surface_clone_and_snapshot() {
    let mut surface = editable_surface("abc", 2, None);
    let mut manager = UiInputManager::default();
    surface.input.input_method_owner = Some(INPUT);
    ime(&mut surface, &mut manager, UiImeInputEventKind::Preedit, "");
    let encoded = serde_json::to_string(&surface).expect("serialize active empty preedit");
    let restored: UiSurface = serde_json::from_str(&encoded).expect("restore active empty preedit");
    for mut snapshot in [surface.clone(), restored] {
        let editable = render_editable(&mut snapshot);
        let composition = editable
            .composition
            .expect("empty preedit remains active after snapshot");
        assert_eq!((composition.range.start, composition.range.end), (2, 2));
        assert!(composition.text.is_empty());
        assert_eq!(composition.restore_text.as_deref(), Some(""));
        assert_eq!(body(&snapshot), "abc");
        let mut manager = UiInputManager::default();
        let cancel = ime(&mut snapshot, &mut manager, UiImeInputEventKind::Cancel, "");
        assert!(cancel.widget_events.is_empty());
        assert_eq!(body(&snapshot), "abc");
        assert!(render_editable(&mut snapshot).composition.is_none());
    }
}

fn editable_surface(value: &str, caret: usize, selection: Option<(usize, usize)>) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime82.inactive_composition.regression"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 80.0))
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                ..UiStateFlags::default()
            }),
    );
    let mut attributes = [
        (
            "content".to_string(),
            toml::Value::String(value.to_string()),
        ),
        (
            "caret_offset".to_string(),
            toml::Value::Integer(caret as i64),
        ),
    ]
    .into_iter()
    .collect::<std::collections::BTreeMap<_, _>>();
    if let Some((anchor, focus)) = selection {
        attributes.insert(
            "selection_anchor".to_string(),
            toml::Value::Integer(anchor as i64),
        );
        attributes.insert(
            "selection_focus".to_string(),
            toml::Value::Integer(focus as i64),
        );
    }
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(INPUT, UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(8.0, 8.0, 180.0, 28.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    hoverable: true,
                    focusable: true,
                    ..UiStateFlags::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "InputField".to_string(),
                    attributes,
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::TextInput,
                        value_property: Some("content".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .expect("attach text input");
    surface.rebuild();
    surface.focus_node(INPUT).expect("focus text input");
    surface
}

fn set_selection(surface: &mut UiSurface, anchor: usize, focus: usize) {
    let metadata = surface
        .tree
        .node_mut(INPUT)
        .and_then(|node| node.template_metadata.as_mut())
        .expect("editable input metadata");
    metadata.attributes.insert(
        "selection_anchor".to_string(),
        toml::Value::Integer(anchor as i64),
    );
    metadata.attributes.insert(
        "selection_focus".to_string(),
        toml::Value::Integer(focus as i64),
    );
}

fn key(
    surface: &mut UiSurface,
    manager: &mut UiInputManager,
    logical_key: &str,
    key_code: u32,
    control: bool,
) -> UiInputDispatchResult {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(1), UiInputSequence::new(1));
    metadata.modifiers.control = control;
    surface
        .dispatch_input_event_with_manager(
            manager,
            UiInputEvent::Keyboard(UiKeyboardInputEvent {
                metadata,
                state: UiKeyboardInputState::Pressed,
                key_code,
                scan_code: None,
                physical_key: logical_key.to_string(),
                logical_key: logical_key.to_string(),
                text: None,
            }),
        )
        .expect("dispatch text keyboard event")
}

fn ime(
    surface: &mut UiSurface,
    manager: &mut UiInputManager,
    kind: UiImeInputEventKind,
    text: &str,
) -> UiInputDispatchResult {
    surface
        .dispatch_input_event_with_manager(
            manager,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(2),
                    UiInputSequence::new(2),
                ),
                kind,
                text: text.to_string(),
                cursor_range: None,
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            }),
        )
        .expect("dispatch IME event")
}

fn edit_receipt(result: &UiInputDispatchResult) -> &UiTextEditReceipt {
    result
        .widget_events
        .iter()
        .find_map(|event| match event {
            UiWidgetEvent::TextEditChange { receipt } => Some(receipt),
            _ => None,
        })
        .expect("committed text edit receipt")
}

fn body(surface: &UiSurface) -> &str {
    text_attribute(surface, INPUT, "content")
}

fn text_attribute<'a>(surface: &'a UiSurface, node_id: UiNodeId, key: &str) -> &'a str {
    surface
        .tree
        .node(node_id)
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(toml::Value::as_str)
        .expect("editable text attribute")
}

fn render_editable(surface: &mut UiSurface) -> UiEditableTextState {
    render_editable_for(surface, INPUT)
}

fn render_editable_for(surface: &mut UiSurface, node_id: UiNodeId) -> UiEditableTextState {
    surface.rebuild();
    surface
        .render_extract
        .list
        .commands
        .iter()
        .filter(|command| command.node_id == node_id && command.kind == UiRenderCommandKind::Text)
        .find_map(|command| {
            command
                .text_layout
                .as_ref()
                .and_then(|layout| layout.editable.clone())
        })
        .expect("render extract retains editable state")
}
