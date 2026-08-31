use super::*;

fn text_edit_receipt(
    result: &zircon_runtime_interface::ui::dispatch::UiInputDispatchResult,
) -> &zircon_runtime_interface::ui::text::UiTextEditReceipt {
    let UiWidgetEvent::TextEditChange { receipt } = &result.widget_events[0] else {
        panic!("expected retained text edit receipt");
    };
    receipt
}

#[test]
fn input_manager_rebinds_after_programmatic_text_source_change() {
    let mut surface = text_input_surface("hello", 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let mut manager = UiInputManager::default();

    let first = dispatch_key_with_manager(&mut manager, &mut surface, "Backspace", 8);
    let first_document = text_edit_receipt(&first).document_id;
    surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            UiNodeId::new(2),
            "content",
            UiValue::String("world".to_string()),
        ))
        .unwrap();

    let second = dispatch_key_with_manager(&mut manager, &mut surface, "Backspace", 8);
    let second = text_edit_receipt(&second);
    assert_ne!(first_document, second.document_id);
    assert_eq!(second.previous_revision.get(), 0);
    assert_eq!(second.revision.get(), 1);
    assert_eq!(text_attr(&surface, "content"), "worl");
}

#[test]
fn input_manager_ime_preedit_preserves_committed_document_until_commit() {
    let mut surface = text_input_surface_with_selection("abcd", 3, 1, 3);
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let mut manager = UiInputManager::default();

    let preedit = dispatch_ime_with_manager(
        &mut manager,
        &mut surface,
        UiImeInputEventKind::Preedit,
        "XY",
    );
    assert!(preedit.widget_events.is_empty());

    let commit =
        dispatch_ime_with_manager(&mut manager, &mut surface, UiImeInputEventKind::Commit, "Z");
    let receipt = text_edit_receipt(&commit);
    assert_eq!(receipt.previous_revision.get(), 0);
    assert_eq!(receipt.revision.get(), 1);
    assert_eq!(receipt.changed.old.start_byte, 1);
    assert_eq!(receipt.changed.old.end_byte, 3);
    assert_eq!(receipt.changed.new.start_byte, 1);
    assert_eq!(receipt.changed.new.end_byte, 2);
    assert_eq!(text_attr(&surface, "content"), "aZd");
}

#[test]
fn input_manager_does_not_share_documents_between_same_id_surface_instances() {
    let mut first_surface = text_input_surface("ab", 2);
    let mut second_surface = text_input_surface("ab", 2);
    first_surface.focus_node(UiNodeId::new(2)).unwrap();
    second_surface.focus_node(UiNodeId::new(2)).unwrap();
    let mut manager = UiInputManager::default();

    let first = dispatch_key_with_manager(&mut manager, &mut first_surface, "Backspace", 8);
    let first_document = text_edit_receipt(&first).document_id;
    let second = dispatch_key_with_manager(&mut manager, &mut second_surface, "Backspace", 8);
    let second_document = text_edit_receipt(&second).document_id;
    assert_ne!(first_document, second_document);

    let returned = dispatch_key_with_manager(&mut manager, &mut first_surface, "Backspace", 8);
    let returned = text_edit_receipt(&returned);
    assert_ne!(returned.document_id, second_document);
    assert_eq!(returned.previous_revision.get(), 0);
    assert_eq!(returned.revision.get(), 1);
    assert_eq!(text_attr(&first_surface, "content"), "");
}

