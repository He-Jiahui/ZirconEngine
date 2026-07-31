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
const CATALOG_GENERATION: &str = "catalog_generation";
const MATCH_COUNT: &str = "match_count";
const WINDOW_COUNT: &str = "window_count";
const WINDOW_OFFSET: &str = "window_offset";
const PAGE_SIZE: &str = "page_size";
const VIRTUALIZATION_ENABLED: &str = "virtualization_enabled";
const VIRTUALIZATION_TOTAL_COUNT: &str = "total_count";
const VIRTUALIZATION_VISIBLE_START: &str = "viewport_start";
const VIRTUALIZATION_VISIBLE_COUNT: &str = "viewport_count";
const WINDOW_REQUEST_CURRENT_OFFSET: &str = "window_request_current_offset";
const WINDOW_REQUEST_OFFSET: &str = "window_request_offset";
const WINDOW_REQUEST_FOCUS: &str = "window_request_focus";
const WINDOW_REQUEST_GENERATION: &str = "window_request_generation";
const DEFAULT_WINDOW_COUNT: usize = 12;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkbenchCommandPaletteOpenState {
    pub query: String,
    pub commands: UiValue,
    pub filtered_commands: UiValue,
    pub selected_command_id: String,
    pub focused_index: i64,
    pub catalog_generation: u64,
    pub total_match_count: usize,
    pub window_offset: usize,
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
            COMMAND_SOURCE,
            UiValue::String(String::new()),
        )?;
        self.apply_command_palette_query_state(state)?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    pub(crate) fn update_command_palette_query(
        &mut self,
        state: WorkbenchCommandPaletteOpenState,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.command_palette_open() {
            return Ok(false);
        }
        self.apply_command_palette_query_state(state)?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    fn apply_command_palette_query_state(
        &mut self,
        state: WorkbenchCommandPaletteOpenState,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let visible_count = match &state.filtered_commands {
            UiValue::Array(commands) => commands.len(),
            _ => 0,
        };
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            QUERY,
            UiValue::String(state.query),
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
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            CATALOG_GENERATION,
            UiValue::Int(saturating_i64(state.catalog_generation)),
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            MATCH_COUNT,
            UiValue::Int(saturating_i64(state.total_match_count)),
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            WINDOW_OFFSET,
            UiValue::Int(saturating_i64(state.window_offset)),
        )?;
        for (property, value) in [
            (WINDOW_COUNT, DEFAULT_WINDOW_COUNT),
            (PAGE_SIZE, DEFAULT_WINDOW_COUNT),
            (VIRTUALIZATION_TOTAL_COUNT, state.total_match_count),
            (VIRTUALIZATION_VISIBLE_START, state.window_offset),
            (VIRTUALIZATION_VISIBLE_COUNT, visible_count),
        ] {
            self.mutate_control_property(
                WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
                property,
                UiValue::Int(saturating_i64(value)),
            )?;
        }
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            VIRTUALIZATION_ENABLED,
            UiValue::Bool(true),
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            WINDOW_REQUEST_CURRENT_OFFSET,
            UiValue::Int(saturating_i64(state.window_offset)),
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            WINDOW_REQUEST_OFFSET,
            UiValue::Int(-1),
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            WINDOW_REQUEST_FOCUS,
            UiValue::String(String::new()),
        )?;
        self.mutate_control_property(
            WORKBENCH_COMMAND_PALETTE_CONTROL_ID,
            WINDOW_REQUEST_GENERATION,
            UiValue::Int(saturating_i64(state.catalog_generation)),
        )?;
        Ok(())
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

    pub(crate) fn command_palette_query(&self) -> String {
        self.control_string(WORKBENCH_COMMAND_PALETTE_CONTROL_ID, QUERY)
            .unwrap_or_default()
    }

    pub(crate) fn command_palette_catalog_generation(&self) -> Option<u64> {
        self.command_palette_integer(CATALOG_GENERATION)
            .and_then(|generation| u64::try_from(generation).ok())
    }

    pub(crate) fn command_palette_window_offset(&self) -> Option<usize> {
        self.command_palette_integer(WINDOW_OFFSET)
            .and_then(|offset| usize::try_from(offset).ok())
    }

    fn command_palette_integer(&self, property: &str) -> Option<i64> {
        self.template_surface
            .surface
            .tree
            .nodes
            .values()
            .find_map(|node| {
                node.template_metadata
                    .as_ref()
                    .filter(|metadata| {
                        metadata.control_id.as_deref() == Some(WORKBENCH_COMMAND_PALETTE_CONTROL_ID)
                    })
                    .and_then(|metadata| metadata.attributes.get(property))
                    .and_then(toml::Value::as_integer)
            })
    }
}

fn saturating_i64(value: impl TryInto<i64>) -> i64 {
    value.try_into().unwrap_or(i64::MAX)
}
