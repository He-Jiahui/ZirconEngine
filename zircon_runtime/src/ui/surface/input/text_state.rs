use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::UiNodeId,
    surface::{
        UiEditableTextState, UiTextCaret, UiTextCaretAffinity, UiTextComposition, UiTextRange,
        UiTextSelection,
    },
    widget::UiWidgetBehavior,
};

use crate::ui::editable_text_composition::composition_clauses_from_metadata;
use crate::ui::secure_text_policy::secure_text_policy;
use crate::ui::text::clamp_grapheme_boundary;

use super::super::surface::UiSurface;

pub(in crate::ui) fn editable_text_state_for_node(
    surface: &UiSurface,
    target: UiNodeId,
) -> Option<UiEditableTextState> {
    crate::profile_scope!("runtime", "ui_text.edit", "state_materialize");
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
    let text = if is_number_field_metadata(metadata) {
        number_field_text(metadata, property.as_str())
    } else {
        string_attribute(metadata, property.as_str())
            .or_else(|| string_attribute(metadata, "value_text"))
            .or_else(|| string_attribute(metadata, "text"))
            .or_else(|| metadata.widget.value.as_ref().map(UiValue::display_text))
            .unwrap_or_default()
    };
    super::editable_text::profile::record_state_materialization(text.len());
    let caret_offset = usize_attribute(metadata, "caret_offset").unwrap_or(text.len());
    let selection = usize_attribute(metadata, "selection_anchor")
        .zip(usize_attribute(metadata, "selection_focus"))
        .map(|(anchor, focus)| UiTextSelection {
            anchor: clamp_grapheme_boundary(&text, anchor),
            focus: clamp_grapheme_boundary(&text, focus),
        });
    let composition = usize_attribute(metadata, "composition_start")
        .zip(usize_attribute(metadata, "composition_end"))
        .zip(string_attribute(metadata, "composition_text"))
        .map(|((start, end), composition_text)| UiTextComposition {
            range: UiTextRange {
                start: clamp_grapheme_boundary(&text, start),
                end: clamp_grapheme_boundary(&text, end),
            },
            preedit_clauses: composition_clauses_from_metadata(metadata, &composition_text),
            text: composition_text,
            restore_text: string_attribute(metadata, "composition_restore_text"),
        });

    Some(UiEditableTextState {
        caret: UiTextCaret {
            offset: clamp_grapheme_boundary(&text, caret_offset),
            affinity: caret_affinity_from_metadata(metadata),
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

fn caret_affinity_from_metadata(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
) -> UiTextCaretAffinity {
    metadata
        .attributes
        .get("caret_affinity")
        .and_then(toml::Value::as_str)
        .is_some_and(|value| value.eq_ignore_ascii_case("upstream"))
        .then_some(UiTextCaretAffinity::Upstream)
        .unwrap_or(UiTextCaretAffinity::Downstream)
}

pub(crate) fn editable_text_input_is_secure(surface: &UiSurface, target: UiNodeId) -> bool {
    let Some(metadata) = surface
        .tree
        .nodes
        .get(&target)
        .and_then(|node| node.template_metadata.as_ref())
    else {
        return false;
    };
    is_editable_text_component(metadata) && secure_text_policy(metadata).is_secure()
}

pub(in crate::ui) fn is_editable_text_input(surface: &UiSurface, target: UiNodeId) -> bool {
    surface
        .tree
        .nodes
        .get(&target)
        .and_then(|node| node.template_metadata.as_ref())
        .is_some_and(is_editable_text_component)
}

pub(in crate::ui::surface) fn is_editable_text_derived_property(property: &str) -> bool {
    matches!(
        property,
        "caret_offset"
            | "caret_affinity"
            | "selection_anchor"
            | "selection_focus"
            | "composition_start"
            | "composition_end"
            | "composition_text"
            | "composition_restore_text"
            | "composition_clauses"
    )
}

pub(in crate::ui::surface) fn is_number_field_internal_property(
    surface: &UiSurface,
    target: UiNodeId,
    property: &str,
) -> bool {
    surface
        .tree
        .nodes
        .get(&target)
        .and_then(|node| node.template_metadata.as_ref())
        .is_some_and(is_number_field_metadata)
        && matches!(
            property,
            "value_text"
                | "number_edit_active"
                | "number_value_revision"
                | "number_edit_base_revision"
        )
}

pub(in crate::ui) fn editable_value_property(
    surface: &UiSurface,
    target: UiNodeId,
) -> Option<String> {
    let metadata = surface
        .tree
        .nodes
        .get(&target)?
        .template_metadata
        .as_ref()?;
    Some(editable_value_property_for_metadata(metadata).to_string())
}

pub(in crate::ui) fn editable_value_property_for_metadata(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
) -> &str {
    if let Some(property) = metadata.widget.value_property.as_ref() {
        property.as_str()
    } else if is_query_text_value_property(metadata) {
        "query"
    } else if metadata.attributes.contains_key("value") {
        "value"
    } else if metadata.attributes.contains_key("value_text") {
        "value_text"
    } else if metadata.attributes.contains_key("text") {
        "text"
    } else {
        "value"
    }
}

pub(in crate::ui) fn is_number_field_metadata(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
) -> bool {
    metadata.component == "NumberField"
}

fn number_field_text(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
    value_property: &str,
) -> String {
    if bool_attribute_any(metadata, &["number_edit_active"]).unwrap_or(false) {
        return string_attribute(metadata, "value_text").unwrap_or_default();
    }
    metadata
        .attributes
        .get(value_property)
        .map(UiValue::from_toml)
        .or_else(|| metadata.widget.value.clone())
        .map(|value| value.display_text())
        .unwrap_or_default()
}

pub(in crate::ui) fn is_editable_text_component(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
) -> bool {
    bool_attribute_any(metadata, &["editable_text", "editableText"]).unwrap_or(false)
        || metadata
            .widget
            .resolved_behavior(metadata.component.as_str())
            == UiWidgetBehavior::TextInput
        || metadata.component == "Autocomplete"
}

fn is_query_text_value_property(
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
) -> bool {
    if !metadata.attributes.contains_key("query") {
        return false;
    }
    if matches!(
        metadata.component.as_str(),
        "Autocomplete" | "SearchField" | "SearchInput"
    ) {
        return true;
    }
    !metadata.attributes.contains_key("value") && !metadata.attributes.contains_key("text")
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
