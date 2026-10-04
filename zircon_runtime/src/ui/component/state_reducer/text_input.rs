use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEvent, UiComponentEventError, UiComponentState,
    UiValidationState, UiValue,
};

const VALIDATION_CANDIDATES_DEFAULT: [&str; 4] = ["query", "value_text", "text", "value"];
const VALIDATION_CANDIDATES_VALUE_TEXT: [&str; 4] = ["value_text", "query", "text", "value"];
const VALIDATION_CANDIDATES_TEXT: [&str; 4] = ["text", "query", "value_text", "value"];
const VALIDATION_CANDIDATES_VALUE: [&str; 4] = ["value", "query", "value_text", "text"];
const VALIDATION_EMPTY_TEXT: &str = "";
const MIRROR_NONE: [&str; 0] = [];
const MIRROR_VALUE: [&str; 1] = ["value"];
const MIRROR_VALUE_TEXT: [&str; 1] = ["value_text"];

#[cfg(test)]
#[path = "text_input/tests/property_borrow_tests.rs"]
mod property_borrow_tests;

#[cfg(test)]
#[path = "text_input/tests/timing_normalization_tests.rs"]
mod timing_normalization_tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TextInputValidationTrigger {
    Change,
    Commit,
    Blur,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TextInputValidationTiming {
    Change,
    Commit,
    Blur,
}

// 文本事件自行维护验证状态；总入口据此避免在成功返回后把延迟、失焦或提交验证反馈覆盖为正常。
pub(super) fn event_manages_validation(
    descriptor: &UiComponentDescriptor,
    event: &UiComponentEvent,
) -> bool {
    is_text_input_control(descriptor)
        && matches!(
            event,
            UiComponentEvent::ValueChanged { .. }
                | UiComponentEvent::SecureValueChanged { .. }
                | UiComponentEvent::Commit { .. }
                | UiComponentEvent::SecureCommit { .. }
                | UiComponentEvent::Focus { .. }
        )
}

pub(super) fn is_text_input_control(descriptor: &UiComponentDescriptor) -> bool {
    matches!(
        descriptor.role.as_str(),
        "text-field"
            | "input"
            | "input-base"
            | "filled-input"
            | "outlined-input"
            | "textarea-autosize"
            | "search-field"
            | "field-editor"
            | "source-editor"
    ) || matches!(
        descriptor.id.as_str(),
        "TextField"
            | "Input"
            | "InputBase"
            | "FilledInput"
            | "OutlinedInput"
            | "TextareaAutosize"
            | "SearchField"
            | "FieldEditor"
            | "SourceEditor"
    )
}

pub(super) fn apply_value_event(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: String,
    value: UiValue,
    trigger: TextInputValidationTrigger,
) -> Result<(), UiComponentEventError> {
    super::apply_value(state, descriptor, property.clone(), value)?;
    mirror_text_input_value(state, descriptor, &property);
    apply_validation_trigger(state, descriptor, trigger);
    Ok(())
}

pub(super) fn apply_focus_event(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    focused: bool,
) -> Result<(), UiComponentEventError> {
    if !focused {
        set_bool_state(state, "validation_touched", true);
        apply_validation_trigger(state, descriptor, TextInputValidationTrigger::Blur);
    }
    Ok(())
}

fn apply_validation_trigger(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    trigger: TextInputValidationTrigger,
) {
    match trigger {
        TextInputValidationTrigger::Change => {
            set_bool_state(state, "validation_dirty", true);
            if validation_timing(state, descriptor) == TextInputValidationTiming::Change {
                validate_current_text(state, descriptor);
            } else {
                set_validation_state(state, UiValidationState::normal());
            }
        }
        TextInputValidationTrigger::Commit => {
            set_bool_state(state, "validation_dirty", false);
            set_bool_state(state, "validation_touched", true);
            validate_current_text(state, descriptor);
        }
        TextInputValidationTrigger::Blur => {
            set_bool_state(state, "validation_touched", true);
            if validation_timing(state, descriptor) == TextInputValidationTiming::Blur {
                set_bool_state(state, "validation_dirty", false);
                validate_current_text(state, descriptor);
            }
        }
    }
}

fn validate_current_text(state: &mut UiComponentState, descriptor: &UiComponentDescriptor) {
    let text = validation_text(state, descriptor);
    let validation = validate_text(state, descriptor, text);
    set_validation_state(state, validation);
}

fn validate_text(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    text: &str,
) -> UiValidationState {
    let override_message = string_setting(state, descriptor, "validation_message");
    if bool_setting(state, descriptor, "required", false) && text.trim().is_empty() {
        return UiValidationState::error(
            override_message.unwrap_or_else(|| "text value is required".to_string()),
        );
    }

    let text_len = text.graphemes(true).count() as i64;
    let min_length = int_setting(state, descriptor, "min_length")
        .unwrap_or(0)
        .max(0);
    if min_length > 0 && text_len < min_length {
        return UiValidationState::error(override_message.unwrap_or_else(|| {
            format!("text value must contain at least {min_length} characters")
        }));
    }

    let max_length = int_setting(state, descriptor, "max_length")
        .unwrap_or(0)
        .max(0);
    if max_length > 0 && text_len > max_length {
        return UiValidationState::error(override_message.unwrap_or_else(|| {
            format!("text value must contain at most {max_length} characters")
        }));
    }

    UiValidationState::normal()
}

fn set_validation_state(state: &mut UiComponentState, validation: UiValidationState) {
    let level = validation.level_name().to_string();
    let message = validation.message.clone().unwrap_or_default();
    // Only overwrite an existing custom validation message when the new state is
    // non-normal, or when there is no existing message to preserve.  This prevents
    // a normal-state transition in blur mode from silently clearing a message that
    // was set by an earlier error (see text_input_blur_timing_validates_on_focus_loss).
    let is_normal =
        validation.level == zircon_runtime_interface::ui::component::UiValidationLevel::Normal;
    let existing_message = state
        .values
        .get("validation_message")
        .and_then(|v| {
            if let zircon_runtime_interface::ui::component::UiValue::String(s) = v {
                Some(s.as_str())
            } else {
                None
            }
        })
        .unwrap_or("");
    let should_update_message = !is_normal || existing_message.is_empty();
    state.validation = validation;
    super::set_value(state, "validation_level".to_string(), UiValue::Enum(level));
    if should_update_message {
        super::set_value(
            state,
            "validation_message".to_string(),
            UiValue::String(message),
        );
    }
}

fn set_bool_state(state: &mut UiComponentState, property: &str, value: bool) {
    super::set_value(state, property.to_string(), UiValue::Bool(value));
}

fn validation_timing(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> TextInputValidationTiming {
    let timing = string_setting_ref(state, descriptor, "validation_timing").unwrap_or("commit");
    match first_normalized_timing_char(timing) {
        Some('c') if normalized_timing_matches(timing, "change") => {
            TextInputValidationTiming::Change
        }
        Some('i') if normalized_timing_matches(timing, "input") => {
            TextInputValidationTiming::Change
        }
        Some('l') if normalized_timing_matches(timing, "live") => TextInputValidationTiming::Change,
        Some('v') if normalized_timing_matches(timing, "valuechanged") => {
            TextInputValidationTiming::Change
        }
        Some('b') if normalized_timing_matches(timing, "blur") => TextInputValidationTiming::Blur,
        Some('f')
            if normalized_timing_matches(timing, "focusout")
                || normalized_timing_matches(timing, "focuslost") =>
        {
            TextInputValidationTiming::Blur
        }
        _ => TextInputValidationTiming::Commit,
    }
}

fn first_normalized_timing_char(value: &str) -> Option<char> {
    value
        .chars()
        .filter(|ch| *ch != '_' && *ch != '-' && !ch.is_whitespace())
        .flat_map(char::to_lowercase)
        .next()
}

fn normalized_timing_matches(value: &str, expected: &str) -> bool {
    let mut expected = expected.chars();
    for ch in value
        .chars()
        .filter(|ch| *ch != '_' && *ch != '-' && !ch.is_whitespace())
    {
        for normalized in ch.to_lowercase() {
            if expected.next() != Some(normalized) {
                return false;
            }
        }
    }
    expected.next().is_none()
}

fn validation_text<'a>(
    state: &'a UiComponentState,
    descriptor: &'a UiComponentDescriptor,
) -> &'a str {
    let candidates = validation_property_candidates(descriptor);
    candidates
        .iter()
        .find_map(|property| state.values.get(*property).and_then(textual_value_ref))
        .or_else(|| {
            candidates.iter().find_map(|property| {
                descriptor
                    .prop(property)
                    .and_then(|schema| schema.default_value.as_ref())
                    .and_then(textual_value_ref)
            })
        })
        .unwrap_or(VALIDATION_EMPTY_TEXT)
}

