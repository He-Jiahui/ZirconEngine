use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentState, UiValidationLevel, UiValue,
};

use super::UiComponentEventError;

#[test]
fn component_state_applies_dropdown_multiple_selection_and_special_options() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let dropdown = registry.descriptor("Dropdown").unwrap();
    assert!(
        dropdown
            .prop("options")
            .unwrap()
            .options
            .iter()
            .any(|option| option.special_condition.is_some()),
        "selection controls should expose a special-condition option for mixed inspector states"
    );

    let mut state = UiComponentState::new().with_value("multiple", UiValue::Bool(true));
    state
        .apply_event(
            dropdown,
            UiComponentEvent::SelectOption {
                property: "value".to_string(),
                option_id: "runtime".to_string(),
                selected: true,
            },
        )
        .unwrap();
    state
        .apply_event(
            dropdown,
            UiComponentEvent::SelectOption {
                property: "value".to_string(),
                option_id: "editor".to_string(),
                selected: true,
            },
        )
        .unwrap();
    state
        .apply_event(
            dropdown,
            UiComponentEvent::SelectOption {
                property: "value".to_string(),
                option_id: "runtime".to_string(),
                selected: false,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("value"),
        Some(&UiValue::Array(vec![UiValue::Enum("editor".to_string())]))
    );
}

#[test]
fn component_state_rejects_disabled_option_ids_from_retained_metadata() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let dropdown = registry.descriptor("Dropdown").unwrap();
    let mut state = UiComponentState::new()
        .with_value("multiple", UiValue::Bool(true))
        .with_value(
            "value",
            UiValue::Array(vec![UiValue::Enum("runtime".to_string())]),
        )
        .with_value(
            "disabled_options",
            UiValue::Array(vec![UiValue::String("debug".to_string())]),
        );

    let error = state
        .apply_event(
            dropdown,
            UiComponentEvent::SelectOption {
                property: "value".to_string(),
                option_id: "debug".to_string(),
                selected: true,
            },
        )
        .unwrap_err();

    assert!(matches!(
        error,
        UiComponentEventError::DisabledOption {
            option_id,
            ..
        } if option_id == "debug"
    ));
    assert_eq!(
        state.value("value"),
        Some(&UiValue::Array(vec![UiValue::Enum("runtime".to_string())]))
    );
    assert_eq!(state.validation.level, UiValidationLevel::Error);
}
