use std::collections::BTreeMap;

use toml::Value as TomlValue;
use zircon_runtime_interface::ui::component::{UiComponentState, UiDragSourceMetadata, UiValue};

use super::{primary_property_for_control, UiComponentShowcaseDemoState};

pub(super) fn project_state_panel_node(
    state: &UiComponentShowcaseDemoState,
    control_id: &str,
    attributes: &mut BTreeMap<String, TomlValue>,
) -> bool {
    let value = match control_id {
        "ComponentShowcaseSelectedCategory" => Some(state.selected_category.clone()),
        "ComponentShowcaseLastControl" => Some(
            state
                .event_log
                .last()
                .map(|entry| entry.control_id.clone())
                .unwrap_or_else(|| "No component event yet".to_string()),
        ),
        "ComponentShowcaseLastAction" => Some(
            state
                .event_log
                .last()
                .map(|entry| entry.action.clone())
                .unwrap_or_else(|| "Waiting for Runtime UI event".to_string()),
        ),
        "ComponentShowcaseCurrentValue" => Some(current_value_summary(state)),
        "ComponentShowcaseValidation" => Some(validation_summary(state)),
        "ComponentShowcaseDragPayload" => Some(drag_payload_summary(state)),
        _ => None,
    };

    let Some(value) = value else {
        return false;
    };
    attributes.insert("value".to_string(), TomlValue::String(value.clone()));
    attributes.insert("value_text".to_string(), TomlValue::String(value));
    true
}

fn current_value_summary(showcase: &UiComponentShowcaseDemoState) -> String {
    let Some(entry) = showcase.event_log.last() else {
        return "Select or edit a control".to_string();
    };
    if let Some(value) = entry
        .value_text
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        return value.to_string();
    }
    let Some(state) = showcase.state_for_control(&entry.control_id) else {
        return entry
            .value_text
            .clone()
            .unwrap_or_else(|| "No value payload".to_string());
    };
    primary_property_for_control(&entry.control_id)
        .and_then(|property| state.value(property))
        .map(UiValue::display_text)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| state_flags_summary(&state))
}

fn validation_summary(showcase: &UiComponentShowcaseDemoState) -> String {
    let Some(entry) = showcase.event_log.last() else {
        return "normal".to_string();
    };
    let Some(state) = showcase.state_for_control(&entry.control_id) else {
        return "normal".to_string();
    };
    let validation = &state.validation;
    match &validation.message {
        Some(message) if !message.is_empty() => format!("{}: {message}", validation.level_name()),
        _ => validation.level_name().to_string(),
    }
}

fn drag_payload_summary(showcase: &UiComponentShowcaseDemoState) -> String {
    let Some(entry) = showcase.event_log.last() else {
        return "No retained drop payload".to_string();
    };
    let Some(state) = showcase.state_for_control(&entry.control_id) else {
        return "No retained drop payload".to_string();
    };
    if let Some(source) = state
        .reference_source("value")
        .and_then(UiDragSourceMetadata::summary)
    {
        return format!("Dropped from {source}");
    }
    let flags = &state.flags;
    if flags.active_drag_target {
        "Active drop target".to_string()
    } else if flags.drop_hovered {
        "Drop hover".to_string()
    } else if flags.dragging {
        "Dragging".to_string()
    } else {
        "No retained drop payload".to_string()
    }
}

fn state_flags_summary(state: &UiComponentState) -> String {
    let flags = &state.flags;
    let mut states = Vec::new();
    if flags.focused {
        states.push("focused");
    }
    if flags.hovered {
        states.push("hovered");
    }
    if flags.pressed {
        states.push("pressed");
    }
    if flags.dragging {
        states.push("dragging");
    }
    if flags.popup_open {
        states.push("popup open");
    }
    if flags.expanded {
        states.push("expanded");
    }
    if flags.selected {
        states.push("selected");
    }
    if flags.checked {
        states.push("checked");
    }
    if states.is_empty() {
        "No value payload".to_string()
    } else {
        states.join(", ")
    }
}
