use serde::{Deserialize, Serialize};
use zircon_runtime_interface::reflect::ReflectedValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneEditorInspectorField {
    pub component_type_path: String,
    pub component_display_name: String,
    pub field_name: String,
    pub field_display_name: String,
    pub value_type_path: String,
    pub value: ReflectedValue,
    pub editable: bool,
    pub serializable: bool,
    pub plugin_owned: bool,
}
