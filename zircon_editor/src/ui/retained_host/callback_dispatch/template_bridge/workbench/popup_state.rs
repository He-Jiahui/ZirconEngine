use zircon_runtime_interface::ui::component::UiValue;

use crate::ui::retained_host::option_spec::parse_retained_option;

use super::super::popup_primitives::{
    menu_item_with_checked_state, menu_item_without_transient_flags, string_array_value,
    template_popup_menu_item_state,
};
use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::context_menu::WORKBENCH_CONTEXT_MENU_CONTROL_ID;
use super::error::BuiltinHostWindowTemplateBridgeError;
use super::settings_window::WORKBENCH_SETTINGS_WINDOW_CONTROL_ID;

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn select_dropdown_option(
        &mut self,
        control_id: &str,
        option_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let options = self.control_string_array(control_id, "options");
        let Some(selected_option) = options
            .iter()
            .map(|option| parse_retained_option(option))
            .find(|option| option.matches_id(option_id))
        else {
            return Ok(false);
        };
        let disabled = self.control_string_array(control_id, "disabled_options");
        if disabled
            .iter()
            .any(|option| selected_option.matches_id(option))
        {
            return Ok(false);
        }

        self.mutate_control_property(
            control_id,
            "value",
            UiValue::String(selected_option.id.clone()),
        )?;
        self.mutate_control_property(
            control_id,
            "value_text",
            UiValue::String(selected_option.label),
        )?;
        self.mutate_control_property(
            control_id,
            "special_options",
            string_array_value([selected_option.id.as_str()]),
        )?;
        for property in ["focused_options", "hovered_options", "pressed_options"] {
            self.mutate_control_property(control_id, property, UiValue::Array(Vec::new()))?;
        }
        for node_id in self.control_node_ids_with_descendants(control_id) {
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
        let menu_items = self.control_string_array(control_id, "menu_items");
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

        let is_choice_menu = self
            .control_string(control_id, "selection_mode")
            .is_some_and(|mode| mode.eq_ignore_ascii_case("single"));
        let normalized_items = UiValue::Array(
            menu_items
                .iter()
                .map(|raw| {
                    let normalized = menu_item_without_transient_flags(raw);
                    let normalized = if is_choice_menu {
                        let checked = template_popup_menu_item_state(raw)
                            .is_some_and(|item| item.action_id == selected_item.action_id);
                        menu_item_with_checked_state(&normalized, checked)
                    } else {
                        normalized
                    };
                    UiValue::String(normalized)
                })
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
        for node_id in self.control_node_ids_with_descendants(control_id) {
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
        let node_ids = self.control_node_ids_with_descendants(control_id);
        if node_ids.is_empty() {
            return Ok(());
        }
        let open = !self.control_bool(control_id, "popup_open");
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
        if control_id == WORKBENCH_SETTINGS_WINDOW_CONTROL_ID {
            return self.close_settings_window();
        }
        if !self.control_bool(control_id, "popup_open") {
            return Ok(false);
        }

        for property in ["focused_options", "hovered_options", "pressed_options"] {
            self.mutate_control_property(control_id, property, UiValue::Array(Vec::new()))?;
        }
        let menu_items = self.control_string_array(control_id, "menu_items");
        if !menu_items.is_empty() {
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

        for node_id in self.control_node_ids_with_descendants(control_id) {
            self.mutate_node_bool(node_id, "popup_open", false)?;
            self.mutate_node_bool(node_id, "focused", false)?;
            self.mutate_node_bool(node_id, "selected", false)?;
        }
        self.close_workbench_window_menu_control(control_id)?;
        if control_id == WORKBENCH_CONTEXT_MENU_CONTROL_ID {
            self.close_context_menu_if_target(control_id)?;
        }
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }
}

#[cfg(test)]
#[path = "tests/popup_state.rs"]
mod tests;
