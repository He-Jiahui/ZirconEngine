use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiClipboardRequestKind, UiDispatchDisposition, UiDispatchHostRequestKind, UiImeInputEvent,
        UiImeInputEventKind, UiInputEvent, UiInputEventMetadata, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputEvent, UiKeyboardInputState, UiTextByteRange, UiTextInputEvent,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

#[test]
fn text_input_keyboard_backspace_uses_widget_value_property() {
    let mut surface = text_input_surface("hello", 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Backspace", 8);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.text_edit")
    );
    assert_eq!(text_attr(&surface, "content"), "hell");
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("hell".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_arrow_left_moves_caret_without_value_event() {
    let mut surface = text_input_surface("he", 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "ArrowLeft", 37);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "he");
    assert_eq!(int_attr(&surface, "caret_offset"), 1);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_read_only_arrow_left_still_moves_caret() {
    let mut surface =
        text_input_surface_with_attributes("hello", 5, [("read_only", toml::Value::Boolean(true))]);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "ArrowLeft", 37);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "hello");
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_read_only_backspace_does_not_mutate_value() {
    let mut surface =
        text_input_surface_with_attributes("hello", 5, [("read_only", toml::Value::Boolean(true))]);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Backspace", 8);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "hello");
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert!(result.component_events.is_empty());
}

