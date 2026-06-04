use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{component::UiValue, event_ui::UiNodeId};

use super::super::popup_primitives::{
    menu_item_without_transient_flags, string_array_value, template_popup_menu_item_state,
    toml_value_string_list,
};
use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn select_dropdown_option(
        &mut self,
        control_id: &str,
        option_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let options = control_string_array(&self.template_surface.surface, control_id, "options")
            .unwrap_or_default();
        if !options.iter().any(|option| option == option_id) {
            return Ok(false);
        }
        let disabled = control_string_array(
            &self.template_surface.surface,
            control_id,
            "disabled_options",
        )
        .unwrap_or_default();
        if disabled.iter().any(|option| option == option_id) {
            return Ok(false);
        }

        self.mutate_control_property(control_id, "value", UiValue::String(option_id.to_string()))?;
        self.mutate_control_property(
            control_id,
            "value_text",
            UiValue::String(option_id.to_string()),
        )?;
        self.mutate_control_property(
            control_id,
            "special_options",
            string_array_value([option_id]),
        )?;
        for property in ["focused_options", "hovered_options", "pressed_options"] {
            self.mutate_control_property(control_id, property, UiValue::Array(Vec::new()))?;
        }
        for node_id in control_node_ids_with_descendants(&self.template_surface.surface, control_id)
        {
            self.mutate_node_bool(node_id, "popup_open", false)?;
            self.mutate_node_bool(node_id, "focused", false)?;
            self.mutate_node_bool(node_id, "selected", false)?;
        }
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    pub(crate) fn select_popup_menu_item(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) -> Result<Option<bool>, BuiltinHostWindowTemplateBridgeError> {
        let menu_items =
            control_string_array(&self.template_surface.surface, control_id, "menu_items")
                .unwrap_or_default();
        if menu_items.is_empty() {
            return Ok(None);
        }
        let Some(selected_item) = menu_items
            .iter()
            .filter_map(|raw| template_popup_menu_item_state(raw))
            .find(|item| item.action_id == action_id)
        else {
            return Ok(None);
        };
        if selected_item.disabled || selected_item.separator {
            return Ok(Some(false));
        }

        let normalized_items = UiValue::Array(
            menu_items
                .iter()
                .map(|raw| UiValue::String(menu_item_without_transient_flags(raw)))
                .collect(),
        );
        self.mutate_control_property(
            control_id,
            "value",
            UiValue::String(selected_item.label.clone()),
        )?;
        self.mutate_control_property(
            control_id,
            "value_text",
            UiValue::String(selected_item.label),
        )?;
        self.mutate_control_property(control_id, "menu_items", normalized_items)?;
        for node_id in control_node_ids_with_descendants(&self.template_surface.surface, control_id)
        {
            self.mutate_node_bool(node_id, "popup_open", false)?;
            self.mutate_node_bool(node_id, "focused", false)?;
            self.mutate_node_bool(node_id, "selected", false)?;
        }
        self.close_workbench_window_menu_control(control_id)?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(Some(true))
    }

    pub(super) fn toggle_popup(
        &mut self,
        control_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let node_ids =
            control_node_ids_with_descendants(&self.template_surface.surface, control_id);
        if node_ids.is_empty() {
            return Ok(());
        }
        let open = !control_has_open_popup(&self.template_surface.surface, control_id);
        for node_id in node_ids {
            self.mutate_node_bool(node_id, "popup_open", open)?;
            self.mutate_node_bool(node_id, "focused", open)?;
            self.mutate_node_bool(node_id, "selected", open)?;
        }
        Ok(())
    }

    pub(crate) fn close_popup(
        &mut self,
        control_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !control_has_open_popup(&self.template_surface.surface, control_id) {
            return Ok(false);
        }

        for property in ["focused_options", "hovered_options", "pressed_options"] {
            self.mutate_control_property(control_id, property, UiValue::Array(Vec::new()))?;
        }
        if let Some(menu_items) =
            control_string_array(&self.template_surface.surface, control_id, "menu_items")
        {
            self.mutate_control_property(
                control_id,
                "menu_items",
                UiValue::Array(
                    menu_items
                        .iter()
                        .map(|raw| UiValue::String(menu_item_without_transient_flags(raw)))
                        .collect(),
                ),
            )?;
        }

        for node_id in control_node_ids_with_descendants(&self.template_surface.surface, control_id)
        {
            self.mutate_node_bool(node_id, "popup_open", false)?;
            self.mutate_node_bool(node_id, "focused", false)?;
            self.mutate_node_bool(node_id, "selected", false)?;
        }
        self.close_workbench_window_menu_control(control_id)?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }
}

fn control_has_open_popup(surface: &UiSurface, control_id: &str) -> bool {
    surface.tree.nodes.values().any(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .and_then(|metadata| metadata.attributes.get("popup_open"))
            .and_then(toml::Value::as_bool)
            .unwrap_or(false)
    })
}

fn control_string_array(
    surface: &UiSurface,
    control_id: &str,
    property: &str,
) -> Option<Vec<String>> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .and_then(|metadata| metadata.attributes.get(property))
            .map(toml_value_string_list)
    })
}

fn control_node_id(surface: &UiSurface, control_id: &str) -> Option<UiNodeId> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.node_id)
    })
}

fn control_node_ids_with_descendants(surface: &UiSurface, control_id: &str) -> Vec<UiNodeId> {
    let Some(root_id) = control_node_id(surface, control_id) else {
        return Vec::new();
    };

    let mut node_ids = Vec::new();
    let mut stack = vec![root_id];
    while let Some(node_id) = stack.pop() {
        node_ids.push(node_id);
        if let Some(node) = surface.tree.nodes.get(&node_id) {
            for child_id in node.children.iter().rev() {
                stack.push(*child_id);
            }
        }
    }
    node_ids
}
