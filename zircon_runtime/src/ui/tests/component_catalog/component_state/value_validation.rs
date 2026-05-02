use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentState, UiValidationLevel, UiValue, UiValueKind,
};

use super::UiComponentEventError;

#[test]
fn component_state_rejects_schema_value_kind_mismatches() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let color = registry.descriptor("ColorField").unwrap();
    let vector3 = registry.descriptor("Vector3Field").unwrap();

    let mut color_state =
        UiComponentState::new().with_value("value", UiValue::Color("#4d89ff".to_string()));
    let color_error = color_state
        .apply_event(
            color,
            UiComponentEvent::ValueChanged {
                property: "value".to_string(),
                value: UiValue::String("#ffcc33".to_string()),
            },
        )
        .unwrap_err();
    assert!(matches!(
        color_error,
        UiComponentEventError::InvalidValueKind {
            expected: UiValueKind::Color,
            actual: UiValueKind::String,
            ..
        }
    ));
    assert_eq!(
        color_state.value("value"),
        Some(&UiValue::Color("#4d89ff".to_string()))
    );
    assert_eq!(color_state.validation.level, UiValidationLevel::Error);

    let mut vector_state = UiComponentState::new();
    let vector_error = vector_state
        .apply_event(
            vector3,
            UiComponentEvent::Commit {
                property: "value".to_string(),
                value: UiValue::Vec2([1.0, 2.0]),
            },
        )
        .unwrap_err();
    assert!(matches!(
        vector_error,
        UiComponentEventError::InvalidValueKind {
            expected: UiValueKind::Vec3,
            actual: UiValueKind::Vec2,
            ..
        }
    ));
    assert_eq!(vector_state.value("value"), None);
    assert_eq!(vector_state.validation.level, UiValidationLevel::Error);
}
