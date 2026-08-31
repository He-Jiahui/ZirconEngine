use crate::core::editor_event::MenuAction;
use crate::ui::binding::EditorUiBinding;
use crate::ui::workbench::event::menu_action_binding;
use zircon_runtime_interface::ui::component::UiValue;

use super::super::popup_primitives::{
    menu_item_with_checked_state, template_popup_menu_item_state,
};
use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const LAYOUT_MENU_CONTROL_ID: &str = "WorkbenchLayoutMenu";
const DEFAULT_LAYOUT_ACTION_ID: &str = "menu.item.default_layout";
const DEFAULT_LAYOUT_LABEL: &str = "Default Layout";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_layout_menu_indicator(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let menu_items = self.control_string_array(LAYOUT_MENU_CONTROL_ID, "menu_items");
        let menu_items = UiValue::Array(
            menu_items
                .iter()
                .map(|raw| {
                    let checked = template_popup_menu_item_state(raw)
                        .is_some_and(|item| item.action_id == DEFAULT_LAYOUT_ACTION_ID);
                    UiValue::String(menu_item_with_checked_state(raw, checked))
                })
                .collect(),
        );
        for property in ["value", "value_text"] {
            self.mutate_control_property(
                LAYOUT_MENU_CONTROL_ID,
                property,
                UiValue::String(DEFAULT_LAYOUT_LABEL.to_string()),
            )?;
        }
        self.mutate_control_property(LAYOUT_MENU_CONTROL_ID, "menu_items", menu_items)?;
        Ok(())
    }

    pub(crate) fn layout_menu_item_binding(
        &self,
        menu_control_id: &str,
        action_id: &str,
    ) -> Option<EditorUiBinding> {
        if menu_control_id != LAYOUT_MENU_CONTROL_ID || action_id != "menu.item.reset_layout" {
            return None;
        }
        Some(menu_action_binding(&MenuAction::ResetLayout))
    }

    pub(crate) fn restore_layout_menu_indicator(
        &mut self,
        menu_control_id: &str,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if menu_control_id != LAYOUT_MENU_CONTROL_ID || action_id != "menu.item.reset_layout" {
            return Ok(());
        }
        self.initialize_layout_menu_indicator()?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(())
    }
}
