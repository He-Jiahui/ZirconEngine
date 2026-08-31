use zircon_runtime_interface::ui::component::UiValue;

use crate::ui::binding::EditorUiBindingPayload;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const COMPONENT_LAB_BINDING_PREFIX: &str = "ComponentLab/";
const COMPONENT_LAB_ACTION_PREFIX: &str = "component_lab.";
const EDIT_ACTION_SUFFIX: &str = ".edit";
const COMMIT_ACTION_SUFFIX: &str = ".commit";
const QUERY_PROPERTY: &str = "query";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn edit_component_lab_field(
        &mut self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) -> Result<Option<bool>, BuiltinHostWindowTemplateBridgeError> {
        if !is_component_lab_field_binding(self, binding_id) {
            return Ok(None);
        }
        if control_id.trim().is_empty()
            || !self.has_control(control_id)
            || !self.control_owns_binding(control_id, binding_id)
        {
            return Ok(Some(false));
        }

        if self.control_string(control_id, QUERY_PROPERTY).is_some() {
            self.mutate_control_property(
                control_id,
                QUERY_PROPERTY,
                UiValue::String(value.to_string()),
            )?;
        } else if let (Some(min), Some(max)) = (
            self.control_float(control_id, "min"),
            self.control_float(control_id, "max"),
        ) {
            let numeric_text = value.trim();
            self.mutate_control_property(
                control_id,
                "value_text",
                UiValue::String(numeric_text.to_string()),
            )?;
            if let Ok(numeric) = numeric_text.parse::<f64>() {
                self.mutate_control_property(control_id, "value", UiValue::Float(numeric))?;
                let span = max - min;
                let percent = if span > f32::EPSILON {
                    ((numeric as f32 - min) / span).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                self.mutate_control_property(
                    control_id,
                    "value_percent",
                    UiValue::Float(f64::from(percent)),
                )?;
            }
        } else {
            self.mutate_control_property(control_id, "value", UiValue::String(value.to_string()))?;
        }
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(Some(true))
    }

    fn control_owns_binding(&self, control_id: &str, binding_id: &str) -> bool {
        self.host_projection()
            .node_by_control_id(control_id)
            .is_some_and(|node| {
                node.routes
                    .iter()
                    .any(|route| route.binding_id == binding_id)
            })
    }
}

fn is_component_lab_field_binding(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    binding_id: &str,
) -> bool {
    if !binding_id.starts_with(COMPONENT_LAB_BINDING_PREFIX) {
        return false;
    }
    bridge.binding_by_id(binding_id).is_some_and(|binding| {
        matches!(
            binding.payload(),
            EditorUiBindingPayload::MenuAction { action_id }
                if action_id.starts_with(COMPONENT_LAB_ACTION_PREFIX)
                    && (action_id.ends_with(EDIT_ACTION_SUFFIX)
                        || action_id.ends_with(COMMIT_ACTION_SUFFIX))
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::layout::UiSize;

    #[test]
    fn component_lab_numeric_edit_updates_value_text_and_normalized_position() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
        assert_eq!(
            bridge
                .edit_component_lab_field(
                    "WorkbenchInputSlider",
                    "ComponentLab/InputSliderEdit",
                    "25",
                )
                .unwrap(),
            Some(true)
        );
        assert_eq!(
            bridge.control_float("WorkbenchInputSlider", "value"),
            Some(25.0)
        );
        assert_eq!(
            bridge.control_float("WorkbenchInputSlider", "value_percent"),
            Some(0.25)
        );
        assert_eq!(
            bridge
                .control_string("WorkbenchInputSlider", "value_text")
                .as_deref(),
            Some("25")
        );
    }

    #[test]
    fn component_lab_search_edit_updates_its_declared_value_property() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
        assert_eq!(
            bridge
                .edit_component_lab_field(SEARCH_CONTROL, "ComponentLab/InputSearchEdit", "button",)
                .unwrap(),
            Some(true)
        );
        assert_eq!(
            bridge.control_string(SEARCH_CONTROL, "query").as_deref(),
            Some("button")
        );
    }

    #[test]
    fn component_lab_text_edit_preserves_user_whitespace() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
        assert_eq!(
            bridge
                .edit_component_lab_field(
                    "WorkbenchInputText",
                    "ComponentLab/InputTextEdit",
                    "  exact text  ",
                )
                .unwrap(),
            Some(true)
        );
        assert_eq!(
            bridge
                .control_string("WorkbenchInputText", "value")
                .as_deref(),
            Some("  exact text  ")
        );
    }

    #[test]
    fn component_lab_incomplete_numeric_draft_keeps_the_last_valid_value() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
        assert_eq!(
            bridge
                .edit_component_lab_field(
                    "WorkbenchInputStepper",
                    "ComponentLab/InputStepperEdit",
                    "-",
                )
                .unwrap(),
            Some(true)
        );
        assert_eq!(
            bridge.control_float("WorkbenchInputStepper", "value"),
            Some(42.0)
        );
        assert_eq!(
            bridge
                .control_string("WorkbenchInputStepper", "value_text")
                .as_deref(),
            Some("-")
        );
    }
}
