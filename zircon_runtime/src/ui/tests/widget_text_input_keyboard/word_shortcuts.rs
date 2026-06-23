use super::*;

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
fn text_input_keyboard_escape_collapses_active_selection() {
    let value = "alpha beta";
    let mut surface = text_input_surface_with_selection(value, 5, 0, 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Escape", 27);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.text_edit")
    );
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert_eq!(int_attr(&surface, "selection_anchor"), 5);
    assert_eq!(int_attr(&surface, "selection_focus"), 5);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_escape_cancels_composition_before_selection_collapse() {
    let mut surface = text_input_surface_with_attributes(
        "aXYd",
        3,
        [
            ("composition_start", toml::Value::Integer(1)),
            ("composition_end", toml::Value::Integer(3)),
            ("composition_text", toml::Value::String("XY".to_string())),
            (
                "composition_restore_text",
                toml::Value::String("bc".to_string()),
            ),
        ],
    );
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Escape", 27);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
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
