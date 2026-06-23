use super::*;

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
