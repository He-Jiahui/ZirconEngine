use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::{
    component::{
        UiComponentDescriptor, UiComponentEvent, UiComponentEventError, UiComponentState,
        UiValidationState, UiValue,
    },
    surface::{
        UiEditableTextState, UiTextCaret, UiTextCaretAffinity, UiTextComposition, UiTextEditAction,
        UiTextRange, UiTextSelection,
    },
};

use crate::ui::text::apply_text_edit_action;

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

pub(super) fn event_manages_validation(
    descriptor: &UiComponentDescriptor,
    event: &UiComponentEvent,
) -> bool {
    is_text_input_control(descriptor)
        && matches!(
            event,
            UiComponentEvent::KeyboardText { .. }
                | UiComponentEvent::ValueChanged { .. }
                | UiComponentEvent::Commit { .. }
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

pub(super) fn apply_keyboard_text(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    text: &str,
) -> Result<(), UiComponentEventError> {
    if text_input_read_only(state, descriptor) {
        return Ok(());
    }
    let Some(text) = keyboard_text_input_payload(text) else {
        return Ok(());
    };
    let Some(primary_property) = text_input_primary_property(descriptor) else {
        return Ok(());
    };

    let current_text = string_setting(state, descriptor, primary_property).unwrap_or_default();
    let next_state = apply_text_edit_action(
        text_input_edit_state(state, descriptor, current_text),
        UiTextEditAction::Insert { text },
    );
    write_text_input_edit_state(state, descriptor, primary_property, next_state);
    state.flags.focused = true;
    apply_validation_trigger(state, descriptor, TextInputValidationTrigger::Change);
    Ok(())
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
    let validation = validate_text(state, descriptor, &text);
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
    state.validation = validation;
    super::set_value(state, "validation_level".to_string(), UiValue::Enum(level));
    super::set_value(
        state,
        "validation_message".to_string(),
        UiValue::String(message),
    );
}

fn set_bool_state(state: &mut UiComponentState, property: &str, value: bool) {
    super::set_value(state, property.to_string(), UiValue::Bool(value));
}

fn validation_timing(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> TextInputValidationTiming {
    match string_setting(state, descriptor, "validation_timing")
        .unwrap_or_else(|| "commit".to_string())
        .chars()
        .filter(|ch| *ch != '_' && *ch != '-' && !ch.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .as_str()
    {
        "change" | "input" | "live" | "valuechanged" => TextInputValidationTiming::Change,
        "blur" | "focusout" | "focuslost" => TextInputValidationTiming::Blur,
        _ => TextInputValidationTiming::Commit,
    }
}

fn validation_text(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> String {
    let candidates = validation_property_candidates(descriptor);
    candidates
        .iter()
        .find_map(|property| state.values.get(*property).and_then(textual_value))
        .or_else(|| {
            candidates.iter().find_map(|property| {
                descriptor
                    .prop(property)
                    .and_then(|schema| schema.default_value.as_ref())
                    .and_then(textual_value)
            })
        })
        .unwrap_or_default()
}

fn validation_property_candidates(descriptor: &UiComponentDescriptor) -> Vec<&'static str> {
    let mut candidates = Vec::new();
    if let Some(primary) = text_input_primary_property(descriptor) {
        candidates.push(primary);
    }
    for property in ["query", "value_text", "text", "value"] {
        if !candidates.contains(&property) {
            candidates.push(property);
        }
    }
    candidates
}

fn text_input_edit_state(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    text: String,
) -> UiEditableTextState {
    let caret_offset = usize_state_value(state, "caret_offset")
        .map(|offset| clamp_text_input_boundary(&text, offset))
        .unwrap_or(text.len());
    let selection = usize_state_value(state, "selection_anchor")
        .zip(usize_state_value(state, "selection_focus"))
        .map(|(anchor, focus)| UiTextSelection {
            anchor: clamp_text_input_boundary(&text, anchor),
            focus: clamp_text_input_boundary(&text, focus),
        });
    let composition = usize_state_value(state, "composition_start")
        .zip(usize_state_value(state, "composition_end"))
        .zip(string_state_value(state, "composition_text"))
        .map(|((start, end), composition_text)| UiTextComposition {
            range: UiTextRange {
                start: clamp_text_input_boundary(&text, start),
                end: clamp_text_input_boundary(&text, end),
            },
            text: composition_text,
            restore_text: string_state_value(state, "composition_restore_text"),
        });

    UiEditableTextState {
        caret: UiTextCaret {
            offset: caret_offset,
            affinity: UiTextCaretAffinity::Downstream,
        },
        selection,
        composition,
        read_only: text_input_read_only(state, descriptor),
        text,
    }
}

fn write_text_input_edit_state(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    primary_property: &str,
    edit_state: UiEditableTextState,
) {
    super::set_value(
        state,
        primary_property.to_string(),
        UiValue::String(edit_state.text.clone()),
    );
    for mirror_property in text_input_mirror_properties(descriptor, primary_property) {
        super::set_value(
            state,
            mirror_property.to_string(),
            UiValue::String(edit_state.text.clone()),
        );
    }
    super::set_value(
        state,
        "caret_offset".to_string(),
        UiValue::Int(edit_state.caret.offset as i64),
    );
    let (selection_anchor, selection_focus) = edit_state
        .selection
        .as_ref()
        .map(|selection| (selection.anchor, selection.focus))
        .unwrap_or((edit_state.caret.offset, edit_state.caret.offset));
    super::set_value(
        state,
        "selection_anchor".to_string(),
        UiValue::Int(selection_anchor as i64),
    );
    super::set_value(
        state,
        "selection_focus".to_string(),
        UiValue::Int(selection_focus as i64),
    );

    let (composition_start, composition_end, composition_text, restore_text) = edit_state
        .composition
        .as_ref()
        .map(|composition| {
            (
                composition.range.start,
                composition.range.end,
                composition.text.clone(),
                composition.restore_text.clone().unwrap_or_default(),
            )
        })
        .unwrap_or((
            edit_state.caret.offset,
            edit_state.caret.offset,
            String::new(),
            String::new(),
        ));
    super::set_value(
        state,
        "composition_start".to_string(),
        UiValue::Int(composition_start as i64),
    );
    super::set_value(
        state,
        "composition_end".to_string(),
        UiValue::Int(composition_end as i64),
    );
    super::set_value(
        state,
        "composition_text".to_string(),
        UiValue::String(composition_text),
    );
    super::set_value(
        state,
        "composition_restore_text".to_string(),
        UiValue::String(restore_text),
    );
}

fn mirror_text_input_value(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) {
    let Some(text) = state.values.get(property).and_then(textual_value) else {
        return;
    };
    for mirror_property in text_input_mirror_properties(descriptor, property) {
        super::set_value(
            state,
            mirror_property.to_string(),
            UiValue::String(text.clone()),
        );
    }
}

fn text_input_read_only(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> bool {
    bool_setting(state, descriptor, "readOnly", false)
        || bool_setting(state, descriptor, "read_only", false)
        || bool_setting(state, descriptor, "inputReadOnly", false)
        || bool_setting(state, descriptor, "input_read_only", false)
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
) -> Vec<&'static str> {
    if primary_property == "query" {
        Vec::new()
    } else if primary_property == "value_text" && descriptor.prop("value").is_some() {
        vec!["value"]
    } else if primary_property == "value" && descriptor.prop("value_text").is_some() {
        vec!["value_text"]
    } else {
        Vec::new()
    }
}

fn keyboard_text_input_payload(text: &str) -> Option<String> {
    let text = text
        .chars()
        .filter(|ch| !ch.is_control())
        .collect::<String>();
    (!text.trim().is_empty()).then_some(text)
}

fn usize_state_value(state: &UiComponentState, property: &str) -> Option<usize> {
    match state.values.get(property) {
        Some(UiValue::Int(value)) if *value >= 0 => Some(*value as usize),
        _ => None,
    }
}

fn string_state_value(state: &UiComponentState, property: &str) -> Option<String> {
    state.values.get(property).and_then(textual_value)
}

fn clamp_text_input_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
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
    state
        .values
        .get(property)
        .and_then(non_empty_textual_value)
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(non_empty_textual_value)
        })
}

fn textual_value(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.clone()),
        _ => None,
    }
}

fn non_empty_textual_value(value: &UiValue) -> Option<String> {
    textual_value(value).filter(|value| !value.is_empty())
}