#[test]
fn text_input_keyboard_arrow_left_moves_by_grapheme_cluster() {
    let value = format!("a{}", combining_acute_cluster());
    let mut surface = text_input_surface(&value, value.len());
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "ArrowLeft", 37);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 1);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_backspace_deletes_previous_grapheme_cluster() {
    let value = format!("a{}", combining_acute_cluster());
    let mut surface = text_input_surface(&value, value.len());
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Backspace", 8);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "a");
    assert_eq!(int_attr(&surface, "caret_offset"), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("a".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_delete_removes_next_grapheme_cluster() {
    let value = format!("a{}b", combining_acute_cluster());
    let mut surface = text_input_surface(&value, 1);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Delete", 46);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "ab");
    assert_eq!(int_attr(&surface, "caret_offset"), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("ab".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_shift_arrow_left_extends_selection_without_value_event() {
    let mut surface = text_input_surface("hello", 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_shift(&mut surface, "ArrowLeft", 37);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "hello");
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
    assert_eq!(int_attr(&surface, "selection_anchor"), 5);
    assert_eq!(int_attr(&surface, "selection_focus"), 4);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_shift_home_extends_selection_to_start() {
    let mut surface = text_input_surface("hello", 4);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_shift(&mut surface, "Home", 36);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "hello");
    assert_eq!(int_attr(&surface, "caret_offset"), 0);
    assert_eq!(int_attr(&surface, "selection_anchor"), 4);
    assert_eq!(int_attr(&surface, "selection_focus"), 0);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_home_moves_to_current_line_start() {
    let value = "one\ntwo three\nfour";
    let mut surface = text_input_surface(value, 10);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Home", 36);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
    assert_eq!(int_attr(&surface, "selection_anchor"), 4);
    assert_eq!(int_attr(&surface, "selection_focus"), 4);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_end_moves_to_current_line_end() {
    let value = "one\ntwo three\nfour";
    let mut surface = text_input_surface(value, 10);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "End", 35);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 13);
    assert_eq!(int_attr(&surface, "selection_anchor"), 13);
    assert_eq!(int_attr(&surface, "selection_focus"), 13);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_shift_end_extends_selection_to_current_line_end() {
    let value = "one\ntwo three\nfour";
    let mut surface = text_input_surface(value, 8);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_shift(&mut surface, "End", 35);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 13);
    assert_eq!(int_attr(&surface, "selection_anchor"), 8);
    assert_eq!(int_attr(&surface, "selection_focus"), 13);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_control_end_moves_to_document_end() {
    let value = "one\ntwo three\nfour";
    let mut surface = text_input_surface(value, 10);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control(&mut surface, "End", 35);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), value.len() as i64);
    assert_eq!(int_attr(&surface, "selection_anchor"), value.len() as i64);
    assert_eq!(int_attr(&surface, "selection_focus"), value.len() as i64);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_home_moves_to_current_crlf_line_start() {
    let value = "one\r\ntwo\r\nthree";
    let mut surface = text_input_surface(value, 7);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Home", 36);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert_eq!(int_attr(&surface, "selection_anchor"), 5);
    assert_eq!(int_attr(&surface, "selection_focus"), 5);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_end_stops_before_crlf_separator() {
    let value = "one\r\ntwo\r\nthree";
    let mut surface = text_input_surface(value, 6);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "End", 35);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 8);
    assert_eq!(int_attr(&surface, "selection_anchor"), 8);
    assert_eq!(int_attr(&surface, "selection_focus"), 8);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_arrow_up_moves_to_previous_line_same_column() {
    let value = "abcd\nwxyz\n12";
    let mut surface = text_input_surface(value, 7);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "ArrowUp", 38);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 2);
    assert_eq!(int_attr(&surface, "selection_anchor"), 2);
    assert_eq!(int_attr(&surface, "selection_focus"), 2);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_arrow_up_on_first_line_moves_to_document_start() {
    let value = "abcd\nwxyz";
    let mut surface = text_input_surface(value, 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "ArrowUp", 38);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 0);
    assert_eq!(int_attr(&surface, "selection_anchor"), 0);
    assert_eq!(int_attr(&surface, "selection_focus"), 0);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_arrow_down_moves_to_next_line_same_column() {
    let value = "abcd\nwxyz\n12";
    let mut surface = text_input_surface(value, 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "ArrowDown", 40);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 7);
    assert_eq!(int_attr(&surface, "selection_anchor"), 7);
    assert_eq!(int_attr(&surface, "selection_focus"), 7);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_arrow_down_on_last_line_moves_to_document_end() {
    let value = "abcd\nwxyz";
    let mut surface = text_input_surface(value, 7);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "ArrowDown", 40);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), value.len() as i64);
    assert_eq!(int_attr(&surface, "selection_anchor"), value.len() as i64);
    assert_eq!(int_attr(&surface, "selection_focus"), value.len() as i64);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_arrow_down_clamps_to_shorter_line() {
    let value = "abcd\nxy";
    let mut surface = text_input_surface(value, 3);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "ArrowDown", 40);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 7);
    assert_eq!(int_attr(&surface, "selection_anchor"), 7);
    assert_eq!(int_attr(&surface, "selection_focus"), 7);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_arrow_down_handles_crlf_boundaries() {
    let value = "abcd\r\nwxyz\r\n12";
    let mut surface = text_input_surface(value, 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "ArrowDown", 40);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 8);
    assert_eq!(int_attr(&surface, "selection_anchor"), 8);
    assert_eq!(int_attr(&surface, "selection_focus"), 8);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_arrow_up_handles_crlf_boundaries() {
    let value = "abcd\r\nwxyz\r\n12";
    let mut surface = text_input_surface(value, 8);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "ArrowUp", 38);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 2);
    assert_eq!(int_attr(&surface, "selection_anchor"), 2);
    assert_eq!(int_attr(&surface, "selection_focus"), 2);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_shift_arrow_down_extends_selection_to_next_line() {
    let value = "abcd\nwxyz";
    let mut surface = text_input_surface(value, 1);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_shift(&mut surface, "ArrowDown", 40);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 6);
    assert_eq!(int_attr(&surface, "selection_anchor"), 1);
    assert_eq!(int_attr(&surface, "selection_focus"), 6);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_arrow_down_uses_grapheme_column() {
    let cluster = combining_acute_cluster();
    let value = format!("{cluster}x\nab");
    let mut surface = text_input_surface(&value, cluster.len());
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "ArrowDown", 40);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 6);
    assert_eq!(int_attr(&surface, "selection_anchor"), 6);
    assert_eq!(int_attr(&surface, "selection_focus"), 6);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_control_arrow_up_moves_to_document_start() {
    let value = "abcd\nwxyz\n12";
    let mut surface = text_input_surface(value, 7);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control(&mut surface, "ArrowUp", 38);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 0);
    assert_eq!(int_attr(&surface, "selection_anchor"), 0);
    assert_eq!(int_attr(&surface, "selection_focus"), 0);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_control_shift_arrow_down_extends_to_document_end() {
    let value = "abcd\nwxyz\n12";
    let mut surface = text_input_surface(value, 1);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control_shift(&mut surface, "ArrowDown", 40);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), value.len() as i64);
    assert_eq!(int_attr(&surface, "selection_anchor"), 1);
    assert_eq!(int_attr(&surface, "selection_focus"), value.len() as i64);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_control_arrow_right_moves_to_word_end() {
    let mut surface = text_input_surface("alpha beta", 0);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control(&mut surface, "ArrowRight", 39);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "alpha beta");
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_control_arrow_left_moves_to_word_start() {
    let value = "alpha beta";
    let mut surface = text_input_surface(value, value.len());
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control(&mut surface, "ArrowLeft", 37);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 6);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_control_shift_arrow_right_extends_word_selection() {
    let mut surface = text_input_surface("alpha beta", 0);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control_shift(&mut surface, "ArrowRight", 39);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "alpha beta");
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert_eq!(int_attr(&surface, "selection_anchor"), 0);
    assert_eq!(int_attr(&surface, "selection_focus"), 5);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_control_backspace_deletes_previous_word() {
    let value = "alpha beta";
    let mut surface = text_input_surface(value, value.len());
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control(&mut surface, "Backspace", 8);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "alpha ");
    assert_eq!(int_attr(&surface, "caret_offset"), 6);
    assert_eq!(int_attr(&surface, "selection_anchor"), 6);
    assert_eq!(int_attr(&surface, "selection_focus"), 6);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("alpha ".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_control_delete_deletes_next_word() {
    let mut surface = text_input_surface("alpha beta", 6);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control(&mut surface, "Delete", 46);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "alpha ");
    assert_eq!(int_attr(&surface, "caret_offset"), 6);
    assert_eq!(int_attr(&surface, "selection_anchor"), 6);
    assert_eq!(int_attr(&surface, "selection_focus"), 6);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("alpha ".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_control_a_selects_all_text() {
    let value = "alpha beta";
    let mut surface = text_input_surface(value, 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control(&mut surface, "a", 65);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), value.len() as i64);
    assert_eq!(int_attr(&surface, "selection_anchor"), 0);
    assert_eq!(int_attr(&surface, "selection_focus"), value.len() as i64);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

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

#[test]
fn text_input_text_event_replaces_active_selection() {
    let mut surface = text_input_surface_with_selection("alpha beta", 5, 0, 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_text(&mut surface, "omega");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("text.edit")
    );
    assert_eq!(text_attr(&surface, "content"), "omega beta");
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert_eq!(int_attr(&surface, "selection_anchor"), 5);
    assert_eq!(int_attr(&surface, "selection_focus"), 5);
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("omega beta".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_backspace_deletes_active_selection() {
    let mut surface = text_input_surface_with_selection("alpha beta", 10, 6, 10);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Backspace", 8);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "alpha ");
    assert_eq!(int_attr(&surface, "caret_offset"), 6);
    assert_eq!(int_attr(&surface, "selection_anchor"), 6);
    assert_eq!(int_attr(&surface, "selection_focus"), 6);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("alpha ".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_selection_replacement_respects_max_chars() {
    let mut surface = text_input_surface_with_selection_and_attributes(
        "abcd",
        3,
        1,
        3,
        [("max_chars", toml::Value::Integer(5))],
    );
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_text(&mut surface, "12345");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "a123d");
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
    assert_eq!(int_attr(&surface, "selection_anchor"), 4);
    assert_eq!(int_attr(&surface, "selection_focus"), 4);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("a123d".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_ime_preedit_replaces_active_selection_and_tracks_composition() {
    let mut surface = text_input_surface_with_selection("abcd", 3, 1, 3);
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "XY", None);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("ime.edit")
    );
    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert_eq!(text_attr(&surface, "content"), "aXYd");
    assert_eq!(int_attr(&surface, "caret_offset"), 3);
    assert_eq!(int_attr(&surface, "selection_anchor"), 3);
    assert_eq!(int_attr(&surface, "selection_focus"), 3);
    assert_eq!(int_attr(&surface, "composition_start"), 1);
    assert_eq!(int_attr(&surface, "composition_end"), 3);
    assert_eq!(text_attr(&surface, "composition_text"), "XY");
    assert_eq!(text_attr(&surface, "composition_restore_text"), "bc");
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("aXYd".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_ime_cancel_restores_selection_replacement_and_clears_owner() {
    let mut surface = text_input_surface_with_selection("abcd", 3, 1, 3);
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let preedit = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "XY", None);
    assert_eq!(preedit.reply.disposition, UiDispatchDisposition::Handled);

    let result = dispatch_ime(&mut surface, UiImeInputEventKind::Cancel, "", None);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(surface.input.input_method_owner, None);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "ime owner cleared"));
    assert_eq!(text_attr(&surface, "content"), "abcd");
    assert_eq!(int_attr(&surface, "caret_offset"), 3);
    assert_eq!(int_attr(&surface, "selection_anchor"), 3);
    assert_eq!(int_attr(&surface, "selection_focus"), 3);
    assert_eq!(int_attr(&surface, "composition_start"), 3);
    assert_eq!(int_attr(&surface, "composition_end"), 3);
    assert_eq!(text_attr(&surface, "composition_text"), "");
    assert_eq!(text_attr(&surface, "composition_restore_text"), "");
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("abcd".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_ime_commit_replaces_composition_and_emits_commit_event() {
    let mut surface = text_input_surface_with_selection("abcd", 3, 1, 3);
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let preedit = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "XY", None);
    assert_eq!(preedit.reply.disposition, UiDispatchDisposition::Handled);

    let result = dispatch_ime(&mut surface, UiImeInputEventKind::Commit, "Z", None);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert_eq!(text_attr(&surface, "content"), "aZd");
    assert_eq!(int_attr(&surface, "caret_offset"), 2);
    assert_eq!(int_attr(&surface, "selection_anchor"), 2);
    assert_eq!(int_attr(&surface, "selection_focus"), 2);
    assert_eq!(int_attr(&surface, "composition_start"), 2);
    assert_eq!(int_attr(&surface, "composition_end"), 2);
    assert_eq!(text_attr(&surface, "composition_text"), "");
    assert_eq!(text_attr(&surface, "composition_restore_text"), "");
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::Commit {
            property: "content".to_string(),
            value: UiValue::String("aZd".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_text_event_applies_filter_and_max_chars() {
    let mut surface = text_input_surface_with_attributes(
        "ab",
        2,
        [
            ("input_filter", toml::Value::String("digits".to_string())),
            ("max_chars", toml::Value::Integer(4)),
        ],
    );
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_text(&mut surface, "1a23b");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("text.edit")
    );
    assert_eq!(text_attr(&surface, "content"), "ab12");
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("ab12".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_text_event_respects_explicit_single_line() {
    let mut surface =
        text_input_surface_with_attributes("a", 1, [("multiline", toml::Value::Boolean(false))]);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_text(&mut surface, "b\nc\r\nd");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "abcd");
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
    assert_widget_binding_report(&result.binding_reports);
}

fn assert_widget_binding_report(
    reports: &[zircon_runtime_interface::ui::binding::UiBindingUpdateReport],
) {
    assert!(!reports.is_empty());
    assert!(reports.iter().any(|report| {
        report
            .updates
            .first()
            .is_some_and(|update| update.source.kind == UiBindingSourceKind::WidgetBehavior)
    }));
}

fn assert_clipboard_request(
    result: &zircon_runtime_interface::ui::dispatch::UiInputDispatchResult,
    kind: UiClipboardRequestKind,
    text: Option<&str>,
) {
    assert_eq!(result.reply.effects.len(), 1);
    assert_eq!(result.applied_effects.len(), 1);
    assert_eq!(result.host_requests.len(), 1);
    let UiDispatchHostRequestKind::Clipboard(request) = &result.host_requests[0].request else {
        panic!("expected clipboard host request");
    };
    assert_eq!(request.kind, kind);
    assert_eq!(request.owner, UiNodeId::new(2));
    assert_eq!(request.text.as_deref(), text);
}

fn dispatch_key(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    dispatch_key_with_metadata(
        surface,
        logical_key,
        key_code,
        |_: &mut UiInputEventMetadata| {},
    )
}

fn dispatch_key_with_shift(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    dispatch_key_with_metadata(surface, logical_key, key_code, |metadata| {
        metadata.modifiers.shift = true;
    })
}

fn dispatch_key_with_control(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    dispatch_key_with_metadata(surface, logical_key, key_code, |metadata| {
        metadata.modifiers.control = true;
    })
}

fn dispatch_key_with_control_shift(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    dispatch_key_with_metadata(surface, logical_key, key_code, |metadata| {
        metadata.modifiers.control = true;
        metadata.modifiers.shift = true;
    })
}

fn dispatch_key_with_metadata(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
    configure: impl FnOnce(&mut UiInputEventMetadata),
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(30), UiInputSequence::new(3));
    configure(&mut metadata);
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
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
        .unwrap()
}

fn dispatch_text(
    surface: &mut UiSurface,
    text: &str,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Text(UiTextInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(31),
                    UiInputSequence::new(4),
                ),
                text: text.to_string(),
            }),
        )
        .unwrap()
}

fn dispatch_ime(
    surface: &mut UiSurface,
    kind: UiImeInputEventKind,
    text: &str,
    cursor_range: Option<UiTextByteRange>,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(32),
                    UiInputSequence::new(5),
                ),
                kind,
                text: text.to_string(),
                cursor_range,
            }),
        )
        .unwrap()
}

