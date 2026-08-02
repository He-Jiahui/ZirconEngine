use zircon_runtime::scene::NodeId;

use crate::core::extension::{FieldEditorContainer, FieldEditorInstance, InspectorField};

#[derive(Clone, Debug)]
pub struct InspectorSnapshot {
    pub id: NodeId,
    pub name: String,
    pub parent: String,
    pub translation: [String; 3],
    pub plugin_components: Vec<InspectorPluginComponentSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
mod tests {
    use crate::core::extension::{
        FieldEditorContainer, FieldEditorInstance, FieldEditorKind, InspectorField,
    };

    use super::InspectorPluginComponentPropertySnapshot;

    fn property(value_kind: &str) -> InspectorPluginComponentPropertySnapshot {
        let field_editor = InspectorField::new(
            "plugin.weather.cloud_layer.value",
            "Value",
            value_kind,
            "res://clouds.ztex",
            true,
        )
        .map(|field| FieldEditorContainer::builtin().resolve(field))
        .unwrap_or_else(|_| FieldEditorInstance::automatic());
        InspectorPluginComponentPropertySnapshot {
            field_id: "plugin.weather.cloud_layer.value".to_string(),
            name: "value".to_string(),
            label: "Value".to_string(),
            value: "res://clouds.ztex".to_string(),
            value_kind: value_kind.to_string(),
            editable: true,
            field_editor,
        }
    }

    #[test]
    fn property_projection_uses_field_editor_container_with_auto_fallback() {
        let numeric = property("f32").field_editor;
        assert_eq!(numeric.kind(), FieldEditorKind::Numeric);
        assert!(numeric.asset_reference_markers().is_empty());

        let asset = property("TextureAsset").field_editor;
        assert_eq!(asset.kind(), FieldEditorKind::AssetReference);
        assert_eq!(asset.asset_reference_markers().len(), 21);
        assert!(asset.asset_reference_markers().contains(&"texture"));

        assert_eq!(
            property("plugin<unsupported>").field_editor.kind(),
            FieldEditorKind::Auto
        );
    }
}
