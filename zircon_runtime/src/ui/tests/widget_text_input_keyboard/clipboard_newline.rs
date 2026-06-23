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
    assert_eq!(text_attr(&surface, "content"), " beta");
    assert_eq!(int_attr(&surface, "caret_offset"), 0);
    assert_eq!(int_attr(&surface, "selection_anchor"), 0);
    assert_eq!(int_attr(&surface, "selection_focus"), 0);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String(" beta".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
    assert_clipboard_request(&result, UiClipboardRequestKind::WriteText, Some("alpha"));
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
    assert_eq!(text_attr(&surface, "content"), " beta");
    assert_eq!(int_attr(&surface, "caret_offset"), 0);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String(" beta".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
    assert_clipboard_request(&result, UiClipboardRequestKind::WriteText, Some("alpha"));
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
    assert_eq!(text_attr(&surface, "content"), " beta");
    assert_eq!(int_attr(&surface, "caret_offset"), 0);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String(" beta".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
    assert_clipboard_request(&result, UiClipboardRequestKind::WriteText, Some("alpha"));
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
fn text_input_keyboard_enter_respects_explicit_single_line() {
    let mut surface = text_input_surface_with_attributes(
        "alpha",
        5,
        [("multiline", toml::Value::Boolean(false))],
    );
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Enter", 13);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(result.diagnostics.handled_phase.as_deref(), None);
    assert_eq!(text_attr(&surface, "content"), "alpha");
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
}
