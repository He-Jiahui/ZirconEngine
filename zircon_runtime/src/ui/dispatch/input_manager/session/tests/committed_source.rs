use zircon_runtime_interface::ui::{
    dispatch::{
        UiImeInputEvent, UiImeInputEventKind, UiInputDispatchResult, UiInputEvent,
        UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent,
        UiKeyboardInputState,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::{UiEditableTextState, UiTextComposition, UiTextRange},
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

use super::super::limits::mvp_text_document_store_limits;
use super::{UiTextDocumentSession, UiTextDocumentSessionError};
use crate::text::document::{
    TextDocumentAdmissionFailure, TextDocumentStore, TextDocumentStoreError,
};
use crate::ui::{
    dispatch::{UiInputManager, UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};

#[test]
fn active_preedit_snapshot_opens_the_committed_source_and_keeps_its_binding() {
    let tree_id = UiTreeId::new("runtime82.ime-committed-source");
    let owner = UiNodeId::new(2);
    let mut session = UiTextDocumentSession::default();
    let selected_empty_preedit = active_preedit("ad", 1..1, "", "bc");

    session.synchronize_editable_source(&tree_id, owner, 0, &selected_empty_preedit);
    let key = session
        .document_key(&tree_id, owner, 0)
        .expect("document binding");
    assert_eq!(
        session
            .store
            .source_range(key.document_id, key.revision, 0..4)
            .expect("committed source"),
        "abcd"
    );
    assert_eq!(
        session.committed_source_matches(&tree_id, owner, 0, "abcd"),
        Ok(true)
    );
    assert_eq!(
        session.committed_source_matches(&tree_id, owner, 0, "ad"),
        Ok(false)
    );
    assert_eq!(
        session.committed_source_matches(&tree_id, owner, 1, "abcd"),
        Err(UiTextDocumentSessionError::SourceNotSynchronized)
    );

    // Further preedit updates change the visible text without replacing the source document.
    let updated_preedit = active_preedit("aXYd", 1..3, "XY", "bc");
    session.synchronize_editable_source(&tree_id, owner, 0, &updated_preedit);
    assert_eq!(
        session
            .document_key(&tree_id, owner, 0)
            .expect("same binding"),
        key
    );
    assert_eq!(
        session
            .store
            .source_range(key.document_id, key.revision, 0..4)
            .expect("unchanged committed source"),
        "abcd"
    );
}

#[test]
fn zero_range_empty_preedit_snapshot_has_a_real_committed_source() {
    let tree_id = UiTreeId::new("runtime82.ime-zero-range");
    let owner = UiNodeId::new(3);
    let mut session = UiTextDocumentSession::default();
    let active = active_preedit("abc", 2..2, "", "");

    session.synchronize_editable_source(&tree_id, owner, 0, &active);
    let key = session
        .document_key(&tree_id, owner, 0)
        .expect("document binding");
    assert_eq!(
        session
            .store
            .source_range(key.document_id, key.revision, 0..3)
            .expect("committed source"),
        "abc"
    );
}

#[test]
fn active_unicode_snapshot_restores_source_bytes_before_opening_the_document() {
    let tree_id = UiTreeId::new("runtime82.ime-unicode-source");
    let owner = UiNodeId::new(4);
    let mut session = UiTextDocumentSession::default();
    let active = active_preedit("a🙂d", 1..5, "🙂", "é界");
    let expected = "aé界d";

    session.synchronize_editable_source(&tree_id, owner, 0, &active);
    let key = session
        .document_key(&tree_id, owner, 0)
        .expect("document binding");
    assert_eq!(
        session
            .store
            .source_range(key.document_id, key.revision, 0..expected.len())
            .expect("Unicode committed source"),
        expected
    );
}

#[test]
fn changed_source_epoch_rebinds_programmatic_text_instead_of_reusing_preedit() {
    let tree_id = UiTreeId::new("runtime82.ime-reset-source");
    let owner = UiNodeId::new(5);
    let mut session = UiTextDocumentSession::default();
    session.synchronize_editable_source(&tree_id, owner, 0, &active_preedit("ad", 1..1, "", "bc"));
    let before = session
        .document_key(&tree_id, owner, 0)
        .expect("preedit binding");
    let reset = UiEditableTextState {
        text: "xyz".to_string(),
        ..Default::default()
    };

    session.synchronize_editable_source(&tree_id, owner, 1, &reset);
    let after = session
        .document_key(&tree_id, owner, 1)
        .expect("reset binding");
    assert_ne!(after.document_id, before.document_id);
    assert_eq!(after.revision.get(), 0);
    assert_eq!(session.store.report().document_count, 1);
    assert_eq!(
        session
            .store
            .source_range(after.document_id, after.revision, 0..3)
            .expect("new committed source"),
        "xyz"
    );
}

#[test]
fn malformed_snapshot_preedit_range_is_rejected_without_opening_visible_text() {
    let tree_id = UiTreeId::new("runtime82.ime-invalid-source");
    let owner = UiNodeId::new(6);
    let mut session = UiTextDocumentSession::default();
    let active = active_preedit("a🙂d", 2..5, "🙂", "bc");

    session.synchronize_editable_source(&tree_id, owner, 0, &active);
    assert_eq!(
        session.document_key(&tree_id, owner, 0),
        Err(UiTextDocumentSessionError::InvalidEditIntent)
    );
    assert_eq!(session.store.report().document_count, 0);
}

#[test]
fn small_document_limit_records_the_same_missing_binding_as_admission_failure() {
    let tree_id = UiTreeId::new("runtime82.ime-admission-source");
    let owner = UiNodeId::new(7);
    let mut limits = mvp_text_document_store_limits();
    limits.max_document_bytes = 2;
    let mut session = UiTextDocumentSession::default();
    session.store = TextDocumentStore::with_limits(limits);

    session.synchronize_source(&tree_id, owner, 0, "abc");
    let expected = UiTextDocumentSessionError::Store(TextDocumentStoreError::AdmissionDenied(
        TextDocumentAdmissionFailure::DocumentBytes,
    ));
    assert_eq!(session.document_key(&tree_id, owner, 0), Err(expected));
    assert_eq!(
        session.committed_source_matches(&tree_id, owner, 0, "abc"),
        Err(expected)
    );
    assert_eq!(session.store.report().document_count, 0);
}

#[test]
fn cancel_restores_preedit_after_small_limit_document_admission_failure() {
    assert_admission_failure_finish(ime_event(UiImeInputEventKind::Cancel, ""), true);
}

#[test]
fn physical_escape_restores_preedit_after_small_limit_document_admission_failure() {
    assert_admission_failure_finish(
        UiInputEvent::Keyboard(UiKeyboardInputEvent {
            metadata: input_metadata(),
            state: UiKeyboardInputState::Pressed,
            key_code: 27,
            scan_code: None,
            physical_key: "Unidentified".to_string(),
            logical_key: "Unidentified".to_string(),
            text: None,
        }),
        true,
    );
}

#[test]
fn unbound_no_intent_commit_finishes_with_source_reset_and_diagnostic() {
    assert_admission_failure_finish(ime_event(UiImeInputEventKind::Commit, ""), false);
}

#[test]
fn physical_escape_without_a_manager_preserves_the_restored_source_epoch() {
    let owner = UiNodeId::new(9);
    let mut surface = admission_failure_surface(owner);
    let pointer = UiPointerDispatcher::default();
    let navigation = UiNavigationDispatcher::default();
    let epoch = surface.input.text_document_epoch(owner);
    let preedit = surface
        .dispatch_input_event(
            &pointer,
            &navigation,
            ime_event(UiImeInputEventKind::Preedit, "X"),
        )
        .expect("dispatch managerless preedit");
    assert_no_rejected_mutation(&preedit);
    assert_eq!(visible_text(&surface, owner), "abcX");

    let cancelled = surface
        .dispatch_input_event(
            &pointer,
            &navigation,
            UiInputEvent::Keyboard(UiKeyboardInputEvent {
                metadata: input_metadata(),
                state: UiKeyboardInputState::Pressed,
                key_code: 27,
                scan_code: None,
                physical_key: "Unidentified".to_string(),
                logical_key: "Unidentified".to_string(),
                text: None,
            }),
        )
        .expect("dispatch managerless physical Escape");
    assert_no_rejected_mutation(&cancelled);
    assert_eq!(visible_text(&surface, owner), "abc");
    assert_eq!(surface.input.text_document_epoch(owner), epoch);
}

fn assert_admission_failure_finish(event: UiInputEvent, explicit_cancel: bool) {
    let owner = UiNodeId::new(8);
    let mut surface = admission_failure_surface(owner);
    let mut manager = UiInputManager::default();
    let mut limits = mvp_text_document_store_limits();
    limits.max_document_bytes = 2;
    manager.text_documents.store = TextDocumentStore::with_limits(limits);
    let epoch = surface
        .input
        .text_document_epoch(owner)
        .expect("source epoch");
    let preedit = manager
        .dispatch_input_event(&mut surface, ime_event(UiImeInputEventKind::Preedit, "X"))
        .expect("dispatch provisional edit despite source admission failure");
    assert_no_rejected_mutation(&preedit);
    assert_eq!(visible_text(&surface, owner), "abcX");
    assert_eq!(surface.input.text_document_epoch(owner), Some(epoch));
    assert_eq!(
        manager
            .text_documents
            .document_key(&surface.tree.tree_id, owner, epoch),
        Err(UiTextDocumentSessionError::Store(
            TextDocumentStoreError::AdmissionDenied(TextDocumentAdmissionFailure::DocumentBytes)
        ))
    );

    let ime_cancel =
        matches!(&event, UiInputEvent::Ime(ime) if ime.kind == UiImeInputEventKind::Cancel);
    let finished = manager
        .dispatch_input_event(&mut surface, event)
        .expect("finish preedit");
    assert_no_rejected_mutation(&finished);
    assert_eq!(visible_text(&surface, owner), "abc");
    assert_eq!(
        surface.input.text_document_epoch(owner),
        Some(if explicit_cancel { epoch } else { epoch + 1 })
    );
    let attributes = &surface
        .tree
        .node(owner)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes;
    assert_eq!(attributes["composition_start"].as_integer(), Some(-1));
    assert_eq!(attributes["composition_end"].as_integer(), Some(-1));
    if ime_cancel {
        assert_eq!(surface.input.input_method_owner, None);
    }
    if !explicit_cancel {
        assert!(finished
            .diagnostics
            .notes
            .iter()
            .any(|note| { note == "text_composition_source_reset:admission_document_bytes" }));
    }
}

fn admission_failure_surface(owner: UiNodeId) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime82.ime-admission-finish"));
    surface.tree.insert_root(
        UiTreeNode::new(owner, UiNodePath::new("input"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 28.0))
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(UiStateFlags {
                visible: true,
                enabled: true,
                focusable: true,
                ..Default::default()
            })
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "InputField".to_string(),
                attributes: [
                    (
                        "content".to_string(),
                        toml::Value::String("abc".to_string()),
                    ),
                    ("caret_offset".to_string(), toml::Value::Integer(3)),
                ]
                .into_iter()
                .collect(),
                widget: UiWidgetContract {
                    behavior: UiWidgetBehavior::TextInput,
                    value_property: Some("content".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }),
    );
    surface.rebuild();
    surface.focus_node(owner).expect("focus input");
    surface.input.input_method_owner = Some(owner);
    surface
}

fn ime_event(kind: UiImeInputEventKind, text: &str) -> UiInputEvent {
    UiInputEvent::Ime(UiImeInputEvent {
        metadata: input_metadata(),
        kind,
        text: text.to_string(),
        cursor_range: None,
        preedit_clauses: Vec::new(),
        delete_surrounding: None,
    })
}

fn input_metadata() -> UiInputEventMetadata {
    UiInputEventMetadata::new(UiInputTimestamp::from_micros(1), UiInputSequence::new(1))
}

fn visible_text(surface: &UiSurface, owner: UiNodeId) -> &str {
    surface
        .tree
        .node(owner)
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get("content"))
        .and_then(toml::Value::as_str)
        .expect("visible input text")
}

fn assert_no_rejected_mutation(result: &UiInputDispatchResult) {
    assert!(result.widget_events.is_empty());
    assert!(!result
        .diagnostics
        .notes
        .iter()
        .any(|note| { note.starts_with("text_state_transaction_rejected:") }));
}

fn active_preedit(
    visible: &str,
    range: std::ops::Range<usize>,
    text: &str,
    restore_text: &str,
) -> UiEditableTextState {
    UiEditableTextState {
        text: visible.to_string(),
        composition: Some(UiTextComposition {
            range: UiTextRange {
                start: range.start,
                end: range.end,
            },
            text: text.to_string(),
            restore_text: Some(restore_text.to_string()),
            preedit_clauses: Vec::new(),
        }),
        ..Default::default()
    }
}