fn validation_property_candidates(descriptor: &UiComponentDescriptor) -> &'static [&'static str] {
    match text_input_primary_property(descriptor) {
        Some("value_text") => &VALIDATION_CANDIDATES_VALUE_TEXT,
        Some("text") => &VALIDATION_CANDIDATES_TEXT,
        Some("value") => &VALIDATION_CANDIDATES_VALUE,
        _ => &VALIDATION_CANDIDATES_DEFAULT,
    }
}

fn mirror_text_input_value(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) {
    let Some(text) = state.values.get(property).and_then(textual_value) else {
        return;
    };
    for mirror_property in text_input_mirror_properties(descriptor, property)
        .iter()
        .copied()
    {
        super::set_value(
            state,
            mirror_property.to_string(),
            UiValue::String(text.clone()),
        );
    }
}

fn text_input_primary_property(descriptor: &UiComponentDescriptor) -> Option<&'static str> {
    if descriptor.prop("query").is_some() {
        Some("query")
    } else if descriptor.prop("value_text").is_some() {
        Some("value_text")
    } else if descriptor.prop("text").is_some() {
        Some("text")
    } else if descriptor.prop("value").is_some() {
        Some("value")
    } else {
        None
    }
}

fn text_input_mirror_properties(
    descriptor: &UiComponentDescriptor,
    primary_property: &str,
) -> &'static [&'static str] {
    if primary_property == "query" {
        &MIRROR_NONE
    } else if primary_property == "value_text" && descriptor.prop("value").is_some() {
        &MIRROR_VALUE
    } else if primary_property == "value" && descriptor.prop("value_text").is_some() {
        &MIRROR_VALUE_TEXT
    } else {
        &MIRROR_NONE
    }
}