#[test]
fn input_manager_undo_redo_preserves_document_identity_and_advances_revision() {
    let mut surface = text_input_surface("ab", 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let mut manager = UiInputManager::default();

    let changed = dispatch_key_with_manager(&mut manager, &mut surface, "Backspace", 8);
    let changed = text_edit_receipt(&changed).clone();
    assert_eq!(text_attr(&surface, "content"), "a");

    let undone = dispatch_key_with_manager_control(&mut manager, &mut surface, "z", 90);
    let undone = text_edit_receipt(&undone);
    assert_eq!(undone.document_id, changed.document_id);
    assert_eq!(undone.previous_revision.get(), 1);
    assert_eq!(undone.revision.get(), 2);
    assert_eq!(
        undone.kind,
        zircon_runtime_interface::ui::text::UiTextEditKind::Undo
    );
    assert_eq!(text_attr(&surface, "content"), "ab");
    assert_eq!(int_attr(&surface, "caret_offset"), 2);

    let redone = dispatch_key_with_manager_control_shift(&mut manager, &mut surface, "z", 90);
    let redone = text_edit_receipt(&redone);
    assert_eq!(redone.document_id, changed.document_id);
    assert_eq!(redone.previous_revision.get(), 2);
    assert_eq!(redone.revision.get(), 3);
    assert_eq!(
        redone.kind,
        zircon_runtime_interface::ui::text::UiTextEditKind::Redo
    );
    assert_eq!(text_attr(&surface, "content"), "a");
}

#[test]
fn input_manager_new_edit_after_undo_discards_redo_branch() {
    let mut surface = text_input_surface("abc", 3);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let mut manager = UiInputManager::default();

    dispatch_key_with_manager(&mut manager, &mut surface, "Backspace", 8);
    dispatch_key_with_manager_control(&mut manager, &mut surface, "z", 90);
    dispatch_key_with_manager(&mut manager, &mut surface, "Backspace", 8);
    let unavailable = dispatch_key_with_manager_control(&mut manager, &mut surface, "y", 89);

    assert_eq!(
        unavailable.reply.disposition,
        UiDispatchDisposition::Handled
    );
    assert!(unavailable.widget_events.is_empty());
    assert!(unavailable
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "text_history_unavailable"));
    assert_eq!(text_attr(&surface, "content"), "ab");
}

#[test]
fn input_manager_ime_preedit_is_one_undoable_composition_commit() {
    let mut surface = text_input_surface_with_selection("abcd", 3, 1, 3);
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let mut manager = UiInputManager::default();

    dispatch_ime_with_manager(
        &mut manager,
        &mut surface,
        UiImeInputEventKind::Preedit,
        "XY",
    );
    let unavailable = dispatch_key_with_manager_control(&mut manager, &mut surface, "z", 90);
    assert!(unavailable.widget_events.is_empty());
    assert_eq!(text_attr(&surface, "content"), "aXYd");

    surface.input.input_method_owner = Some(UiNodeId::new(2));
    dispatch_ime_with_manager(&mut manager, &mut surface, UiImeInputEventKind::Commit, "Z");
    let undone = dispatch_key_with_manager_control(&mut manager, &mut surface, "z", 90);
    let undone = text_edit_receipt(&undone);
    assert_eq!(undone.previous_revision.get(), 1);
    assert_eq!(undone.revision.get(), 2);
    assert_eq!(text_attr(&surface, "content"), "abcd");
    assert_eq!(int_attr(&surface, "selection_anchor"), 1);
    assert_eq!(int_attr(&surface, "selection_focus"), 3);
}

#[test]
fn input_manager_focus_loss_cancels_preedit_and_preserves_document_revision_chain() {
    let mut surface = text_input_surface("abc", 3);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let mut manager = UiInputManager::default();

    let first = dispatch_key_with_manager(&mut manager, &mut surface, "Backspace", 8);
    let first = text_edit_receipt(&first).clone();
    dispatch_ime_with_manager(
        &mut manager,
        &mut surface,
        UiImeInputEventKind::Preedit,
        "X",
    );
    assert_eq!(text_attr(&surface, "content"), "abX");

    surface.clear_focus();
    assert_eq!(text_attr(&surface, "content"), "ab");
    assert_eq!(text_attr(&surface, "composition_text"), "");
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let unavailable = dispatch_key_with_manager_control(&mut manager, &mut surface, "z", 90);
    assert_eq!(
        unavailable.reply.disposition,
        UiDispatchDisposition::Handled
    );
    assert!(unavailable.widget_events.is_empty());
    assert!(unavailable
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "text_history_unavailable"));

    let second = dispatch_key_with_manager(&mut manager, &mut surface, "Backspace", 8);
    let second = text_edit_receipt(&second);
    assert_eq!(second.document_id, first.document_id);
    assert_eq!(second.previous_revision.get(), 1);
    assert_eq!(second.revision.get(), 2);
    assert_eq!(text_attr(&surface, "content"), "a");
}

#[test]
fn input_manager_secure_text_edits_form_a_history_barrier() {
    let mut surface =
        text_input_surface_with_attributes("secret", 6, [("secure", toml::Value::Boolean(true))]);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let mut manager = UiInputManager::default();

    dispatch_key_with_manager(&mut manager, &mut surface, "Backspace", 8);
    let unavailable = dispatch_key_with_manager_control(&mut manager, &mut surface, "z", 90);

    assert_eq!(
        unavailable.reply.disposition,
        UiDispatchDisposition::Handled
    );
    assert!(unavailable.widget_events.is_empty());
    assert!(unavailable.diagnostics.secure_text_redacted);
    assert_eq!(text_attr(&surface, "content"), "secre");
}
