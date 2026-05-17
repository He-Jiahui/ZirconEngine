use zircon_runtime_interface::ui::{
    component::{UiComponentState, UiValue},
    event_ui::UiNodeId,
    tree::{UiTemplateNodeMetadata, UiTreeError, UiTreeNode},
};

use crate::ui::tree::UiRuntimeTreeAccessExt;

use super::UiSurface;

impl UiSurface {
    pub(super) fn node_interaction_enabled(&self, node_id: UiNodeId) -> Result<bool, UiTreeError> {
        let node = self
            .tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        Ok(
            self.node_interaction_enabled_from_parts(
                node_id,
                node,
                node.template_metadata.as_ref(),
            ),
        )
    }

    pub(super) fn widget_interaction_enabled(
        &self,
        node_id: UiNodeId,
        node: &UiTreeNode,
        metadata: &UiTemplateNodeMetadata,
    ) -> bool {
        self.node_interaction_enabled_from_parts(node_id, node, Some(metadata))
    }

    fn node_interaction_enabled_from_parts(
        &self,
        node_id: UiNodeId,
        node: &UiTreeNode,
        metadata: Option<&UiTemplateNodeMetadata>,
    ) -> bool {
        node.state_flags.enabled
            && !self
                .component_states
                .get(node_id)
                .and_then(disabled_component_state_value)
                .unwrap_or(false)
            && !metadata.is_some_and(|metadata| {
                metadata.widget.disabled
                    || bool_attribute_value(&metadata.attributes, "disabled") == Some(true)
            })
    }
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
