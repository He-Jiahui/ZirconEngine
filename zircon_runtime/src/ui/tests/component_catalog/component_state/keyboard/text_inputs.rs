use super::*;
use zircon_runtime_interface::ui::component::{UiComponentEventError, UiHostCapability};

#[test]
fn material_text_inputs_reject_raw_keyboard_text_without_mutating_state() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    for (component_id, property) in [
        ("TextField", "value_text"),
        ("Input", "value_text"),
        ("InputBase", "value_text"),
        ("FilledInput", "value_text"),
        ("OutlinedInput", "value_text"),
        ("TextareaAutosize", "value_text"),
        ("SearchField", "query"),
        ("FieldEditor", "value_text"),
        ("SourceEditor", "text"),
    ] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("{component_id} descriptor"));
        assert!(
            !descriptor.supports_event(UiComponentEventKind::KeyboardText),
            "{component_id} raw keyboard text must be owned by the surface edit transaction"
        );

        let mut state = UiComponentState::new()
            .with_value(property, UiValue::String("retained".to_string()))
            .with_value("caret_offset", UiValue::Int(3))
            .with_value("selection_anchor", UiValue::Int(1))
            .with_value("selection_focus", UiValue::Int(3));
        let error = state
            .apply_event(
                descriptor,
                UiComponentEvent::KeyboardText {
                    text: "X".to_string(),
                },
            )
            .unwrap_err();

        assert_eq!(
            error,
            UiComponentEventError::UnsupportedEvent {
                component_id: component_id.to_string(),
                event_kind: UiComponentEventKind::KeyboardText,
            }
        );
        assert_eq!(
            state.value(property),
            Some(&UiValue::String("retained".to_string()))
        );
        assert_eq!(state.value("caret_offset"), Some(&UiValue::Int(3)));
        assert_eq!(state.value("selection_anchor"), Some(&UiValue::Int(1)));
        assert_eq!(state.value("selection_focus"), Some(&UiValue::Int(3)));
        assert!(!state.flags.focused);
    }
}

#[test]
fn material_text_inputs_expose_semantic_edit_events_and_retained_edit_state() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    for component_id in [
        "TextField",
        "Input",
        "InputBase",
        "FilledInput",
        "OutlinedInput",
        "TextareaAutosize",
        "SearchField",
        "FieldEditor",
        "SourceEditor",
    ] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("{component_id} descriptor"));
        for event in [
            UiComponentEventKind::Focus,
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::Commit,
        ] {
            assert!(
                descriptor.supports_event(event),
                "{component_id} must expose semantic event {event:?}"
            );
        }
        assert!(
            descriptor
                .required_host_capabilities
                .contains(&UiHostCapability::TextInput),
            "{component_id} must declare the host text-input capability"
        );
        for property in [
            "caret_offset",
            "caret_affinity",
            "selection_anchor",
            "selection_focus",
            "composition_start",
            "composition_end",
            "composition_text",
            "composition_restore_text",
            "composition_clauses",
        ] {
            assert!(
                descriptor.prop(property).is_some(),
                "{component_id} missing retained edit property {property}"
            );
        }
    }
}
