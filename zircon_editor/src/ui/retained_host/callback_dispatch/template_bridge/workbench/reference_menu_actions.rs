use crate::ui::retained_host::workbench_preview_actions::is_workbench_preview_action;

use super::{
    blend_space_search::is_blend_space_search_action,
    blend_space_transport::is_animation_transport_action,
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
    extension_module_navigation::{
        is_workbench_extension_action, workbench_extension_panel_command_control_id,
        workbench_extension_panel_command_group, workbench_extension_panel_row_control_id,
        workbench_extension_panel_row_group, workbench_extension_panel_tab_control_id,
        workbench_extension_panel_tab_group, workbench_extension_workspace_control_id,
    },
    generated_bottom_panel_navigation::is_workbench_generated_bottom_action,
    module_navigation::{
        is_workbench_module_action, workbench_module_command_control_id,
        workbench_module_panel_command_control_id, workbench_module_panel_row_control_id,
        workbench_module_panel_row_group, workbench_module_panel_tab_control_id,
        workbench_module_panel_tab_group, workbench_module_tab_control_id, MODULE_COMMAND_CONTROLS,
        MODULE_PANEL_COMMAND_CONTROLS, MODULE_TAB_CONTROLS,
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
const PANEL_SCENE_TAB_CONTROLS: &[&str] = &["WorkbenchSceneTabScene", "WorkbenchSceneTabLayers"];
const PANEL_INSPECTOR_TAB_CONTROLS: &[&str] = &[
    "WorkbenchInspectorTabInspector",
    "WorkbenchInspectorTabHistory",
];
const PANEL_COMPONENT_DRAWER_TAB_CONTROLS: &[&str] =
    &["WorkbenchDrawerTabComponents", "WorkbenchDrawerTabConsole"];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_reference_menu_action(
        &mut self,
        source_control_id: &str,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if self.apply_workbench_window_menu_action(source_control_id, action_id)? {
            return Ok(());
        }
        match action_id {
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
            "scene_tree.scene_tab.select" => {
                self.select_exclusive(PANEL_SCENE_TAB_CONTROLS, "WorkbenchSceneTabScene")?;
            }
            "scene_tree.layers_tab.select" => {
                self.select_exclusive(PANEL_SCENE_TAB_CONTROLS, "WorkbenchSceneTabLayers")?;
            }
            "inspector.main_tab.select" => {
                self.select_exclusive(
                    PANEL_INSPECTOR_TAB_CONTROLS,
                    "WorkbenchInspectorTabInspector",
                )?;
            }
            "inspector.history_tab.select" => {
                self.select_exclusive(
                    PANEL_INSPECTOR_TAB_CONTROLS,
                    "WorkbenchInspectorTabHistory",
                )?;
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
        if let Some(control_id) = workbench_module_tab_control_id(action_id) {
            self.select_exclusive(MODULE_TAB_CONTROLS, control_id)?;
            self.apply_workbench_module_workspace(action_id)?;
        } else if let Some(control_id) = workbench_module_command_control_id(action_id) {
            self.select_exclusive(MODULE_COMMAND_CONTROLS, control_id)?;
            if action_id == "workbench.module.browse.invoke" {
                self.select_exclusive(MODULE_TAB_CONTROLS, "WorkbenchModuleAssets")?;
                self.apply_workbench_module_workspace("workbench.module.assets.select")?;
            }
        } else if let Some(control_id) = workbench_module_panel_tab_control_id(action_id) {
            self.select_exclusive(workbench_module_panel_tab_group(action_id), control_id)?;
        } else if let Some(control_id) = workbench_module_panel_row_control_id(action_id) {
            self.select_exclusive_selected(
                workbench_module_panel_row_group(action_id),
                control_id,
            )?;
        } else if let Some(control_id) = workbench_module_panel_command_control_id(action_id) {
            self.select_exclusive(MODULE_PANEL_COMMAND_CONTROLS, control_id)?;
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
        if workbench_extension_workspace_control_id(action_id).is_some() {
            self.apply_workbench_extension_workspace(action_id)?;
        }
        if let Some(control_id) = workbench_extension_panel_tab_control_id(action_id) {
            self.select_exclusive(workbench_extension_panel_tab_group(action_id), control_id)?;
        } else if let Some(control_id) = workbench_extension_panel_row_control_id(action_id) {
            self.select_exclusive_selected(
                workbench_extension_panel_row_group(action_id),
                control_id,
            )?;
        } else if let Some(control_id) = workbench_extension_panel_command_control_id(action_id) {
            self.select_exclusive(
                workbench_extension_panel_command_group(action_id),
                control_id,
            )?;
        } else if self.should_open_dropdown_for_module_field_action(source_control_id, action_id) {
            self.toggle_popup(source_control_id)?;
        }
        self.apply_workbench_extension_module_command_feedback(action_id)
    }
}