fn bool_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    default_value: bool,
) -> bool {
    state
        .values
        .get(property)
        .and_then(bool_value)
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(bool_value)
        })
        .unwrap_or(default_value)
}

fn bool_value(value: &UiValue) -> Option<bool> {
    match value {
        UiValue::Bool(value) => Some(*value),
        _ => None,
    }
}

fn int_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<i64> {
    state.values.get(property).and_then(int_value).or_else(|| {
        descriptor
            .prop(property)
            .and_then(|schema| schema.default_value.as_ref())
            .and_then(int_value)
    })
}

fn int_value(value: &UiValue) -> Option<i64> {
    match value {
        UiValue::Int(value) => Some(*value),
        UiValue::Float(value) => Some(value.round() as i64),
        _ => None,
    }
}

fn string_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<String> {
    string_setting_ref(state, descriptor, property).map(str::to_owned)
}

fn string_setting_ref<'a>(
    state: &'a UiComponentState,
    descriptor: &'a UiComponentDescriptor,
    property: &str,
) -> Option<&'a str> {
    state
        .values
        .get(property)
        .and_then(non_empty_textual_value_ref)
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(non_empty_textual_value_ref)
        })
}

fn textual_value(value: &UiValue) -> Option<String> {
    textual_value_ref(value).map(str::to_owned)
}

fn textual_value_ref(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.as_str()),
        _ => None,
    }
}

fn non_empty_textual_value_ref(value: &UiValue) -> Option<&str> {
    textual_value_ref(value).filter(|value| !value.is_empty())
}
