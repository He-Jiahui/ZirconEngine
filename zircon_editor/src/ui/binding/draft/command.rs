use serde::{Deserialize, Serialize};
use zircon_runtime::ui::binding::UiBindingValue;

pub fn inspector_field_control_id(field_id: &str) -> Option<&'static str> {
    match field_id {
        "name" => Some("NameField"),
        "parent" => Some("ParentField"),
        "transform.translation.x" => Some("PositionXField"),
        "transform.translation.y" => Some("PositionYField"),
        "transform.translation.z" => Some("PositionZField"),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DraftCommand {
    SetInspectorField {
        subject_path: String,
        field_id: String,
        value: UiBindingValue,
    },
    SetMeshImportPath {
        value: String,
    },
}
