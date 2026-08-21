use zircon_runtime::ui::surface::UiPropertyMutationRequest;
use zircon_runtime_interface::ui::{component::UiValue, event_ui::UiNodeId};

use super::super::popup_primitives::toml_value_string_list;
use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn set_control_active(
        &mut self,
        control_id: &str,
        selected: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let node_ids = self.control_node_ids_with_descendants(control_id);
        if node_ids.is_empty() {
            return Ok(());
        }
        let value = UiValue::Bool(selected);
        for node_id in &node_ids {
            self.mutate_node_bool(*node_id, "selected", selected)?;
            self.mutate_node_bool(*node_id, "checked", selected)?;
        }
        let _ = self
            .template_surface
            .surface
            .mutate_property(UiPropertyMutationRequest::new(node_ids[0], "value", value))?;
        Ok(())
    }

    pub(super) fn set_selected(
        &mut self,
        control_id: &str,
        selected: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let node_ids = self.control_node_ids_with_descendants(control_id);
        if node_ids.is_empty() {
            return Ok(());
        }
        for node_id in node_ids {
            self.mutate_node_bool(node_id, "selected", selected)?;
        }
        Ok(())
    }

    pub(super) fn toggle_checked(
        &mut self,
        control_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let node_ids = self.control_node_ids_with_descendants(control_id);
        if node_ids.is_empty() {
            return Ok(());
        }
        let checked = !self.control_bool(control_id, "checked");
        for node_id in &node_ids {
            self.mutate_node_bool(*node_id, "checked", checked)?;
            self.mutate_node_bool(*node_id, "selected", checked)?;
        }
        let _ = self
            .template_surface
            .surface
            .mutate_property(UiPropertyMutationRequest::new(
                node_ids[0],
                "value",
                UiValue::Bool(checked),
            ))?;
        Ok(())
    }

    pub(super) fn set_visible(
        &mut self,
        control_id: &str,
        visible: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let Some(node_id) = self.control_node_id(control_id) else {
            return Ok(());
        };
        let visibility = if visible { "visible" } else { "collapsed" };
        let _ = self
            .template_surface
            .surface
            .mutate_property(UiPropertyMutationRequest::new(
                node_id,
                "visibility",
                UiValue::String(visibility.to_string()),
            ))?;
        if visible {
            let _ =
                self.template_surface
                    .surface
                    .mutate_property(UiPropertyMutationRequest::new(
                        node_id,
                        "visible",
                        UiValue::Bool(true),
                    ))?;
        }
        Ok(())
    }

    pub(super) fn cycle_string_property(
        &mut self,
        control_id: &str,
        property: &str,
        values: &[&str],
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if values.is_empty() {
            return Ok(());
        }
        let Some(node_id) = self.control_node_id(control_id) else {
            return Ok(());
        };
        let current = self
            .control_string(control_id, property)
            .unwrap_or_else(|| values[0].to_string());
        let current_index = values
            .iter()
            .position(|value| *value == current)
            .unwrap_or(0);
        let next = values[(current_index + 1) % values.len()];
        let _ = self
            .template_surface
            .surface
            .mutate_property(UiPropertyMutationRequest::new(
                node_id,
                property,
                UiValue::String(next.to_string()),
            ))?;
        Ok(())
    }

    pub(super) fn mutate_control_property(
        &mut self,
        control_id: &str,
        property: &str,
        value: UiValue,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let Some(node_id) = self.control_node_id(control_id) else {
            return Ok(());
        };
        let _ = self
            .template_surface
            .surface
            .mutate_property(UiPropertyMutationRequest::new(node_id, property, value))?;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn mutate_control_property_for_test(
        &mut self,
        control_id: &str,
        property: &str,
        value: UiValue,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(control_id, property, value)
    }

    pub(super) fn mutate_node_bool(
        &mut self,
        node_id: UiNodeId,
        property: &str,
        value: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let _ = self
            .template_surface
            .surface
            .mutate_property(UiPropertyMutationRequest::new(
                node_id,
                property,
                UiValue::Bool(value),
            ))
            .map_err(
                |source| BuiltinHostWindowTemplateBridgeError::LayoutMutation {
                    node_id,
                    property: property.to_string(),
                    source,
                },
            )?;
        Ok(())
    }

    pub(super) fn control_bool(&self, control_id: &str, property: &str) -> bool {
        let Some(node_id) = self.control_node_id(control_id) else {
            return false;
        };
        self.template_surface
            .surface
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.attributes.get(property))
                    .and_then(toml::Value::as_bool)
            })
            .unwrap_or(false)
    }

    pub(super) fn control_integer(&self, control_id: &str, property: &str) -> Option<i64> {
        let node_id = self.control_node_id(control_id)?;
        self.template_surface
            .surface
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.attributes.get(property))
                    .and_then(toml::Value::as_integer)
            })
    }

    pub(super) fn control_string(&self, control_id: &str, property: &str) -> Option<String> {
        let node_id = self.control_node_id(control_id)?;
        self.template_surface
            .surface
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.attributes.get(property))
                    .and_then(toml::Value::as_str)
                    .map(str::to_string)
            })
    }

    pub(super) fn control_string_array(&self, control_id: &str, property: &str) -> Vec<String> {
        let Some(node_id) = self.control_node_id(control_id) else {
            return Vec::new();
        };
        self.template_surface
            .surface
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.attributes.get(property))
                    .map(toml_value_string_list)
            })
            .unwrap_or_default()
    }

    fn control_node_ids_with_descendants(&self, control_id: &str) -> Vec<UiNodeId> {
        let Some(root_id) = self.control_node_id(control_id) else {
            return Vec::new();
        };

        let mut node_ids = Vec::new();
        let mut stack = vec![root_id];
        while let Some(node_id) = stack.pop() {
            node_ids.push(node_id);
            if let Some(node) = self.template_surface.surface.tree.nodes.get(&node_id) {
                stack.extend(node.children.iter().rev().copied());
            }
        }
        node_ids
    }
}
