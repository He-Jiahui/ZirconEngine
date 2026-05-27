use zircon_runtime_interface::ui::{
    component::{UiComponentState, UiValue},
    event_ui::UiNodeId,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

use super::surface::UiSurface;

pub(crate) fn ui_surface_effective_disabled(
    surface: &UiSurface,
    node_id: UiNodeId,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    if ui_surface_node_disabled(surface, node_id, node, metadata) {
        return true;
    }
    let mut current = node.parent;
    while let Some(node_id) = current {
        let Some(parent) = surface.tree.nodes.get(&node_id) else {
            return true;
        };
        if ui_surface_node_disabled(surface, node_id, parent, parent.template_metadata.as_ref()) {
            return true;
        }
        current = parent.parent;
    }
    false
}

pub(crate) fn ui_surface_node_disabled(
    surface: &UiSurface,
    node_id: UiNodeId,
    node: &UiTreeNode,
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    let component_state = surface.component_states.get(node_id);
    !node.state_flags.enabled
        || component_state
            .and_then(disabled_component_state_value)
            .unwrap_or(false)
        || metadata.is_some_and(|metadata| {
            metadata.widget.disabled
                || bool_attribute_value(&metadata.attributes, "disabled") == Some(true)
        })
}

fn disabled_component_state_value(state: &UiComponentState) -> Option<bool> {
    bool_component_state_value(state, "disabled")
        .or_else(|| bool_component_state_value(state, "enabled").map(|enabled| !enabled))
        .or_else(|| state.flags.disabled.then_some(true))
}

fn bool_component_state_value(state: &UiComponentState, property: &str) -> Option<bool> {
    match state.value(property) {
        Some(UiValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn bool_attribute_value(
    values: &std::collections::BTreeMap<String, toml::Value>,
    key: &str,
) -> Option<bool> {
    values.get(key).and_then(toml::Value::as_bool)
}
