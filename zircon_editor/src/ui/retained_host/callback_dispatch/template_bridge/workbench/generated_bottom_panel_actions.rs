use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
    generated_bottom_panel_navigation::{
        GENERATED_BOTTOM_MODE_CONTROLS, GENERATED_BOTTOM_ROUTE_CONTROLS,
        workbench_generated_bottom_mode_control_id, workbench_generated_bottom_route_control_id,
    },
};

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_workbench_generated_bottom_action(
        &mut self,
        source_control_id: &str,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.open_workbench_generated_bottom_drawer()?;
        if let Some(control_id) = workbench_generated_bottom_mode_control_id(action_id) {
            self.select_exclusive(GENERATED_BOTTOM_MODE_CONTROLS, control_id)?;
        } else if let Some(control_id) = workbench_generated_bottom_route_control_id(action_id) {
            self.select_exclusive_selected(GENERATED_BOTTOM_ROUTE_CONTROLS, control_id)?;
        } else if self.should_open_dropdown_for_module_field_action(source_control_id, action_id) {
            self.toggle_popup(source_control_id)?;
        }
        self.apply_workbench_generated_bottom_feedback(action_id)
    }
}
