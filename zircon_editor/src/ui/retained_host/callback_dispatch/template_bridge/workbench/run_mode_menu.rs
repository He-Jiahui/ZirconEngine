use crate::core::editor_event::MenuAction;
use crate::core::play::PlayKind;
use crate::ui::binding::EditorUiBinding;
use crate::ui::workbench::event::menu_action_binding;
use zircon_runtime_interface::ui::component::UiValue;

use super::super::popup_primitives::{
    menu_item_with_checked_state, template_popup_menu_item_state,
};
use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const RUN_MODE_MENU_CONTROL_ID: &str = "WorkbenchRunModeMenu";
const RUN_MODE_TRIGGER_CONTROL_ID: &str = "WorkbenchRunMode";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_run_mode_menu_indicator(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.sync_run_mode_menu_for_trigger(RUN_MODE_TRIGGER_CONTROL_ID, PlayKind::Play)
    }

    pub(crate) fn owns_run_mode_menu_trigger(&self, control_id: &str) -> bool {
        control_id == RUN_MODE_TRIGGER_CONTROL_ID
    }

    pub(crate) fn run_mode_menu_item_binding(
        &self,
        menu_control_id: &str,
        action_id: &str,
    ) -> Option<EditorUiBinding> {
        if menu_control_id != RUN_MODE_MENU_CONTROL_ID {
            return None;
        }
        let kind = match action_id {
            "menu.item.play_in_editor" => PlayKind::Play,
            "menu.item.simulate" => PlayKind::Simulate,
            _ => return None,
        };
        Some(menu_action_binding(&MenuAction::SelectPlayMode(kind)))
    }

    pub(crate) fn sync_run_mode_menu_for_trigger(
        &mut self,
        trigger_control_id: &str,
        kind: PlayKind,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if !self.owns_run_mode_menu_trigger(trigger_control_id) {
            return Ok(());
        }
        let (selected_action_id, label) = run_mode_choice(kind);
        let menu_items = self.control_string_array(RUN_MODE_MENU_CONTROL_ID, "menu_items");
        let menu_items = UiValue::Array(
            menu_items
                .iter()
                .map(|raw| {
                    let checked = template_popup_menu_item_state(raw)
                        .is_some_and(|item| item.action_id == selected_action_id);
                    UiValue::String(menu_item_with_checked_state(raw, checked))
                })
                .collect(),
        );
        self.mutate_control_property(
            RUN_MODE_MENU_CONTROL_ID,
            "value",
            UiValue::String(label.to_string()),
        )?;
        self.mutate_control_property(
            RUN_MODE_MENU_CONTROL_ID,
            "value_text",
            UiValue::String(label.to_string()),
        )?;
        self.mutate_control_property(RUN_MODE_MENU_CONTROL_ID, "menu_items", menu_items)?;
        Ok(())
    }
}

fn run_mode_choice(kind: PlayKind) -> (&'static str, &'static str) {
    match kind {
        PlayKind::Play => ("menu.item.play_in_editor", "Play In Editor"),
        PlayKind::Simulate => ("menu.item.simulate", "Simulate"),
    }
}
