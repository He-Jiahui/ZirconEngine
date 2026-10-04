mod native_fields;
mod rotation;

pub use native_fields::InspectorNativeFieldSnapshot;

use zircon_runtime::scene::NodeId;

use crate::core::extension::{FieldEditorContainer, FieldEditorInstance, InspectorField};

#[derive(Clone, Debug, PartialEq, Eq)]
/// 当前实体检查器的只读字段视图；选中身份属于权威world，文本可叠加编辑草稿。
pub struct InspectorSnapshot {
    pub id: NodeId,
    pub name: String,
    pub parent: String,
    pub translation: [String; 3],
    /// Read-only local XYZ Euler angles in degrees; None means unavailable.
    pub rotation_degrees: Option<[String; 3]>,
    pub scale: [String; 3],
    pub render_layer_mask: u32,
    /// Reflected native component fields; current UI exposes these as read-only.
    pub native_fields: Vec<InspectorNativeFieldSnapshot>,
    pub plugin_components: Vec<InspectorPluginComponentSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// 插件schema/customization状态；插件定义缺失时保留受保护JSON展示和诊断。
pub struct InspectorPluginComponentSnapshot {
    pub component_id: String,
    pub display_name: String,
    pub plugin_id: String,
    pub customization_available: bool,
    pub customization_ui_document: Option<String>,
    pub customization_controller: Option<String>,
    pub customization_template_id: Option<String>,
    pub customization_data_root: Option<String>,
    pub customization_bindings: Vec<String>,
    pub diagnostic: Option<String>,
    pub properties: Vec<InspectorPluginComponentPropertySnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// 字段显示与解析好的编辑器；editable提示不能代替写入边界验证。
pub struct InspectorPluginComponentPropertySnapshot {
    pub field_id: String,
    pub name: String,
    pub label: String,
    pub value: String,
    pub value_kind: String,
    pub editable: bool,
    /// Resolved while the immutable editor snapshot is built.
    pub field_editor: FieldEditorInstance,
}

#[cfg(test)]
#[path = "tests/inspector_snapshot.rs"]
mod tests;