fn text_input_surface(value: &str, caret_offset: usize) -> UiSurface {
    text_input_surface_with_attributes(value, caret_offset, [])
}

fn text_input_surface_with_selection(
    value: &str,
    caret_offset: usize,
    selection_anchor: usize,
    selection_focus: usize,
) -> UiSurface {
    text_input_surface_with_selection_and_attributes(
        value,
        caret_offset,
        selection_anchor,
        selection_focus,
        [],
    )
}

fn text_input_surface_with_selection_and_attributes(
    value: &str,
    caret_offset: usize,
    selection_anchor: usize,
    selection_focus: usize,
    extra_attributes: impl IntoIterator<Item = (&'static str, toml::Value)>,
) -> UiSurface {
    text_input_surface_with_attributes(
        value,
        caret_offset,
        [
            (
                "selection_anchor",
                toml::Value::Integer(selection_anchor as i64),
            ),
            (
                "selection_focus",
                toml::Value::Integer(selection_focus as i64),
            ),
        ]
        .into_iter()
        .chain(extra_attributes),
    )
}

fn text_input_surface_with_attributes(
    value: &str,
    caret_offset: usize,
    extra_attributes: impl IntoIterator<Item = (&'static str, toml::Value)>,
) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.text_input.keyboard"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 80.0)),
    );
    let attributes = [
        (
            "content".to_string(),
            toml::Value::String(value.to_string()),
        ),
        (
            "caret_offset".to_string(),
            toml::Value::Integer(caret_offset as i64),
        ),
    ]
    .into_iter()
    .chain(
        extra_attributes
            .into_iter()
            .map(|(key, value)| (key.to_string(), value)),
    )
    .collect();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/search"))
                .with_frame(UiFrame::new(8.0, 8.0, 160.0, 28.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "SearchBox".to_string(),
                    attributes,
                    bindings: vec![
                        binding("SearchBox/Change", UiEventKind::Change),
                        binding("SearchBox/Submit", UiEventKind::Submit),
                    ],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::TextInput,
                        value_property: Some("content".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn text_attr(surface: &UiSurface, key: &str) -> String {
    surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get(key)
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn int_attr(surface: &UiSurface, key: &str) -> i64 {
    surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get(key)
        .and_then(toml::Value::as_integer)
        .unwrap_or_default()
}

fn binding(id: &str, event: UiEventKind) -> UiBindingRef {
    UiBindingRef {
        id: id.to_string(),
        event,
        route: Some(id.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
}

fn focusable_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        ..UiStateFlags::default()
    }
}

fn combining_acute_cluster() -> &'static str {
    "e\u{301}"
}
