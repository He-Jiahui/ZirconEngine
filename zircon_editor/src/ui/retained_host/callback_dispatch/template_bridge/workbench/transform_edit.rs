use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const POSITION_ROW: &str = "WorkbenchTransformPosition";
const POSITION_X: &str = "WorkbenchTransformPositionX";
const POSITION_Y: &str = "WorkbenchTransformPositionY";
const POSITION_Z: &str = "WorkbenchTransformPositionZ";
const ROTATION_ROW: &str = "WorkbenchTransformRotation";
const ROTATION_X: &str = "WorkbenchTransformRotationX";
const ROTATION_Y: &str = "WorkbenchTransformRotationY";
const ROTATION_Z: &str = "WorkbenchTransformRotationZ";
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
        "Inspector/TransformRotationXEdit" | "Inspector/TransformRotationXCommit" => {
            Some(rotation_edit(ROTATION_X, "X"))
        }
        "Inspector/TransformRotationYEdit" | "Inspector/TransformRotationYCommit" => {
            Some(rotation_edit(ROTATION_Y, "Y"))
        }
        "Inspector/TransformRotationZEdit" | "Inspector/TransformRotationZCommit" => {
            Some(rotation_edit(ROTATION_Z, "Z"))
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

fn rotation_edit(control_id: &'static str, axis_label: &'static str) -> TransformAxisEdit {
    TransformAxisEdit {
        control_id,
        row_control_id: ROTATION_ROW,
        row_fields: [ROTATION_X, ROTATION_Y, ROTATION_Z],
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
    control_string(bridge, field_control_id, "value").unwrap_or_default()
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

fn control_string(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Option<String> {
    bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .and_then(|metadata| metadata.attributes.get(property))
            .and_then(toml::Value::as_str)
            .map(str::to_string)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
