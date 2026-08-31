use super::*;

#[test]
fn text_input_keyboard_control_c_requests_clipboard_write_for_selection() {
    let mut surface = text_input_surface_with_selection("alpha beta", 5, 0, 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control(&mut surface, "c", 67);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.clipboard_copy")
    );
    assert_eq!(text_attr(&surface, "content"), "alpha beta");
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
    assert_clipboard_request(&result, UiClipboardRequestKind::WriteText, Some("alpha"));
}

#[test]
fn text_input_keyboard_control_x_cuts_selection_and_requests_clipboard_write() {
    let mut surface = text_input_surface_with_selection("alpha beta", 5, 0, 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control(&mut surface, "x", 88);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.clipboard_cut")
    );
    assert_eq!(text_attr(&surface, "content"), "alpha beta");
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
    let request =
        assert_clipboard_request(&result, UiClipboardRequestKind::WriteText, Some("alpha"));

    let completion = dispatch_clipboard_completion(
        &mut surface,
        &request,
        UiClipboardTransferOutcome::WriteText,
    );

    assert_eq!(text_attr(&surface, "content"), " beta");
    assert_eq!(int_attr(&surface, "caret_offset"), 0);
    assert_eq!(int_attr(&surface, "selection_anchor"), 0);
    assert_eq!(int_attr(&surface, "selection_focus"), 0);
    assert_eq!(
        completion.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String(" beta".to_string()),
        }
    );
    assert_widget_binding_report(&completion.binding_reports);
}

#[test]
fn text_input_keyboard_control_v_requests_clipboard_read() {
    let mut surface = text_input_surface("alpha", 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control(&mut surface, "v", 86);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.clipboard_paste")
    );
    assert_eq!(text_attr(&surface, "content"), "alpha");
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
    assert_clipboard_request(&result, UiClipboardRequestKind::ReadText, None);
}

#[test]
fn text_input_keyboard_copy_key_requests_clipboard_write_for_selection() {
    let mut surface = text_input_surface_with_selection("alpha beta", 5, 0, 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Copy", 0);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.clipboard_copy")
    );
    assert_eq!(text_attr(&surface, "content"), "alpha beta");
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
    assert_clipboard_request(&result, UiClipboardRequestKind::WriteText, Some("alpha"));
}

#[test]
fn text_input_keyboard_cut_key_cuts_selection_and_requests_clipboard_write() {
    let mut surface = text_input_surface_with_selection("alpha beta", 5, 0, 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Cut", 0);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.clipboard_cut")
    );
    assert_eq!(text_attr(&surface, "content"), "alpha beta");
    let request =
        assert_clipboard_request(&result, UiClipboardRequestKind::WriteText, Some("alpha"));

    let completion = dispatch_clipboard_completion(
        &mut surface,
        &request,
        UiClipboardTransferOutcome::WriteText,
    );

    assert_eq!(text_attr(&surface, "content"), " beta");
    assert_eq!(int_attr(&surface, "caret_offset"), 0);
    assert_eq!(
        completion.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String(" beta".to_string()),
        }
    );
    assert_widget_binding_report(&completion.binding_reports);
}

#[test]
fn text_input_keyboard_paste_key_requests_clipboard_read() {
    let mut surface = text_input_surface("alpha", 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Paste", 0);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.clipboard_paste")
    );
    assert_eq!(text_attr(&surface, "content"), "alpha");
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
    assert_clipboard_request(&result, UiClipboardRequestKind::ReadText, None);
}

#[test]
fn text_input_keyboard_shift_delete_cuts_selection_and_requests_clipboard_write() {
    let mut surface = text_input_surface_with_selection("alpha beta", 5, 0, 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_shift(&mut surface, "Delete", 46);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.clipboard_cut")
    );
    assert_eq!(text_attr(&surface, "content"), "alpha beta");
    let request =
        assert_clipboard_request(&result, UiClipboardRequestKind::WriteText, Some("alpha"));

    let completion = dispatch_clipboard_completion(
        &mut surface,
        &request,
        UiClipboardTransferOutcome::WriteText,
    );

    assert_eq!(text_attr(&surface, "content"), " beta");
    assert_eq!(int_attr(&surface, "caret_offset"), 0);
    assert_eq!(
        completion.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String(" beta".to_string()),
        }
    );
    assert_widget_binding_report(&completion.binding_reports);
}

#[test]
fn text_input_cut_failure_and_stale_completion_preserve_document() {
    let mut failed = text_input_surface_with_selection("alpha beta", 5, 0, 5);
    failed.focus_node(UiNodeId::new(2)).unwrap();
    let failed_request = assert_clipboard_request(
        &dispatch_key_with_control(&mut failed, "x", 88),
        UiClipboardRequestKind::WriteText,
        Some("alpha"),
    );

    let failed_result = dispatch_clipboard_completion(
        &mut failed,
        &failed_request,
        UiClipboardTransferOutcome::Failed {
            reason: zircon_runtime_interface::ui::dispatch::UiClipboardTransferFailure::Unavailable,
        },
    );

    assert_eq!(text_attr(&failed, "content"), "alpha beta");
    assert!(failed_result.component_events.is_empty());

    let mut stale = text_input_surface_with_selection("alpha beta", 5, 0, 5);
    stale.focus_node(UiNodeId::new(2)).unwrap();
    let stale_request = assert_clipboard_request(
        &dispatch_key_with_control(&mut stale, "x", 88),
        UiClipboardRequestKind::WriteText,
        Some("alpha"),
    );
    dispatch_text(&mut stale, "Z");

    let stale_result = dispatch_clipboard_completion(
        &mut stale,
        &stale_request,
        UiClipboardTransferOutcome::WriteText,
    );

    assert_eq!(text_attr(&stale, "content"), "Z beta");
    assert_eq!(
        stale_result
            .diagnostics
            .clipboard_transfer
            .as_ref()
            .map(|receipt| receipt.status),
        Some(zircon_runtime_interface::ui::dispatch::UiClipboardTransferStatus::RejectedStale)
    );
}

