use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventKind, UiComponentState, UiValue,
};

use super::super::assert_has_event;

#[test]
fn range_slider_value_writes_sync_percent_mirrors() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let range_slider = registry.descriptor("RangeSlider").unwrap();
    assert_has_event(range_slider, UiComponentEventKind::ValueChanged);
    assert_has_event(range_slider, UiComponentEventKind::DragDelta);

    let mut state = UiComponentState::new()
        .with_value("range_min", UiValue::Float(20.0))
        .with_value("range_min_percent", UiValue::Float(0.2))
        .with_value("value", UiValue::Float(80.0))
        .with_value("value_percent", UiValue::Float(0.8))
        .with_value("min", UiValue::Float(0.0))
        .with_value("max", UiValue::Float(100.0))
        .with_value("step", UiValue::Float(10.0));

    state
        .apply_event(
            range_slider,
            UiComponentEvent::ValueChanged {
                property: "value".to_string(),
                value: UiValue::Float(50.0),
            },
        )
        .unwrap();
    assert_eq!(state.value("value"), Some(&UiValue::Float(50.0)));
    assert_eq!(state.value("value_percent"), Some(&UiValue::Float(0.5)));
    assert_eq!(
        state.value("range_min_percent"),
        Some(&UiValue::Float(0.2)),
        "upper thumb writes must not disturb the lower thumb mirror"
    );

    state
        .apply_event(
            range_slider,
            UiComponentEvent::ValueChanged {
                property: "range_min".to_string(),
                value: UiValue::Float(25.0),
            },
        )
        .unwrap();
    assert_eq!(state.value("range_min"), Some(&UiValue::Float(25.0)));
    assert_eq!(
        state.value("range_min_percent"),
        Some(&UiValue::Float(0.25))
    );

    state
        .apply_event(
            range_slider,
            UiComponentEvent::DragDelta {
                property: "value".to_string(),
                delta: 2.0,
            },
        )
        .unwrap();
    assert_eq!(state.value("value"), Some(&UiValue::Float(70.0)));
    assert_eq!(state.value("value_percent"), Some(&UiValue::Float(0.7)));
}

#[test]
fn range_slider_percent_writes_update_clamped_thumb_values() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let range_slider = registry.descriptor("RangeSlider").unwrap();

    let mut lower_state = UiComponentState::new()
        .with_value("range_min", UiValue::Float(40.0))
        .with_value("range_min_percent", UiValue::Float(0.4))
        .with_value("value", UiValue::Float(60.0))
        .with_value("value_percent", UiValue::Float(0.6))
        .with_value("min", UiValue::Float(0.0))
        .with_value("max", UiValue::Float(100.0));
    lower_state
        .apply_event(
            range_slider,
            UiComponentEvent::ValueChanged {
                property: "range_min_percent".to_string(),
                value: UiValue::Float(0.8),
            },
        )
        .unwrap();
    assert_eq!(
        lower_state.value("range_min"),
        Some(&UiValue::Float(60.0)),
        "lower percent writes should clamp against the current upper thumb"
    );
    assert_eq!(
        lower_state.value("range_min_percent"),
        Some(&UiValue::Float(0.6)),
        "percent mirror should be normalized after clamping the lower thumb"
    );

    let mut upper_state = UiComponentState::new()
        .with_value("range_min", UiValue::Float(40.0))
        .with_value("range_min_percent", UiValue::Float(0.4))
        .with_value("value", UiValue::Float(60.0))
        .with_value("value_percent", UiValue::Float(0.6))
        .with_value("min", UiValue::Float(0.0))
        .with_value("max", UiValue::Float(100.0));
    upper_state
        .apply_event(
            range_slider,
            UiComponentEvent::ValueChanged {
                property: "value_percent".to_string(),
                value: UiValue::Float(0.2),
            },
        )
        .unwrap();
    assert_eq!(
        upper_state.value("value"),
        Some(&UiValue::Float(40.0)),
        "upper percent writes should clamp against the current lower thumb"
    );
    assert_eq!(
        upper_state.value("value_percent"),
        Some(&UiValue::Float(0.4)),
        "upper percent mirror should reflect the clamped value"
    );
}
