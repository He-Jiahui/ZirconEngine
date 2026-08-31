use zircon_runtime_interface::ui::{binding::UiBindingValue, component::UiValue};

use crate::core::editor_event::InspectorFieldChange;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const POSITION_ROW: &str = "WorkbenchTransformPosition";
const POSITION_X: &str = "WorkbenchTransformPositionX";
const POSITION_Y: &str = "WorkbenchTransformPositionY";
const POSITION_Z: &str = "WorkbenchTransformPositionZ";
const SCALE_ROW: &str = "WorkbenchTransformScale";
const SCALE_X: &str = "WorkbenchTransformScaleX";
const SCALE_Y: &str = "WorkbenchTransformScaleY";
const SCALE_Z: &str = "WorkbenchTransformScaleZ";

#[derive(Clone, Copy)]
struct TransformAxisEdit {
    control_id: &'static str,
    row_control_id: &'static str,
    row_fields: [&'static str; 3],
    axis_label: &'static str,
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn transform_axis_commit_binding(
        &self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) -> Result<Option<EditorUiBinding>, String> {
        let (expected_control_id, field_id, axis_label, binding_control_id) = match binding_id {
            "Inspector/TransformPositionXCommit" => (
                POSITION_X,
                "transform.translation.x",
                "X",
                "TransformPositionXCommit",
            ),
            "Inspector/TransformPositionYCommit" => (
                POSITION_Y,
                "transform.translation.y",
                "Y",
                "TransformPositionYCommit",
            ),
            "Inspector/TransformPositionZCommit" => (
                POSITION_Z,
                "transform.translation.z",
                "Z",
                "TransformPositionZCommit",
            ),
            "Inspector/TransformScaleXCommit" => {
                (SCALE_X, "transform.scale.x", "X", "TransformScaleXCommit")
            }
            "Inspector/TransformScaleYCommit" => {
                (SCALE_Y, "transform.scale.y", "Y", "TransformScaleYCommit")
            }
            "Inspector/TransformScaleZCommit" => {
                (SCALE_Z, "transform.scale.z", "Z", "TransformScaleZCommit")
            }
            _ => return Ok(None),
        };
        if !control_id.is_empty() && control_id != expected_control_id {
            return Ok(None);
        }
        if !self.has_control(expected_control_id) {
            return Ok(None);
        }
        let scalar = parse_finite_axis_scalar(value, axis_label)?;

        Ok(Some(EditorUiBinding::new(
            "Inspector",
            binding_control_id,
            EditorUiEventKind::Submit,
            EditorUiBindingPayload::inspector_field_batch(
                "entity://selected",
                [InspectorFieldChange::new(
                    field_id,
                    UiBindingValue::Float(f64::from(scalar)),
                )],
            ),
        )))
    }

    pub(crate) fn edit_inspector_transform_axis(
        &mut self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) -> Result<Option<bool>, BuiltinHostWindowTemplateBridgeError> {
        let Some(edit) = transform_axis_edit_for_binding(binding_id) else {
            return Ok(None);
        };
        if !control_id.is_empty() && control_id != edit.control_id {
            return Ok(Some(false));
        }
        if !self.has_control(edit.control_id) {
            return Ok(Some(false));
        }

        let raw_value = strip_axis_prefix(value, edit.axis_label);
        self.mutate_control_property(edit.control_id, "value", UiValue::String(raw_value.clone()))?;
        self.mutate_control_property(
            edit.row_control_id,
            "value",
            UiValue::String(format_axis_row_value(
                &axis_field_value(self, edit.row_fields[0], edit.control_id, &raw_value),
                &axis_field_value(self, edit.row_fields[1], edit.control_id, &raw_value),
                &axis_field_value(self, edit.row_fields[2], edit.control_id, &raw_value),
            )),
        )?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(Some(true))
    }
}

fn transform_axis_edit_for_binding(binding_id: &str) -> Option<TransformAxisEdit> {
    match binding_id {
        "Inspector/TransformPositionXEdit" | "Inspector/TransformPositionXCommit" => {
            Some(position_edit(POSITION_X, "X"))
        }
        "Inspector/TransformPositionYEdit" | "Inspector/TransformPositionYCommit" => {
            Some(position_edit(POSITION_Y, "Y"))
        }
        "Inspector/TransformPositionZEdit" | "Inspector/TransformPositionZCommit" => {
            Some(position_edit(POSITION_Z, "Z"))
        }
        "Inspector/TransformScaleXEdit" | "Inspector/TransformScaleXCommit" => {
            Some(scale_edit(SCALE_X, "X"))
        }
        "Inspector/TransformScaleYEdit" | "Inspector/TransformScaleYCommit" => {
            Some(scale_edit(SCALE_Y, "Y"))
        }
        "Inspector/TransformScaleZEdit" | "Inspector/TransformScaleZCommit" => {
            Some(scale_edit(SCALE_Z, "Z"))
        }
        _ => None,
    }
}

fn position_edit(control_id: &'static str, axis_label: &'static str) -> TransformAxisEdit {
    TransformAxisEdit {
        control_id,
        row_control_id: POSITION_ROW,
        row_fields: [POSITION_X, POSITION_Y, POSITION_Z],
        axis_label,
    }
}

fn scale_edit(control_id: &'static str, axis_label: &'static str) -> TransformAxisEdit {
    TransformAxisEdit {
        control_id,
        row_control_id: SCALE_ROW,
        row_fields: [SCALE_X, SCALE_Y, SCALE_Z],
        axis_label,
    }
}

fn axis_field_value(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    field_control_id: &str,
    edited_control_id: &str,
    edited_value: &str,
) -> String {
    if field_control_id == edited_control_id {
        return edited_value.to_string();
    }
    bridge
        .control_string(field_control_id, "value")
        .unwrap_or_default()
}

fn format_axis_row_value(x: &str, y: &str, z: &str) -> String {
    format!("X {}   Y {}   Z {}", x.trim(), y.trim(), z.trim())
}

fn strip_axis_prefix(value: &str, axis_label: &str) -> String {
    value
        .trim()
        .strip_prefix(axis_label)
        .map(str::trim_start)
        .unwrap_or_else(|| value.trim())
        .to_string()
}

fn parse_finite_axis_scalar(value: &str, axis_label: &str) -> Result<f32, String> {
    let scalar = strip_axis_prefix(value, axis_label);
    let parsed = scalar.parse::<f32>().map_err(|_| {
        format!("Inspector transform {axis_label} value `{scalar}` must be a finite number")
    })?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(format!(
            "Inspector transform {axis_label} value `{scalar}` must be a finite number"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::layout::UiSize;

    #[test]
    fn strip_axis_prefix_accepts_native_axis_labels() {
        assert_eq!(strip_axis_prefix("X 42.0", "X"), "42.0");
        assert_eq!(strip_axis_prefix("90 deg", "Y"), "90 deg");
    }

    #[test]
    fn format_axis_row_value_keeps_reference_spacing() {
        assert_eq!(
            format_axis_row_value("12.0", "3.5", "-8.0"),
            "X 12.0   Y 3.5   Z -8.0"
        );
    }

    #[test]
    fn transform_commit_emits_a_typed_finite_scalar() {
        let bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

        let binding = bridge
            .transform_axis_commit_binding(
                POSITION_X,
                "Inspector/TransformPositionXCommit",
                "X 4.25",
            )
            .unwrap()
            .expect("position commit should resolve");
        let EditorUiBindingPayload::InspectorFieldBatch { changes, .. } = binding.payload() else {
            panic!("position commit must dispatch an inspector field batch");
        };

        assert_eq!(changes[0].field_id, "transform.translation.x");
        assert_eq!(changes[0].value, UiBindingValue::Float(4.25));

        let binding = bridge
            .transform_axis_commit_binding(SCALE_Z, "Inspector/TransformScaleZCommit", "Z 2.5")
            .unwrap()
            .expect("scale commit should resolve");
        let EditorUiBindingPayload::InspectorFieldBatch { changes, .. } = binding.payload() else {
            panic!("scale commit must dispatch an inspector field batch");
        };

        assert_eq!(changes[0].field_id, "transform.scale.z");
        assert_eq!(changes[0].value, UiBindingValue::Float(2.5));
        assert_eq!(
            bridge
                .transform_axis_commit_binding(SCALE_Z, "Inspector/TransformScaleZCommit", "Z NaN",)
                .unwrap_err(),
            "Inspector transform Z value `NaN` must be a finite number"
        );
    }
}
