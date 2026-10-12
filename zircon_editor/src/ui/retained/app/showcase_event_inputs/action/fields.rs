use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;
use zircon_runtime_interface::ui::component::UiValue;

use super::super::action_matches;

pub(super) fn demo_field_input(action_id: &str) -> Option<UiComponentShowcaseDemoEventInput> {
    match action_id {
        action if action_matches(action, "number_field_drag_update") => {
            Some(UiComponentShowcaseDemoEventInput::DragDelta(5.0))
        }
        action if action_matches(action, "number_field_large_drag_update") => {
            Some(UiComponentShowcaseDemoEventInput::LargeDragDelta(1.0))
        }
        action if action_matches(action, "number_field_changed") => Some(
            UiComponentShowcaseDemoEventInput::Value(UiValue::Float(47.0)),
        ),
        action if action_matches(action, "range_field_drag_update") => {
            Some(UiComponentShowcaseDemoEventInput::DragDelta(5.0))
        }
        action if action_matches(action, "range_field_changed") => Some(
            UiComponentShowcaseDemoEventInput::Value(UiValue::Float(72.0)),
        ),
        action if action_matches(action, "color_field_changed") => Some(
            UiComponentShowcaseDemoEventInput::Value(UiValue::Color("#ffcc33".to_string())),
        ),
        action if action_matches(action, "vector2_field_changed") => {
            Some(UiComponentShowcaseDemoEventInput::Value(UiValue::Vec2([
                16.0, 32.0,
            ])))
        }
        action if action_matches(action, "vector3_field_changed") => {
            Some(UiComponentShowcaseDemoEventInput::Value(UiValue::Vec3([
                3.0, 4.0, 5.0,
            ])))
        }
        action if action_matches(action, "vector4_field_changed") => {
            Some(UiComponentShowcaseDemoEventInput::Value(UiValue::Vec4([
                0.25, 0.5, 0.75, 1.0,
            ])))
        }
        action if action_matches(action, "input_field") => {
            Some(UiComponentShowcaseDemoEventInput::Value(UiValue::String(
                "Runtime UI event".to_string(),
            )))
        }
        action if action_matches(action, "text_field") => {
            Some(UiComponentShowcaseDemoEventInput::Value(UiValue::String(
                "Runtime UI event-driven text".to_string(),
            )))
        }
        action
            if action_matches(action, "toggle_button_changed")
                || action_matches(action, "checkbox_changed") =>
        {
            Some(UiComponentShowcaseDemoEventInput::Toggle(false))
        }
        action if action_matches(action, "radio_changed") => {
            Some(UiComponentShowcaseDemoEventInput::Toggle(true))
        }
        action if action_matches(action, "slider_drag_update") => {
            Some(UiComponentShowcaseDemoEventInput::DragDelta(5.0))
        }
        action if action_matches(action, "slider_changed") => Some(
            UiComponentShowcaseDemoEventInput::Value(UiValue::Float(47.0)),
        ),
        action if action_matches(action, "range_slider_drag_update") => {
            Some(UiComponentShowcaseDemoEventInput::DragDelta(5.0))
        }
        action if action_matches(action, "range_slider_changed") => Some(
            UiComponentShowcaseDemoEventInput::Value(UiValue::Float(78.0)),
        ),
        _ => None,
    }
}
