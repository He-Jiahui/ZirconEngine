use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiComponentState, UiValue},
    dispatch::UiComponentEventReport,
    event_ui::UiNodeId,
    tree::{UiTemplateNodeMetadata, UiTreeError},
};

use super::UiSurface;

const CURRENT_TOAST_ID: &str = "current_toast_id";
const AUTO_HIDE_DURATION_MS: &str = "auto_hide_duration_ms";
const AUTO_HIDE_DURATION_CAMEL: &str = "autoHideDuration";

impl UiSurface {
    pub(crate) fn toast_timer_for_component_node(
        &self,
        node_id: UiNodeId,
    ) -> Option<(String, u64)> {
        let node = self.tree.node(node_id)?;
        let metadata = node.template_metadata.as_ref()?;
        if !is_toast_component(metadata)
            || !self.widget_interaction_enabled(node_id, node, metadata)
        {
            return None;
        }

        let component_state = self.component_states.get(node_id);
        let toast_id = string_retained_value(metadata, component_state, CURRENT_TOAST_ID)?;
        let timeout_ms = u64_retained_value(
            metadata,
            component_state,
            &[AUTO_HIDE_DURATION_MS, AUTO_HIDE_DURATION_CAMEL],
        )?;
        (timeout_ms > 0).then_some((toast_id, timeout_ms))
    }

    pub(crate) fn apply_default_toast_timeout_component_event(
        &self,
        node_id: UiNodeId,
        toast_id: &str,
    ) -> Result<Vec<UiComponentEventReport>, UiTreeError> {
        let Some((current_id, _)) = self.toast_timer_for_component_node(node_id) else {
            return Ok(Vec::new());
        };
        if toast_id.is_empty() || current_id != toast_id {
            return Ok(Vec::new());
        }

        self.component_event_reports_for_bindings(
            node_id,
            UiEventKind::Change,
            UiComponentEvent::Commit {
                property: "expired_toast_id".to_string(),
                value: UiValue::String(toast_id.to_string()),
            },
            true,
        )
    }
}

fn is_toast_component(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(metadata.component.as_str(), "Snackbar" | "Toast")
        || metadata
            .attributes
            .get("role")
            .and_then(toml::Value::as_str)
            .is_some_and(|role| matches!(role, "snackbar" | "toast"))
}

fn string_retained_value(
    metadata: &UiTemplateNodeMetadata,
    component_state: Option<&UiComponentState>,
    property: &str,
) -> Option<String> {
    component_state
        .and_then(|state| string_component_state_value(state, property))
        .or_else(|| string_attribute_value(metadata, property))
        .filter(|value| !value.is_empty())
}

fn u64_retained_value(
    metadata: &UiTemplateNodeMetadata,
    component_state: Option<&UiComponentState>,
    properties: &[&str],
) -> Option<u64> {
    properties.iter().find_map(|property| {
        component_state
            .and_then(|state| u64_component_state_value(state, property))
            .or_else(|| u64_attribute_value(metadata, property))
    })
}

fn string_attribute_value(metadata: &UiTemplateNodeMetadata, property: &str) -> Option<String> {
    metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_str)
        .map(str::to_string)
}

fn u64_attribute_value(metadata: &UiTemplateNodeMetadata, property: &str) -> Option<u64> {
    metadata
        .attributes
        .get(property)
        .and_then(toml::Value::as_integer)
        .map(|value| value.max(0) as u64)
}

fn string_component_state_value(state: &UiComponentState, property: &str) -> Option<String> {
    match state.value(property) {
        Some(UiValue::String(value) | UiValue::Enum(value)) => Some(value.clone()),
        _ => None,
    }
}

fn u64_component_state_value(state: &UiComponentState, property: &str) -> Option<u64> {
    match state.value(property) {
        Some(UiValue::Int(value)) => Some((*value).max(0) as u64),
        Some(UiValue::Float(value)) => Some((*value).round().max(0.0) as u64),
        Some(UiValue::String(value) | UiValue::Enum(value)) => value.parse::<u64>().ok(),
        _ => None,
    }
}
