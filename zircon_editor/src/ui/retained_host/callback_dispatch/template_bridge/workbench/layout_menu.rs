use crate::core::editor_event::MenuAction;
use crate::ui::binding::EditorUiBinding;
use crate::ui::workbench::event::menu_action_binding;
use zircon_runtime_interface::ui::component::UiValue;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const LAYOUT_MENU_CONTROL_ID: &str = "WorkbenchLayoutMenu";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
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
        for property in ["value", "value_text"] {
            self.mutate_control_property(
                LAYOUT_MENU_CONTROL_ID,
                property,
                UiValue::String("Default Layout".to_string()),
            )?;
        }
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(())
    }
}
