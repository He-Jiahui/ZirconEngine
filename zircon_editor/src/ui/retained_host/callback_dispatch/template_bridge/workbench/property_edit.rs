use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const MESH_ROW: &str = "WorkbenchMeshRow";
const MATERIAL_ROW: &str = "WorkbenchMaterialRow";
const PROPERTY_SLOT_03_ROW: &str = "WorkbenchComponentPropertySlot03Row";
const PROPERTY_SLOT_04_ROW: &str = "WorkbenchComponentPropertySlot04Row";
const PROPERTY_VIRTUAL_ROW_PREFIX: &str = "WorkbenchComponentPropertyVirtualRow";
const PROPERTY_01_EDIT: &str = "Inspector/ComponentProperty01Edit";
const PROPERTY_01_COMMIT: &str = "Inspector/ComponentProperty01Commit";
const PROPERTY_02_EDIT: &str = "Inspector/ComponentProperty02Edit";
const PROPERTY_02_COMMIT: &str = "Inspector/ComponentProperty02Commit";
const PROPERTY_03_EDIT: &str = "Inspector/ComponentProperty03Edit";
const PROPERTY_03_COMMIT: &str = "Inspector/ComponentProperty03Commit";
const PROPERTY_04_EDIT: &str = "Inspector/ComponentProperty04Edit";
const PROPERTY_04_COMMIT: &str = "Inspector/ComponentProperty04Commit";
const PROPERTY_FIELD_ID: &str = "inspector_property_field_id";
const PROPERTY_NAME: &str = "inspector_property_name";
const PROPERTY_LABEL: &str = "inspector_property_label";
const PROPERTY_EDITABLE: &str = "inspector_property_editable";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn edit_inspector_component_property(
        &mut self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) -> Result<Option<bool>, BuiltinHostWindowTemplateBridgeError> {
        let Some(authored_control_id) = inspector_property_row_for_binding(binding_id) else {
            return Ok(None);
        };
        let target_control_id =
            self.edit_target_control_id(control_id, binding_id, authored_control_id)?;
        let target_control_id = target_control_id.as_str();
        if !control_id.is_empty() && control_id != target_control_id {
            return Ok(Some(false));
        }
        if !self.has_control(target_control_id)
            || !control_bool(self, target_control_id, PROPERTY_EDITABLE)
            || control_string(self, target_control_id, PROPERTY_FIELD_ID)
                .map_or(true, |field_id| field_id.trim().is_empty())
        {
            return Ok(Some(false));
        }

        let raw_value = edit_raw_value(self, target_control_id, value);
        self.mutate_control_property(
            target_control_id,
            "value",
            UiValue::String(raw_value.clone()),
        )?;
        self.mutate_control_property(
            target_control_id,
            "value_text",
            UiValue::String(component_property_value_text(&raw_value)),
        )?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(Some(true))
    }

    fn edit_target_control_id<'a>(
        &self,
        control_id: &str,
        binding_id: &str,
        authored_control_id: &'static str,
    ) -> Result<String, BuiltinHostWindowTemplateBridgeError> {
        if !control_id.is_empty()
            && matches!(binding_id, PROPERTY_04_EDIT | PROPERTY_04_COMMIT)
            && (control_id == PROPERTY_SLOT_04_ROW
                || control_id.starts_with(PROPERTY_VIRTUAL_ROW_PREFIX))
            && self.control_node_id(control_id).is_some()
        {
            Ok(control_id.to_string())
        } else {
            Ok(authored_control_id.to_string())
        }
    }
}

fn inspector_property_row_for_binding(binding_id: &str) -> Option<&'static str> {
    match binding_id {
        PROPERTY_01_EDIT | PROPERTY_01_COMMIT => Some(MESH_ROW),
        PROPERTY_02_EDIT | PROPERTY_02_COMMIT => Some(MATERIAL_ROW),
        PROPERTY_03_EDIT | PROPERTY_03_COMMIT => Some(PROPERTY_SLOT_03_ROW),
        PROPERTY_04_EDIT | PROPERTY_04_COMMIT => Some(PROPERTY_SLOT_04_ROW),
        _ => None,
    }
}

fn component_property_value_text(value: &str) -> String {
    non_empty_label(value, "-")
}

fn edit_raw_value(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    value: &str,
) -> String {
    let Some(label) = control_string(bridge, control_id, PROPERTY_LABEL)
        .or_else(|| control_string(bridge, control_id, PROPERTY_NAME))
        .map(|label| non_empty_label(&label, ""))
        .filter(|label| !label.is_empty())
    else {
        return value.to_string();
    };
    value
        .strip_prefix(label.as_str())
        .map(str::trim_start)
        .unwrap_or(value)
        .to_string()
}

fn control_bool(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> bool {
    bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
                .and_then(|metadata| metadata.attributes.get(property))
                .and_then(toml::Value::as_bool)
        })
        .unwrap_or(false)
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

fn non_empty_label(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}
