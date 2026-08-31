use super::*;

#[test]
fn command_component_adapter_dispatches_committed_command_id_through_editor_events() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_command_component_adapter_commit");
    let envelope = command_commit_envelope("file.project.open");

    let result = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .expect("committed menu command id should dispatch through editor events");

    assert!(result.changed);
    assert!(!result.dirty);
    assert_eq!(
        result.transaction_id.as_deref(),
        Some("command:file.project.open")
    );
    assert_eq!(result.mutation_source.as_deref(), Some("command"));
    let journal = harness.runtime.journal();
    let record = journal
        .records()
        .last()
        .expect("command should write an event");
    assert_eq!(record.source, EditorEventSource::RetainedHost);
    assert_eq!(
        record.event,
        EditorEvent::WorkbenchMenu(MenuAction::OpenProject)
    );
    assert_eq!(
        harness.runtime.status_line(),
        "Open an existing project or create a renderable empty project."
    );
}

#[test]
fn command_component_adapter_rejects_non_string_command_value() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_command_component_adapter_invalid_value");
    let envelope = command_commit_envelope_with_value(UiValue::Int(42));

    let error = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .unwrap_err();

    assert_eq!(
        error,
        UiComponentAdapterError::InvalidValueKind {
            domain: "command".to_string(),
            path: "committed_command_id".to_string(),
            expected: UiValueKind::String,
            actual: UiValueKind::Int,
        }
    );
    assert!(harness.runtime.journal().records().is_empty());
}

#[test]
fn command_component_adapter_dispatches_palette_open_command() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_command_component_adapter_palette_open");
    let envelope = command_commit_envelope("editor.command.palette");

    let result = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .expect("palette open command should dispatch through command adapter");

    assert!(result.changed);
    let journal = harness.runtime.journal();
    let record = journal
        .records()
        .last()
        .expect("palette open command should append an editor event");
    assert_eq!(
        record.event,
        EditorEvent::Transient(EditorEventTransient::OpenCommandPalette)
    );
    assert!(record
        .effects
        .contains(&EditorEventEffect::CommandPaletteOpenRequested));
}

#[test]
fn keymap_dispatches_unhandled_keyboard_result_through_editor_command_binding() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_keymap_unhandled_keyboard");
    let result = keyboard_dispatch_result(
        "o",
        79,
        key_modifiers(true, false, false, false),
        UiKeyboardInputState::Pressed,
        UiDispatchReply::unhandled(),
    );

    let record = harness
        .runtime
        .dispatch_unhandled_input_keymap_command(&result, EditorEventSource::RetainedHost)
        .expect("unhandled keymap command should dispatch")
        .expect("Ctrl+O should resolve to a workbench command");

    assert_eq!(record.source, EditorEventSource::RetainedHost);
    assert_eq!(
        record.event,
        EditorEvent::WorkbenchMenu(MenuAction::OpenProject)
    );
    assert_eq!(
        harness.runtime.status_line(),
        "Open an existing project or create a renderable empty project."
    );
}

#[test]
fn keymap_dispatch_ignores_handled_and_released_keyboard_results() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_keymap_consumed_keyboard");
    let handled = keyboard_dispatch_result(
        "o",
        79,
        key_modifiers(true, false, false, false),
        UiKeyboardInputState::Pressed,
        UiDispatchReply::handled(),
    );
    let released = keyboard_dispatch_result(
        "o",
        79,
        key_modifiers(true, false, false, false),
        UiKeyboardInputState::Released,
        UiDispatchReply::unhandled(),
    );

    assert_eq!(
        harness
            .runtime
            .dispatch_unhandled_input_keymap_command(&handled, EditorEventSource::RetainedHost)
            .expect("handled keyboard result should be accepted"),
        None
    );
    assert_eq!(
        harness
            .runtime
            .dispatch_unhandled_input_keymap_command(&released, EditorEventSource::RetainedHost)
            .expect("released keyboard result should be accepted"),
        None
    );
    assert!(harness.runtime.journal().records().is_empty());
}

#[test]
fn inspector_component_adapter_rejects_missing_selection_without_mutation() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_adapter_missing_subject");
    let before = harness.runtime.editor_snapshot().inspector.unwrap().name;
    let envelope = UiComponentEventEnvelope::new(
        "res://ui/editor/host/inspector_surface_controls.zui",
        "NameField",
        UiComponentBindingTarget::new("inspector", "name"),
        UiComponentEvent::ValueChanged {
            property: "value".to_string(),
            value: UiValue::String("Should Not Apply".to_string()),
        },
    )
    .with_component_id("InspectorField");

    let error = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .unwrap_err();

    assert_eq!(
        error,
        UiComponentAdapterError::MissingSource {
            domain: "inspector".to_string(),
            path: "name".to_string(),
            source_name: "subject".to_string(),
        }
    );
    assert_eq!(
        harness.runtime.editor_snapshot().inspector.unwrap().name,
        before
    );
}

#[test]
fn inspector_component_adapter_rejects_unsupported_field() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_adapter_unsupported_field");
    let before = harness.runtime.editor_snapshot().inspector.unwrap();
    let envelope = inspector_value_envelope("transform.rotation.x", UiValue::Array(Vec::new()));

    let error = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .unwrap_err();

    assert_eq!(
        error,
        UiComponentAdapterError::UnsupportedTargetPath {
            domain: "inspector".to_string(),
            path: "transform.rotation.x".to_string(),
        }
    );
    let after = harness.runtime.editor_snapshot().inspector.unwrap();
    assert_eq!(after.name, before.name);
    assert_eq!(after.translation, before.translation);
}

#[test]
fn inspector_component_adapter_rejects_invalid_value_kind_for_supported_field() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_adapter_invalid_value_kind");
    let before = harness.runtime.editor_snapshot().inspector.unwrap();
    let envelope = inspector_value_envelope("name", UiValue::Bool(true));

    let error = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .unwrap_err();

    assert_eq!(
        error,
        UiComponentAdapterError::InvalidValueKind {
            domain: "inspector".to_string(),
            path: "name".to_string(),
            expected: UiValueKind::String,
            actual: UiValueKind::Bool,
        }
    );
    assert_eq!(
        harness.runtime.editor_snapshot().inspector.unwrap().name,
        before.name
    );
}

#[test]
fn inspector_component_adapter_rejects_non_value_property_without_mutation() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_ui_component_adapter_invalid_property");
    let before = harness.runtime.editor_snapshot().inspector.unwrap();
    let envelope = UiComponentEventEnvelope::new(
        "res://ui/editor/host/inspector_surface_controls.zui",
        "NameField",
        UiComponentBindingTarget::inspector("entity://selected", "name"),
        UiComponentEvent::ValueChanged {
            property: "label".to_string(),
            value: UiValue::String("Should Not Apply".to_string()),
        },
    )
    .with_component_id("InspectorField");

    let error = harness
        .runtime
        .dispatch_ui_component_adapter_event(&envelope)
        .unwrap_err();

    assert!(matches!(
        error,
        UiComponentAdapterError::RejectedInput { .. }
    ));
    assert_eq!(
        harness.runtime.editor_snapshot().inspector.unwrap().name,
        before.name
    );
}
