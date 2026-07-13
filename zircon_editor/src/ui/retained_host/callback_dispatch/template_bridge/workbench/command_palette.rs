use zircon_runtime_interface::ui::component::UiValue;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

pub(crate) const WORKBENCH_COMMAND_PALETTE_CONTROL_ID: &str = "WorkbenchCommandPalette";

const OPEN: &str = "open";
const POPUP_OPEN: &str = "popup_open";
const FOCUSED: &str = "focused";
const SELECTED: &str = "selected";
const QUERY: &str = "query";
const COMMANDS: &str = "commands";
const FILTERED_COMMANDS: &str = "filtered_commands";
const SELECTED_COMMAND_ID: &str = "selected_command_id";
const FOCUSED_INDEX: &str = "focused_index";
const COMMAND_SOURCE: &str = "command_source";

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkbenchCommandPaletteOpenState {
    pub commands: UiValue,
    pub filtered_commands: UiValue,
    pub selected_command_id: String,
    pub focused_index: i64,
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn open_command_palette(
        &mut self,
        state: WorkbenchCommandPaletteOpenState,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.has_control(WORKBENCH_COMMAND_PALETTE_CONTROL_ID) {
            return Ok(false);
        }

        self.set_visible(WORKBENCH_COMMAND_PALETTE_CONTROL_ID, true)?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            OPEN,
            UiValue::Bool(true),
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            POPUP_OPEN,
            UiValue::Bool(true),
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            FOCUSED,
            UiValue::Bool(true),
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            SELECTED,
            UiValue::Bool(true),
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            QUERY,
            UiValue::String(String::new()),
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            COMMAND_SOURCE,
            UiValue::String(String::new()),
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            COMMANDS,
            state.commands,
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            FILTERED_COMMANDS,
            state.filtered_commands,
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            SELECTED_COMMAND_ID,
            UiValue::String(state.selected_command_id),
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            FOCUSED_INDEX,
            UiValue::Int(state.focused_index),
        )?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    pub(crate) fn close_command_palette(
        &mut self,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.command_palette_open() {
            return Ok(false);
        }

        self.set_visible(WORKBENCH_COMMAND_PALETTE_CONTROL_ID, false)?;
        for property in [OPEN, POPUP_OPEN, FOCUSED, SELECTED] {
            self.mutate_control_property(
                WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
                property,
                UiValue::Bool(false),
            )?;
        }
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    pub(crate) fn command_palette_open(&self) -> bool {
        self.control_bool(WORKBENCH_COMMAND_PALETTE_CONTROL_ID, OPEN)
            || self.control_bool(WORKBENCH_COMMAND_PALETTE_CONTROL_ID, POPUP_OPEN)
    }
}
