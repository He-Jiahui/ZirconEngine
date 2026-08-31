use crate::ui::retained_host::workbench_preview_actions::is_workbench_preview_action;

use super::{
    blend_space_search::is_blend_space_search_action,
    blend_space_transport::is_animation_transport_action,
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
    extension_module_navigation::{
        is_workbench_extension_action, workbench_extension_panel_command_control_id,
        workbench_extension_panel_row_control_id, workbench_extension_panel_row_group,
        workbench_extension_workspace_control_id,
    },
    generated_bottom_panel_navigation::is_workbench_generated_bottom_action,
    inspector_filter::is_inspector_filter_action,
    module_navigation::{
        is_workbench_module_action, workbench_module_command_control_id,
        workbench_module_panel_command_control_id, workbench_module_panel_row_control_id,
        workbench_module_panel_row_group, workbench_module_tab_control_id, MODULE_TAB_CONTROLS,
    },
};

const RADIO_CONTROLS: &[&str] = &["WorkbenchRadioOn", "WorkbenchRadioOff"];
const LABS_TAB_CONTROLS: &[&str] = &[
    "WorkbenchLabsTabOne",
    "WorkbenchLabsTabTwo",
    "WorkbenchLabsTabThree",
];
const LIST_CONTROLS: &[&str] = &["WorkbenchListItem", "WorkbenchListSelected"];
const TABLE_CONTROLS: &[&str] = &[
    "WorkbenchTableItem",
    "WorkbenchTableSelected",
    "WorkbenchTableTail",
];
const PANEL_COMPONENT_DRAWER_TAB_CONTROLS: &[&str] =
    &["WorkbenchDrawerTabComponents", "WorkbenchDrawerTabConsole"];
