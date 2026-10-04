use zircon_runtime_interface::reflect::ReflectedValue;
use zircon_runtime_interface::world_sync::WorldInspectionFieldRow;

use crate::core::extension::{FieldEditorContainer, FieldEditorInstance, InspectorField};

/// UI projection of a registered native component field. It carries no plugin ownership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorNativeFieldSnapshot {
    pub component_type_path: String,
    pub component_label: String,
    pub field_id: String,
    pub name: String,
    pub label: String,
    pub value: String,
    /// Authoritative resource identity retained separately from its displayed locator.
    pub resource_id: Option<String>,
    pub value_kind: String,
    pub field_editor: FieldEditorInstance,
}

impl InspectorNativeFieldSnapshot {
    /// Limits this surface to scene rendering components with registered reflection contracts.
    /// Transform and RenderLayerMask retain their existing typed owners.
    pub(crate) fn is_component_type(type_path: &str) -> bool {
        matches!(
            type_path,
            "zircon_runtime::scene::components::CameraComponent"
                | "zircon_runtime::scene::components::MeshRenderer"
                | "zircon_runtime::scene::components::AmbientLight"
                | "zircon_runtime::scene::components::DirectionalLight"
                | "zircon_runtime::scene::components::PointLight"
                | "zircon_runtime::scene::components::RectLight"
                | "zircon_runtime::scene::components::SpotLight"
        )
    }

    pub(crate) fn project(fields: &[WorldInspectionFieldRow]) -> Vec<Self> {
        let editors = FieldEditorContainer::builtin();
        fields
            .iter()
            .filter(|field| {
                !field.plugin_owned && Self::is_component_type(&field.component_type_path)
            })
            .map(|field| {
                let field_id = format!("{}.{}", field.component_type_path, field.field_name);
                let label = match (
                    field.component_type_path.as_str(),
                    field.field_name.as_str(),
                ) {
                    ("zircon_runtime::scene::components::CameraComponent", "fov_y_radians") => {
                        "FOV Y (Radians)".to_string()
                    }
                    ("zircon_runtime::scene::components::CameraComponent", "z_near") => {
                        "Near Clip".to_string()
                    }
                    ("zircon_runtime::scene::components::CameraComponent", "z_far") => {
                        "Far Clip".to_string()
                    }
                    _ => title_case(&field.field_display_name),
                };
                let value =
                    super::super::editor_state_snapshot_build::reflected_value_label(&field.value);
                let field_editor = InspectorField::new(
                    field_id.clone(),
                    label.clone(),
                    field.value_type_path.clone(),
                    value.clone(),
                    false,
                )
                .map(|field| editors.resolve(field))
                .unwrap_or_else(|_| FieldEditorInstance::automatic());
                Self {
                    component_type_path: field.component_type_path.clone(),
                    component_label: match field.component_type_path.as_str() {
                        "zircon_runtime::scene::components::CameraComponent" => {
                            "Camera".to_string()
                        }
                        "zircon_runtime::scene::components::MeshRenderer" => {
                            "Mesh Renderer".to_string()
                        }
                        "zircon_runtime::scene::components::AmbientLight" => {
                            "Ambient Light".to_string()
                        }
                        "zircon_runtime::scene::components::DirectionalLight" => {
                            "Directional Light".to_string()
                        }
                        "zircon_runtime::scene::components::PointLight" => {
                            "Point Light".to_string()
                        }
                        "zircon_runtime::scene::components::RectLight" => "Rect Light".to_string(),
                        "zircon_runtime::scene::components::SpotLight" => "Spot Light".to_string(),
                        _ => title_case(&field.component_display_name),
                    },
                    field_id,
                    name: field.field_name.clone(),
                    label,
                    value,
                    resource_id: match &field.value {
                        ReflectedValue::Resource(id) => Some(id.clone()),
                        _ => None,
                    },
                    value_kind: field.value_type_path.clone(),
                    field_editor,
                }
            })
            .collect()
    }
}

fn title_case(name: &str) -> String {
    name.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
#[path = "native_fields/tests/cases.rs"]
mod tests;
