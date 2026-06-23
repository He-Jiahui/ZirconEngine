use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::UiComponentEventReport,
    event_ui::UiNodeId,
    tree::{UiTemplateNodeMetadata, UiTreeError},
};

use crate::ui::surface::UiSurface;

const DEFAULT_TYPEAHEAD_TIMEOUT_MS: u64 = 500;
const DEFAULT_SUBMENU_HOVER_DELAY_MS: u64 = 300;
const DEFAULT_TOOLTIP_DELAY_MS: u64 = 500;

impl UiSurface {
    pub(crate) fn typeahead_timeout_ms_for_component_node(&self, node_id: UiNodeId) -> Option<u64> {
        let node = self.tree.node(node_id)?;
        let metadata = node.template_metadata.as_ref()?;
        if !is_menu_component(metadata) || !self.widget_interaction_enabled(node_id, node, metadata)
        {
            return None;
        }
        Some(
            u64_attribute_value(&metadata.attributes, "typeahead_timeout_ms")
                .unwrap_or(DEFAULT_TYPEAHEAD_TIMEOUT_MS),
        )
    }

    pub(crate) fn submenu_hover_delay_ms_for_component_node(
        &self,
        node_id: UiNodeId,
    ) -> Option<u64> {
        let node = self.tree.node(node_id)?;
        let metadata = node.template_metadata.as_ref()?;
        if !is_menu_component(metadata) || !self.widget_interaction_enabled(node_id, node, metadata)
        {
            return None;
        }
        Some(
            u64_attribute_value(&metadata.attributes, "submenu_hover_delay_ms")
                .unwrap_or(DEFAULT_SUBMENU_HOVER_DELAY_MS),
        )
    }

    pub(crate) fn tooltip_timer_for_component_node(
        &self,
        node_id: UiNodeId,
    ) -> Option<(String, u64)> {
        let node = self.tree.node(node_id)?;
        let metadata = node.template_metadata.as_ref()?;
        if !self.widget_interaction_enabled(node_id, node, metadata) {
            return None;
        }
        let tooltip_id = tooltip_id_for_metadata(metadata)?;
        let delay_ms = first_u64_attribute_value(
            &metadata.attributes,
            &[
                "tooltip_delay_ms",
                "tooltipDelayMs",
                "enter_delay_ms",
                "enterDelay",
                "delay",
            ],
        )
        .unwrap_or(DEFAULT_TOOLTIP_DELAY_MS);
        Some((tooltip_id, delay_ms))
    }

    pub(crate) fn apply_default_typeahead_expired_component_event(
        &self,
        node_id: UiNodeId,
    ) -> Result<Vec<UiComponentEventReport>, UiTreeError> {
        if self
            .typeahead_timeout_ms_for_component_node(node_id)
            .is_none()
        {
            return Ok(Vec::new());
        }
        self.component_event_reports_for_bindings(
            node_id,
            UiEventKind::Change,
            UiComponentEvent::TypeaheadExpired,
            true,
        )
    }

    pub(crate) fn apply_default_submenu_hover_ready_component_event(
        &self,
        node_id: UiNodeId,
    ) -> Result<Vec<UiComponentEventReport>, UiTreeError> {
        if self
            .submenu_hover_delay_ms_for_component_node(node_id)
            .is_none()
        {
            return Ok(Vec::new());
        }
        self.component_event_reports_for_bindings(
            node_id,
            UiEventKind::Change,
            UiComponentEvent::ValueChanged {
                property: "submenu_hover_ready".to_string(),
                value: UiValue::Bool(true),
            },
            true,
        )
    }
}

fn is_menu_component(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "Menu"
            | "MenuList"
            | "PopupMenu"
            | "MenuPopup"
            | "ContextMenu"
            | "ContextActionMenu"
            | "DropdownPopup"
    ) || metadata
        .attributes
        .get("role")
        .and_then(toml::Value::as_str)
        .is_some_and(|role| {
            matches!(
                role,
                "menu" | "menu-list" | "context-menu" | "dropdown-popup"
            )
        })
}

fn u64_attribute_value(
    values: &std::collections::BTreeMap<String, toml::Value>,
    key: &str,
) -> Option<u64> {
    values
        .get(key)
        .and_then(toml::Value::as_integer)
        .map(|value| value.max(0) as u64)
}

fn first_u64_attribute_value(
    values: &std::collections::BTreeMap<String, toml::Value>,
    keys: &[&str],
) -> Option<u64> {
    keys.iter().find_map(|key| u64_attribute_value(values, key))
}

fn tooltip_id_for_metadata(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    metadata
        .widget
        .tooltip
        .clone()
        .or_else(|| {
            first_string_attribute_value(
                &metadata.attributes,
                &["tooltip_id", "tooltipId", "tooltip", "tooltipTitle"],
            )
        })
        .filter(|value| !value.is_empty())
}

fn first_string_attribute_value(
    values: &std::collections::BTreeMap<String, toml::Value>,
    keys: &[&str],
) -> Option<String> {
    keys.iter()
        .filter_map(|key| values.get(*key).and_then(toml::Value::as_str))
        .map(str::trim)
        .find(|value| !value.is_empty())
        .map(str::to_string)
}
