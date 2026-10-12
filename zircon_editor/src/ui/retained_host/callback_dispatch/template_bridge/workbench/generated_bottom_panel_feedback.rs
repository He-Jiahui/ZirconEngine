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
            _ => {}
        }

        Ok(())
    }

    pub(super) fn select_generated_bottom_mode(
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
        Ok(())
    }
}

// Pure projection metadata from the existing Generated navigation owner.
// The caller writes these attributes directly to its already leased pane Surface.
impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn generated_bottom_pane_feedback_attributes(
        action_id: &str,
    ) -> (
        std::collections::BTreeMap<String, std::collections::BTreeMap<String, toml::Value>>,
        Option<String>,
    ) {
        use std::collections::BTreeMap;
        let mut updates = BTreeMap::new();
        let route = workbench_generated_bottom_route_target(action_id);
        let mode = route
            .map(|target| target.mode_control_id)
            .or_else(|| workbench_generated_bottom_mode_control_id(action_id));
        let mut status = None;
        if let Some(target) = route {
            for (control, value) in [
                ("WorkbenchGeneratedBottomSelectedRoute", target.panel_route),
                (
                    "WorkbenchGeneratedBottomSelectedModule",
                    target.module_label,
                ),
                ("WorkbenchGeneratedBottomSelectedPanel", target.panel_label),
            ] {
                updates.insert(
                    control.to_string(),
                    BTreeMap::from([(
                        "value_text".to_string(),
                        toml::Value::String(value.to_string()),
                    )]),
                );
            }
            status = Some(format!(
                "{} {} bottom panel selected",
                target.module_label, target.panel_label
            ));
        }
        if let Some(mode) = mode {
            for control in GENERATED_BOTTOM_MODE_CONTROLS {
                let selected = *control == mode;
                updates.insert(
                    control.to_string(),
                    BTreeMap::from([
                        ("selected".to_string(), toml::Value::Boolean(selected)),
                        ("checked".to_string(), toml::Value::Boolean(selected)),
                        ("value".to_string(), toml::Value::Boolean(selected)),
                    ]),
                );
            }
            updates.insert(
                "WorkbenchGeneratedBottomSelectedMode".to_string(),
                BTreeMap::from([(
                    "value_text".to_string(),
                    toml::Value::String(
                        mode.strip_prefix("WorkbenchGeneratedBottomMode")
                            .unwrap_or("Output")
                            .to_string(),
                    ),
                )]),
            );
            if status.is_none() {
                status = Some("Generated bottom panel mode selected".to_string());
            }
        }
        match action_id {
            "workbench.generated_bottom.open_panel.invoke" => {
                updates.insert(
                    "WorkbenchGeneratedBottomPanelHost".to_string(),
                    BTreeMap::from([(
                        "visibility".to_string(),
                        toml::Value::String("visible".to_string()),
                    )]),
                );
                status = Some("Generated bottom panel opened".to_string());
            }
            "workbench.generated_bottom.pin_panel.invoke" => {
                status = Some("Generated bottom panel pinned".to_string())
            }
            "workbench.generated_bottom.filter.edit"
            | "workbench.generated_bottom.filter.commit" => {
                status = Some("Generated bottom filter updated".to_string())
            }
            _ => {}
        }
        (updates, status)
    }
}
