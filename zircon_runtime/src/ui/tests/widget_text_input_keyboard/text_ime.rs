use super::*;

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
fn text_input_ime_preedit_publishes_filter_and_capacity_receipt() {
    let mut surface = text_input_surface_with_selection_and_attributes(
        "abcd",
        3,
        1,
        3,
        [
            ("input_filter", toml::Value::String("digits".to_string())),
            ("max_chars", toml::Value::Integer(3)),
        ],
    );
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "A12", None);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "a1d");
    let receipt = result
        .diagnostics
        .text_constraint
        .expect("IME sanitization publishes the shared typed constraint receipt");
    assert_eq!(receipt.removed_hard_line_count, 0);
    assert_eq!(receipt.removed_filter_scalar_count, 1);
    assert!(receipt.max_graphemes_truncated);
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
    assert!(
        result
            .diagnostics
            .notes
            .iter()
            .any(|note| note == "ime owner cleared")
    );
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
    let receipt = result
        .diagnostics
        .text_constraint
        .expect("filtered and truncated text publishes a typed constraint receipt");
    assert_eq!(receipt.removed_hard_line_count, 0);
    assert_eq!(receipt.removed_filter_scalar_count, 2);
    assert!(receipt.max_graphemes_truncated);
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

    let result = dispatch_text(
        &mut surface,
        "b\nc\r\nd\u{2028}e\u{0085}f\u{000b}g\u{000c}h\u{2029}i",
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "abcdefghi");
    assert_eq!(int_attr(&surface, "caret_offset"), 9);
    let receipt = result
        .diagnostics
        .text_constraint
        .expect("single-line sanitization publishes a typed constraint receipt");
    assert_eq!(receipt.removed_hard_line_count, 7);
    assert_eq!(receipt.removed_filter_scalar_count, 0);
    assert!(!receipt.max_graphemes_truncated);
    assert_widget_binding_report(&result.binding_reports);
}
