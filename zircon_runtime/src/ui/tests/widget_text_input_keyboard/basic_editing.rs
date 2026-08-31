use super::*;
use zircon_runtime_interface::ui::dispatch::{
    UiNumberInputCommitMethod, UiNumberInputCommitStatus, UiNumberInputParseStatus,
};

#[test]
fn text_input_keyboard_backspace_uses_widget_value_property() {
    let mut surface = text_input_surface("hello", 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let revision_before = text_layout_revision(&surface);

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
    assert_eq!(text_layout_revision(&surface), revision_before + 1);
}

#[test]
fn text_input_keyboard_each_value_change_advances_text_revision_before_rebuild() {
    let mut surface = text_input_surface("hello", 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let revision_before = text_layout_revision(&surface);

    let first = dispatch_key(&mut surface, "Backspace", 8);
    let second = dispatch_key(&mut surface, "Backspace", 8);

    assert_eq!(first.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(second.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "hel");
    assert_eq!(int_attr(&surface, "caret_offset"), 3);
    assert_eq!(text_layout_revision(&surface), revision_before + 2);
}

#[test]
fn input_manager_publishes_consecutive_receipts_from_one_retained_document() {
    let mut surface = text_input_surface("hello", 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let mut manager = UiInputManager::default();

    let first = dispatch_key_with_manager(&mut manager, &mut surface, "Backspace", 8);
    let second = dispatch_key_with_manager(&mut manager, &mut surface, "Backspace", 8);

    let UiWidgetEvent::TextEditChange { receipt: first } = &first.widget_events[0] else {
        panic!("expected a retained text edit receipt");
    };
    let UiWidgetEvent::TextEditChange { receipt: second } = &second.widget_events[0] else {
        panic!("expected a retained text edit receipt");
    };
    assert_eq!(first.document_id, second.document_id);
    assert_eq!(first.previous_revision.get(), 0);
    assert_eq!(first.revision.get(), 1);
    assert_eq!(second.previous_revision.get(), 1);
    assert_eq!(second.revision.get(), 2);
    assert_eq!(text_attr(&surface, "content"), "hel");
}

#[test]
fn text_input_keyboard_rejects_reserved_value_property_without_partial_state() {
    let mut surface = text_input_surface("hello", 5);
    let metadata = surface
        .tree
        .node_mut(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_mut())
        .expect("text input metadata");
    metadata.attributes.insert(
        "visibility".to_string(),
        toml::Value::String("hello".to_string()),
    );
    metadata.widget.value_property = Some("visibility".to_string());
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Backspace", 8);

    let metadata = surface
        .tree
        .node(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .expect("text input metadata");
    assert_eq!(
        metadata.attributes.get("visibility"),
        Some(&toml::Value::String("hello".to_string()))
    );
    assert_eq!(
        metadata.attributes.get("caret_offset"),
        Some(&toml::Value::Integer(5))
    );
    assert!(!metadata.attributes.contains_key("selection_anchor"));
    assert!(!metadata.attributes.contains_key("selection_focus"));
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
    assert!(
        result
            .diagnostics
            .notes
            .iter()
            .any(|note| note == "text_state_transaction_rejected:reserved_value_property")
    );
}

#[test]
fn number_field_text_edit_uses_a_string_buffer_without_corrupting_numeric_value() {
    let mut surface = number_field_surface(42.0, 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_text(&mut surface, "0");

    let metadata = surface
        .tree
        .node(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .expect("number field metadata");
    assert_eq!(
        metadata.attributes.get("value"),
        Some(&toml::Value::Float(42.0))
    );
    assert_eq!(text_attr(&surface, "value_text"), "420");
    assert_eq!(
        metadata.attributes.get("caret_offset"),
        Some(&toml::Value::Integer(3))
    );
    assert_eq!(bool_attr(&surface, "number_edit_active"), Some(true));
    assert_eq!(int_attr(&surface, "number_value_revision"), 0);
    assert_eq!(int_attr(&surface, "number_edit_base_revision"), 0);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
    assert_eq!(
        result
            .diagnostics
            .number_input
            .map(|receipt| receipt.parse_status),
        Some(UiNumberInputParseStatus::OutOfRange)
    );
}

#[test]
fn number_field_enter_clamps_and_publishes_a_typed_commit() {
    let mut surface = number_field_surface(42.0, 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let _ = dispatch_text(&mut surface, "0");

    let result = dispatch_key(&mut surface, "Enter", 13);

    assert_eq!(float_attr(&surface, "value"), Some(100.0));
    assert_eq!(text_attr(&surface, "value_text"), "100");
    assert_eq!(bool_attr(&surface, "number_edit_active"), Some(false));
    assert_eq!(int_attr(&surface, "number_value_revision"), 1);
    assert_eq!(int_attr(&surface, "number_edit_base_revision"), 1);
    assert_eq!(
        result.diagnostics.number_input.map(|receipt| (
            receipt.parse_status,
            receipt.commit_method,
            receipt.commit_status,
        )),
        Some((
            UiNumberInputParseStatus::OutOfRange,
            UiNumberInputCommitMethod::Enter,
            UiNumberInputCommitStatus::Clamped,
        ))
    );
    assert!(result.component_events.iter().any(|report| {
        matches!(
            &report.event,
            UiComponentEvent::Commit { property, value }
                if property == "value" && value == &UiValue::Float(100.0)
        )
    }));
}

#[test]
fn number_field_intermediate_enter_stays_editable_without_typed_commit() {
    let mut surface = number_field_surface(42.0, 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let _ = dispatch_key_with_control(&mut surface, "a", 65);
    let _ = dispatch_text(&mut surface, "-");

    let result = dispatch_key(&mut surface, "Enter", 13);

    assert_eq!(float_attr(&surface, "value"), Some(42.0));
    assert_eq!(text_attr(&surface, "value_text"), "-");
    assert_eq!(bool_attr(&surface, "number_edit_active"), Some(true));
    assert_eq!(
        result
            .diagnostics
            .number_input
            .map(|receipt| (receipt.parse_status, receipt.commit_status,)),
        Some((
            UiNumberInputParseStatus::Intermediate,
            UiNumberInputCommitStatus::Rejected,
        ))
    );
    assert!(result.component_events.is_empty());
}

#[test]
fn number_field_escape_cancels_the_buffer_and_focus_loss_reverts_invalid_text() {
    let mut surface = number_field_surface(42.0, 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let _ = dispatch_key_with_control(&mut surface, "a", 65);
    let _ = dispatch_text(&mut surface, "-");

    let cancelled = dispatch_key(&mut surface, "Escape", 27);

    assert_eq!(float_attr(&surface, "value"), Some(42.0));
    assert_eq!(text_attr(&surface, "value_text"), "42");
    assert_eq!(bool_attr(&surface, "number_edit_active"), Some(false));
    assert_eq!(int_attr(&surface, "number_value_revision"), 0);
    assert_eq!(int_attr(&surface, "number_edit_base_revision"), 0);
    assert_eq!(
        cancelled
            .diagnostics
            .number_input
            .map(|receipt| receipt.commit_status),
        Some(UiNumberInputCommitStatus::Cancelled)
    );

    let _ = dispatch_key_with_control(&mut surface, "a", 65);
    let _ = dispatch_text(&mut surface, ".");
    surface.clear_focus();

    assert_eq!(float_attr(&surface, "value"), Some(42.0));
    assert_eq!(text_attr(&surface, "value_text"), "42");
    assert_eq!(bool_attr(&surface, "number_edit_active"), Some(false));
}

#[test]
fn number_field_arrow_up_steps_the_canonical_value_and_exits_text_edit_mode() {
    let mut surface = number_field_surface(42.0, 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let _ = dispatch_text(&mut surface, "0");

    let result = dispatch_key(&mut surface, "ArrowUp", 38);

    assert_eq!(float_attr(&surface, "value"), Some(43.0));
    assert_eq!(text_attr(&surface, "value_text"), "43");
    assert_eq!(bool_attr(&surface, "number_edit_active"), Some(false));
    assert_eq!(int_attr(&surface, "number_value_revision"), 1);
    assert_eq!(int_attr(&surface, "number_edit_base_revision"), 1);
    assert_eq!(
        result.diagnostics.number_input.map(|receipt| (
            receipt.parse_status,
            receipt.commit_method,
            receipt.commit_status,
        )),
        Some((
            UiNumberInputParseStatus::Valid,
            UiNumberInputCommitMethod::KeyboardStep,
            UiNumberInputCommitStatus::Applied,
        ))
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.number_step")
    );
    assert!(result.component_events.iter().any(|report| {
        matches!(
            &report.event,
            UiComponentEvent::Commit { property, value }
                if property == "value" && value == &UiValue::Float(43.0)
        )
    }));
}

#[test]
fn number_field_repeated_arrow_down_reuses_the_typed_step_path() {
    let mut surface = number_field_surface(42.0, 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_state(
        &mut surface,
        "ArrowDown",
        40,
        UiKeyboardInputState::Repeated,
    );

    assert_eq!(float_attr(&surface, "value"), Some(41.0));
    assert_eq!(
        result
            .diagnostics
            .number_input
            .map(|receipt| receipt.commit_method),
        Some(UiNumberInputCommitMethod::KeyboardStep)
    );
}

#[test]
fn number_field_keyboard_step_rejects_invalid_step_without_partial_state() {
    let mut surface = number_field_surface(42.0, 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let _ = dispatch_key_with_control(&mut surface, "a", 65);
    let _ = dispatch_text(&mut surface, "-");
    let metadata = surface
        .tree
        .node_mut(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_mut())
        .expect("number field metadata");
    metadata
        .attributes
        .insert("step".to_string(), toml::Value::Float(0.0));
    let attributes_before = surface
        .tree
        .node(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .expect("number field metadata")
        .attributes
        .clone();

    let result = dispatch_key(&mut surface, "ArrowDown", 40);

    let attributes_after = &surface
        .tree
        .node(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .expect("number field metadata")
        .attributes;
    assert_eq!(attributes_after, &attributes_before);
    assert_eq!(
        result
            .diagnostics
            .number_input
            .map(|receipt| (receipt.parse_status, receipt.commit_status)),
        Some((
            UiNumberInputParseStatus::InvalidPolicy,
            UiNumberInputCommitStatus::Rejected,
        ))
    );
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
}

fn number_field_surface(value: f64, caret_offset: usize) -> UiSurface {
    let mut surface = text_input_surface(&UiValue::Float(value).display_text(), caret_offset);
    let metadata = surface
        .tree
        .node_mut(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_mut())
        .expect("number field metadata");
    metadata.component = "NumberField".to_string();
    metadata.attributes.remove("content");
    metadata
        .attributes
        .insert("value".to_string(), toml::Value::Float(value));
    metadata
        .attributes
        .insert("min".to_string(), toml::Value::Float(0.0));
    metadata
        .attributes
        .insert("max".to_string(), toml::Value::Float(100.0));
    metadata
        .attributes
        .insert("step".to_string(), toml::Value::Float(1.0));
    metadata.widget.value = Some(UiValue::Float(value));
    metadata.widget.value_property = Some("value".to_string());
    surface.rebuild();
    surface
}

fn float_attr(surface: &UiSurface, key: &str) -> Option<f64> {
    surface
        .tree
        .node(UiNodeId::new(2))?
        .template_metadata
        .as_ref()?
        .attributes
        .get(key)?
        .as_float()
}

fn bool_attr(surface: &UiSurface, key: &str) -> Option<bool> {
    surface
        .tree
        .node(UiNodeId::new(2))?
        .template_metadata
        .as_ref()?
        .attributes
        .get(key)?
        .as_bool()
}

#[test]
fn text_input_keyboard_arrow_left_moves_caret_without_value_event() {
    let mut surface = text_input_surface("he", 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let revision_before = text_layout_revision(&surface);

    let result = dispatch_key(&mut surface, "ArrowLeft", 37);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "he");
    assert_eq!(int_attr(&surface, "caret_offset"), 1);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
    assert_eq!(text_layout_revision(&surface), revision_before);
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