#[test]
fn text_input_paste_completion_applies_constraints_once() {
    let mut surface = text_input_surface_with_attributes(
        "alpha",
        5,
        [("multiline", toml::Value::Boolean(false))],
    );
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let request = assert_clipboard_request(
        &dispatch_key_with_control(&mut surface, "v", 86),
        UiClipboardRequestKind::ReadText,
        None,
    );

    let completion = dispatch_clipboard_completion(
        &mut surface,
        &request,
        UiClipboardTransferOutcome::ReadText {
            text: "\r\nbeta".to_string(),
        },
    );

    assert_eq!(text_attr(&surface, "content"), "alphabeta");
    assert_eq!(
        completion
            .diagnostics
            .text_constraint
            .map(|receipt| receipt.removed_hard_line_count),
        Some(1)
    );
    assert_eq!(
        completion
            .diagnostics
            .clipboard_transfer
            .as_ref()
            .map(|receipt| receipt.status),
        Some(zircon_runtime_interface::ui::dispatch::UiClipboardTransferStatus::Applied)
    );

    let duplicate = dispatch_clipboard_completion(
        &mut surface,
        &request,
        UiClipboardTransferOutcome::ReadText {
            text: "ignored".to_string(),
        },
    );
    assert_eq!(text_attr(&surface, "content"), "alphabeta");
    assert_eq!(
        duplicate
            .diagnostics
            .clipboard_transfer
            .as_ref()
            .map(|receipt| receipt.status),
        Some(zircon_runtime_interface::ui::dispatch::UiClipboardTransferStatus::RejectedUnknown)
    );
}

#[test]
fn clipboard_transfer_does_not_cross_surface_clone_or_focus_epoch() {
    let mut surface = text_input_surface("alpha", 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let request = assert_clipboard_request(
        &dispatch_key_with_control(&mut surface, "v", 86),
        UiClipboardRequestKind::ReadText,
        None,
    );

    let mut cloned = surface.clone();
    let clone_result = dispatch_clipboard_completion(
        &mut cloned,
        &request,
        UiClipboardTransferOutcome::ReadText {
            text: "clone".to_string(),
        },
    );
    assert_eq!(text_attr(&cloned, "content"), "alpha");
    assert_eq!(
        clone_result
            .diagnostics
            .clipboard_transfer
            .as_ref()
            .map(|receipt| receipt.status),
        Some(zircon_runtime_interface::ui::dispatch::UiClipboardTransferStatus::RejectedUnknown)
    );

    surface.clear_focus();
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let focus_result = dispatch_clipboard_completion(
        &mut surface,
        &request,
        UiClipboardTransferOutcome::ReadText {
            text: "focus".to_string(),
        },
    );
    assert_eq!(text_attr(&surface, "content"), "alpha");
    assert_eq!(
        focus_result
            .diagnostics
            .clipboard_transfer
            .as_ref()
            .map(|receipt| receipt.status),
        Some(zircon_runtime_interface::ui::dispatch::UiClipboardTransferStatus::RejectedStale)
    );
}

#[test]
fn text_input_keyboard_enter_inserts_newline_when_multiline() {
    let mut surface = text_input_surface("alpha", 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Enter", 13);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.text_edit")
    );
    assert_eq!(text_attr(&surface, "content"), "alpha\n");
    assert_eq!(int_attr(&surface, "caret_offset"), 6);
    assert_eq!(int_attr(&surface, "selection_anchor"), 6);
    assert_eq!(int_attr(&surface, "selection_focus"), 6);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("alpha\n".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_enter_replaces_selection_with_newline() {
    let mut surface = text_input_surface_with_selection("alpha beta", 6, 5, 6);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Enter", 13);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "alpha\nbeta");
    assert_eq!(int_attr(&surface, "caret_offset"), 6);
    assert_eq!(int_attr(&surface, "selection_anchor"), 6);
    assert_eq!(int_attr(&surface, "selection_focus"), 6);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("alpha\nbeta".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_enter_submits_explicit_single_line() {
    let mut surface = text_input_surface_with_attributes(
        "alpha",
        5,
        [("multiline", toml::Value::Boolean(false))],
    );
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Enter", 13);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.submit")
    );
    assert_eq!(text_attr(&surface, "content"), "alpha");
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert!(result.diagnostics.text_constraint.is_none());
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::Commit {
            property: "content".to_string(),
            value: UiValue::String("alpha".to_string()),
        }
    );
    assert!(result.binding_reports.is_empty());
}

#[test]
fn text_input_keyboard_repeated_enter_does_not_resubmit_single_line() {
    let mut surface = text_input_surface_with_attributes(
        "alpha",
        5,
        [("multiline", toml::Value::Boolean(false))],
    );
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_state(&mut surface, "Enter", 13, UiKeyboardInputState::Repeated);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.submit")
    );
    assert_eq!(text_attr(&surface, "content"), "alpha");
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
}