const BLEND_SPACE_VALIDATION_FILTER_CONTROLS: &[&str] = &[
    "WorkbenchValidationLogAll",
    "WorkbenchValidationLogErrors",
    "WorkbenchValidationLogWarnings",
    "WorkbenchValidationLogInfos",
];
const BLEND_SPACE_VALIDATION_ROWS: &[&str] = &[
    "WorkbenchValidationLogInfoAxesRow",
    "WorkbenchValidationLogWarningRow",
    "WorkbenchValidationLogInfoRangeRow",
    "WorkbenchValidationLogInfoDuplicatesRow",
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_panel_live_control_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive(
            PANEL_COMPONENT_DRAWER_TAB_CONTROLS,
            "WorkbenchDrawerTabComponents",
        )?;
        self.select_exclusive(
            BLEND_SPACE_VALIDATION_FILTER_CONTROLS,
            "WorkbenchValidationLogAll",
        )?;
        self.select_generated_bottom_mode("WorkbenchGeneratedBottomModeOutput")?;
        self.initialize_ability_workspace_state()?;
        self.initialize_assets_workspace_state()?;
        self.initialize_behavior_workspace_state()?;
        self.initialize_effect_workspace_state()?;
        self.initialize_hud_workspace_state()?;
        self.initialize_material_workspace_state()?;
        self.initialize_perception_workspace_state()?;
        self.initialize_render_workspace_state()?;
        self.initialize_tags_workspace_state()?;
        self.initialize_vfx_workspace_state()?;
        self.set_selected("WorkbenchScenePropsItem", true)?;
        self.set_visible("WorkbenchComponentDrawerBody", true)?;
        self.set_visible("WorkbenchComponentDrawerConsoleBody", false)?;
        Ok(())
    }

    pub(super) fn apply_reference_menu_action(
        &mut self,
        source_control_id: &str,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if self.apply_workbench_window_menu_action(source_control_id, action_id)? {
            return Ok(());
        }
        match action_id {
            "workbench.module.details_drawer.toggle" => {
                self.toggle_compact_module_details_drawer()?;
            }
            "component_lab.input_dropdown.open" | "component_lab.button_dropdown.open" => {
                self.toggle_popup(source_control_id)?;
            }
            "component_lab.input_segment.select" => {
                self.cycle_string_property(
                    "WorkbenchInputSegmented",
                    "value",
                    &["left", "center", "right"],
                )?;
                self.set_selected("WorkbenchInputSegmented", true)?;
            }
            "component_lab.icon_toggle_segment.select" => {
                self.cycle_string_property(
                    "WorkbenchIconToggleSegmented",
                    "value",
                    &["grid", "list", "columns"],
                )?;
                self.set_selected("WorkbenchIconToggleSegmented", true)?;
            }
            "component_lab.labs_tab_one.select" => {
                self.select_exclusive(LABS_TAB_CONTROLS, "WorkbenchLabsTabOne")?;
            }
            "component_lab.labs_tab_two.select" => {
                self.select_exclusive(LABS_TAB_CONTROLS, "WorkbenchLabsTabTwo")?;
            }
            "component_lab.labs_tab_three.select" => {
                self.select_exclusive(LABS_TAB_CONTROLS, "WorkbenchLabsTabThree")?;
            }
            "component_lab.checkbox_on.toggle" => {
                self.toggle_checked("WorkbenchCheckboxOn")?;
            }
            "component_lab.checkbox_off.toggle" => {
                self.toggle_checked("WorkbenchCheckboxOff")?;
            }
            "component_lab.radio_on.select" => {
                self.select_exclusive(RADIO_CONTROLS, "WorkbenchRadioOn")?;
            }
            "component_lab.radio_off.select" => {
                self.select_exclusive(RADIO_CONTROLS, "WorkbenchRadioOff")?;
            }
            "component_lab.switch.toggle" => {
                self.toggle_checked("WorkbenchToggleOn")?;
            }
            "component_lab.list_item.select" => {
                self.select_exclusive_selected(LIST_CONTROLS, "WorkbenchListItem")?;
            }
            "component_lab.list_selected.select" => {
                self.select_exclusive_selected(LIST_CONTROLS, "WorkbenchListSelected")?;
            }
            "component_lab.table_item.select" => {
                self.select_exclusive_selected(TABLE_CONTROLS, "WorkbenchTableItem")?;
            }
            "component_lab.table_selected.select" => {
                self.select_exclusive_selected(TABLE_CONTROLS, "WorkbenchTableSelected")?;
            }
            "component_lab.table_tail.select" => {
                self.select_exclusive_selected(TABLE_CONTROLS, "WorkbenchTableTail")?;
            }
            "component_drawer.components_tab.select" => {
                self.select_exclusive(
                    PANEL_COMPONENT_DRAWER_TAB_CONTROLS,
                    "WorkbenchDrawerTabComponents",
                )?;
                self.set_visible("WorkbenchComponentDrawerBody", true)?;
                self.set_visible("WorkbenchComponentDrawerConsoleBody", false)?;
            }
            "component_drawer.console_tab.select" => {
                self.select_exclusive(
                    PANEL_COMPONENT_DRAWER_TAB_CONTROLS,
                    "WorkbenchDrawerTabConsole",
                )?;
                self.set_visible("WorkbenchComponentDrawerBody", false)?;
                self.set_visible("WorkbenchComponentDrawerConsoleBody", true)?;
            }
            action_id if is_workbench_module_action(action_id) => {
                self.apply_workbench_module_action(source_control_id, action_id)?;
            }
            action_id if is_animation_transport_action(action_id) => {
                self.apply_blend_space_transport_action(action_id)?;
            }
            action_id if is_blend_space_search_action(action_id) => {
                self.apply_blend_space_search_action(action_id)?;
            }
            action_id if is_inspector_filter_action(action_id) => {
                self.apply_inspector_filter_action(action_id)?;
            }
            action_id if is_workbench_extension_action(action_id) => {
                self.apply_workbench_extension_action(source_control_id, action_id)?;
            }
            action_id if is_workbench_generated_bottom_action(action_id) => {
                self.apply_workbench_generated_bottom_action(source_control_id, action_id)?;
            }
            action_id if is_workbench_preview_action(action_id) => {}
            _ => {}
        }
        Ok(())
    }

    fn apply_workbench_module_action(
        &mut self,
        source_control_id: &str,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if self.apply_ability_workspace_action(action_id)? {
            return Ok(());
        }
        if self.apply_assets_workspace_action(action_id)? {
            return Ok(());
        }
        if self.apply_behavior_workspace_action(action_id)? {
            return Ok(());
        }
        if self.apply_effect_workspace_action(action_id)? {
            return Ok(());
        }
        if self.apply_hud_workspace_action(action_id)? {
            return Ok(());
        }
        if self.apply_material_workspace_action(action_id)? {
            return Ok(());
        }
        if self.apply_perception_workspace_action(action_id)? {
            return Ok(());
        }
        if self.apply_render_workspace_action(action_id)? {
            return Ok(());
        }
        if self.apply_tags_workspace_action(action_id)? {
            return Ok(());
        }
        if self.apply_vfx_workspace_action(action_id)? {
            return Ok(());
        }
        if let Some(control_id) = workbench_module_tab_control_id(action_id) {
            self.select_exclusive(MODULE_TAB_CONTROLS, control_id)?;
            self.apply_workbench_module_workspace(action_id)?;
        } else if workbench_module_command_control_id(action_id).is_some() {
            if action_id == "workbench.module.browse.invoke" {
                self.select_exclusive(MODULE_TAB_CONTROLS, "WorkbenchModuleAssets")?;
                self.apply_workbench_module_workspace("workbench.module.assets.select")?;
            }
        } else if let Some(control_id) = workbench_module_panel_row_control_id(action_id) {
            self.select_exclusive_selected(
                workbench_module_panel_row_group(action_id),
                control_id,
            )?;
        } else if workbench_module_panel_command_control_id(action_id).is_some() {
            return self.apply_workbench_module_command_feedback(action_id);
        } else if self.should_open_dropdown_for_module_field_action(source_control_id, action_id) {
            self.toggle_popup(source_control_id)?;
        }
        self.apply_workbench_module_command_feedback(action_id)
    }

    fn apply_workbench_extension_action(
        &mut self,
        source_control_id: &str,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if self.apply_blend_space_validation_action(action_id)? {
            return self.apply_workbench_extension_module_command_feedback(action_id);
        }
        if self.apply_blend_space_asset_selection_action(action_id)? {
            return self.apply_workbench_extension_module_command_feedback(action_id);
        }
        if self.apply_blend_space_sample_selection_action(action_id)? {
            return Ok(());
        }
        if self.apply_blend_space_contextual_command_feedback(action_id)? {
            return Ok(());
        }
        if workbench_extension_workspace_control_id(action_id).is_some() {
            self.apply_workbench_extension_workspace(action_id)?;
        }
        if let Some(control_id) = workbench_extension_panel_row_control_id(action_id) {
            self.select_exclusive_selected(
                workbench_extension_panel_row_group(action_id),
                control_id,
            )?;
        } else if workbench_extension_panel_command_control_id(action_id).is_some() {
            return self.apply_workbench_extension_module_command_feedback(action_id);
        } else if self.should_open_dropdown_for_module_field_action(source_control_id, action_id) {
            self.toggle_popup(source_control_id)?;
        }
        self.apply_workbench_extension_module_command_feedback(action_id)
    }

    fn apply_blend_space_validation_action(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let (selected_control_id, visible_rows) = match action_id {
            "workbench.extension.blend_space.validation.filter_all" => (
                Some("WorkbenchValidationLogAll"),
                BLEND_SPACE_VALIDATION_ROWS,
            ),
            "workbench.extension.blend_space.validation.filter_errors" => {
                (Some("WorkbenchValidationLogErrors"), &[])
            }
            "workbench.extension.blend_space.validation.filter_warnings" => (
                Some("WorkbenchValidationLogWarnings"),
                &["WorkbenchValidationLogWarningRow"][..],
            ),
            "workbench.extension.blend_space.validation.filter_infos" => (
                Some("WorkbenchValidationLogInfos"),
                &[
                    "WorkbenchValidationLogInfoAxesRow",
                    "WorkbenchValidationLogInfoRangeRow",
                    "WorkbenchValidationLogInfoDuplicatesRow",
                ][..],
            ),
            "workbench.extension.blend_space.validation.clear" => (None, &[]),
            _ => return Ok(false),
        };

        if let Some(selected_control_id) = selected_control_id {
            self.select_exclusive(BLEND_SPACE_VALIDATION_FILTER_CONTROLS, selected_control_id)?;
        } else {
            for control_id in BLEND_SPACE_VALIDATION_FILTER_CONTROLS {
                self.set_control_active(control_id, false)?;
            }
        }
        for row_id in BLEND_SPACE_VALIDATION_ROWS {
            self.set_visible(row_id, visible_rows.contains(row_id))?;
        }
        Ok(true)
    }
}
