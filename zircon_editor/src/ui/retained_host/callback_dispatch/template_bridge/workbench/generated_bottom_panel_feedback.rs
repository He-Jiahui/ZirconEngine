use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
    generated_bottom_panel_navigation::{
        workbench_generated_bottom_mode_control_id, workbench_generated_bottom_route_target,
        GENERATED_BOTTOM_MODE_CONTROLS,
    },
};

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_workbench_generated_bottom_feedback(
        &mut self,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if let Some(target) = workbench_generated_bottom_route_target(action_id) {
            self.mutate_control_property(
                "WorkbenchStatusReady",
                "text",
                UiValue::String(format!(
                    "{} {} bottom panel selected",
                    target.module_label, target.panel_label
                )),
            )?;
            self.mutate_control_property(
                "WorkbenchStatusMessages",
                "text",
                UiValue::String("1 Message".to_string()),
            )?;
            self.mutate_control_property(
                "WorkbenchGeneratedBottomSelectedRoute",
                "value_text",
                UiValue::String(target.panel_route.to_string()),
            )?;
            self.mutate_control_property(
                "WorkbenchGeneratedBottomSelectedModule",
                "value_text",
                UiValue::String(target.module_label.to_string()),
            )?;
            self.mutate_control_property(
                "WorkbenchGeneratedBottomSelectedPanel",
                "value_text",
                UiValue::String(target.panel_label.to_string()),
            )?;
            self.select_generated_bottom_mode(target.mode_control_id)?;
            return Ok(());
        }

        if let Some(mode_control_id) = workbench_generated_bottom_mode_control_id(action_id) {
            self.select_generated_bottom_mode(mode_control_id)?;
            self.mutate_control_property(
                "WorkbenchStatusReady",
                "text",
                UiValue::String("Generated bottom panel mode selected".to_string()),
            )?;
            return Ok(());
        }

        match action_id {
            "workbench.generated_bottom.open_panel.invoke" => {
                self.set_visible("WorkbenchGeneratedBottomPanel", true)?;
                self.mutate_control_property(
                    "WorkbenchStatusReady",
                    "text",
                    UiValue::String("Generated bottom panel opened".to_string()),
                )?;
            }
            "workbench.generated_bottom.pin_panel.invoke" => {
                self.mutate_control_property(
                    "WorkbenchStatusReady",
                    "text",
                    UiValue::String("Generated bottom panel pinned".to_string()),
                )?;
            }
            "workbench.generated_bottom.filter.edit"
            | "workbench.generated_bottom.filter.commit" => {
                self.mutate_control_property(
                    "WorkbenchStatusReady",
                    "text",
                    UiValue::String("Generated bottom filter updated".to_string()),
                )?;
            }
            "workbench.generated_bottom.mode.edit" | "workbench.generated_bottom.mode.commit" => {
                self.mutate_control_property(
                    "WorkbenchStatusReady",
                    "text",
                    UiValue::String("Generated bottom mode updated".to_string()),
                )?;
            }
            _ => {}
        }

        Ok(())
    }

    fn select_generated_bottom_mode(
        &mut self,
        selected_control_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        for control_id in GENERATED_BOTTOM_MODE_CONTROLS {
            self.set_control_active(control_id, *control_id == selected_control_id)?;
        }

        let mode_text = selected_control_id
            .strip_prefix("WorkbenchGeneratedBottomMode")
            .unwrap_or("Output");
        self.mutate_control_property(
            "WorkbenchGeneratedBottomSelectedMode",
            "value_text",
            UiValue::String(mode_text.to_string()),
        )?;
        self.mutate_control_property(
            "WorkbenchGeneratedBottomModeDropdown",
            "value",
            UiValue::String(mode_text.to_string()),
        )?;
        self.mutate_control_property(
            "WorkbenchGeneratedBottomModeDropdown",
            "text",
            UiValue::String(mode_text.to_string()),
        )?;
        Ok(())
    }
}
