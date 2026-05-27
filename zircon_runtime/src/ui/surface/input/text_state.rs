use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::UiNodeId,
    surface::{
        UiEditableTextState, UiTextCaret, UiTextCaretAffinity, UiTextComposition, UiTextRange,
        UiTextSelection,
    },
    widget::UiWidgetBehavior,
};

use super::super::surface::UiSurface;

pub(super) fn editable_text_state_for_node(
    surface: &UiSurface,
    target: UiNodeId,
) -> Option<UiEditableTextState> {
    let metadata = surface
        .tree
        .nodes
        .get(&target)?
        .template_metadata
        .as_ref()?;
    if !is_editable_text_component(metadata) {
        return None;
    }
    let property = editable_value_property(surface, target)?;
    let text = string_attribute(metadata, property.as_str())
        .or_else(|| string_attribute(metadata, "value_text"))
        .or_else(|| string_attribute(metadata, "text"))
        .or_else(|| metadata.widget.value.as_ref().map(UiValue::display_text))
        .unwrap_or_default();
    let caret_offset = usize_attribute(metadata, "caret_offset").unwrap_or(text.len());
    let selection = usize_attribute(metadata, "selection_anchor")
        .zip(usize_attribute(metadata, "selection_focus"))
        .map(|(anchor, focus)| UiTextSelection {
            anchor: clamp_text_boundary(&text, anchor),
            focus: clamp_text_boundary(&text, focus),
        });
    let composition = usize_attribute(metadata, "composition_start")
        .zip(usize_attribute(metadata, "composition_end"))
        .zip(string_attribute(metadata, "composition_text"))
        .map(|((start, end), composition_text)| UiTextComposition {
            range: UiTextRange {
                start: clamp_text_boundary(&text, start),
                end: clamp_text_boundary(&text, end),
            },
            text: composition_text,
            restore_text: string_attribute(metadata, "composition_restore_text"),
        });

    Some(UiEditableTextState {
        caret: UiTextCaret {
            offset: clamp_text_boundary(&text, caret_offset),
            affinity: UiTextCaretAffinity::Downstream,
        },
        selection,
        composition,
        read_only: bool_attribute_any(
            metadata,
            &["read_only", "readOnly", "input_read_only", "inputReadOnly"],
        )
        .unwrap_or(false),
        text,
    })
}

pub(super) fn editable_value_property(surface: &UiSurface, target: UiNodeId) -> Option<String> {
    let metadata = surface
        .tree
        .nodes
        .get(&target)?
        .template_metadata
        .as_ref()?;
    if let Some(property) = metadata.widget.value_property.as_ref() {
        Some(property.clone())
    } else if metadata.attributes.contains_key("value") {
        Some("value".to_string())
    } else if metadata.attributes.contains_key("text") {
        Some("text".to_string())
    } else {
        Some("value".to_string())
    }
}

pub(super) fn clamp_text_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn is_editable_text_component(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
) -> bool {
    bool_attribute_any(metadata, &["editable_text", "editableText"]).unwrap_or(false)
        || metadata
            .widget
            .resolved_behavior(metadata.component.as_str())
            == UiWidgetBehavior::TextInput
        || matches!(
            metadata.component.as_str(),
            "InputField"
                | "TextField"
                | "LineEdit"
                | "TextEdit"
                | "NumberField"
                | "InputBase"
                | "Input"
                | "OutlinedInput"
                | "FilledInput"
                | "TextareaAutosize"
                | "Autocomplete"
        )
}

fn string_attribute(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
    key: &str,
) -> Option<String> {
    metadata.attributes.get(key).and_then(|value| match value {
        toml::Value::String(value) => Some(value.clone()),
        toml::Value::Integer(value) => Some(value.to_string()),
        toml::Value::Float(value) => Some(value.to_string()),
        toml::Value::Boolean(value) => Some(value.to_string()),
        _ => None,
    })
}

fn usize_attribute(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
    key: &str,
) -> Option<usize> {
    metadata.attributes.get(key).and_then(|value| match value {
        toml::Value::Integer(value) => (*value >= 0).then_some(*value as usize),
        toml::Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        _ => None,
    })
}

fn bool_attribute_any(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
    keys: &[&str],
) -> Option<bool> {
    keys.iter()
        .find_map(|key| metadata.attributes.get(*key).and_then(toml::Value::as_bool))
}
