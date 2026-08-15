use crate::ui::workbench::snapshot::{
    InspectorPluginComponentPropertySnapshot, InspectorPluginComponentSnapshot,
};

use super::super::pane_payload::{
    InspectorPanePayload, InspectorPluginComponentPayload, InspectorPluginComponentPropertyPayload,
    PanePayload,
};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    let inspector = context.chrome.inspector.as_ref();
    PanePayload::InspectorV1(InspectorPanePayload {
        node_id: inspector.map(|inspector| inspector.id).unwrap_or_default(),
        name: inspector
            .map(|inspector| inspector.name.clone())
            .unwrap_or_default(),
        parent: inspector
            .map(|inspector| inspector.parent.clone())
            .unwrap_or_default(),
        translation: inspector
            .map(|inspector| inspector.translation.clone())
            .unwrap_or_else(|| Default::default()),
        delete_enabled: inspector.is_some(),
        plugin_components: inspector
            .map(|inspector| {
                inspector
                    .plugin_components
                    .iter()
                    .map(plugin_component_payload)
                    .collect()
            })
            .unwrap_or_default(),
    })
}

fn plugin_component_payload(
    component: &InspectorPluginComponentSnapshot,
) -> InspectorPluginComponentPayload {
    InspectorPluginComponentPayload {
        component_id: component.component_id.clone(),
        display_name: component.display_name.clone(),
        plugin_id: component.plugin_id.clone(),
        customization_available: component.customization_available,
        customization_ui_document: component.customization_ui_document.clone(),
        customization_controller: component.customization_controller.clone(),
        customization_template_id: component.customization_template_id.clone(),
        customization_data_root: component.customization_data_root.clone(),
        customization_bindings: component.customization_bindings.clone(),
        diagnostic: component.diagnostic.clone(),
        properties: component
            .properties
            .iter()
            .map(plugin_component_property_payload)
            .collect(),
    }
}

fn plugin_component_property_payload(
    property: &InspectorPluginComponentPropertySnapshot,
) -> InspectorPluginComponentPropertyPayload {
    let editor = &property.field_editor;
    InspectorPluginComponentPropertyPayload {
        field_id: property.field_id.clone(),
        name: property.name.clone(),
        label: property.label.clone(),
        value: property.value.clone(),
        value_kind: property.value_kind.clone(),
        field_editor_kind: editor.kind().as_str().to_string(),
        asset_reference_markers: editor
            .asset_reference_markers()
            .iter()
            .map(|marker| (*marker).to_string())
            .collect(),
        editable: property.editable,
    }
}

#[cfg(test)]
mod tests {
    use crate::core::extension::{FieldEditorContainer, FieldEditorKind, InspectorField};
    use crate::ui::workbench::snapshot::InspectorPluginComponentPropertySnapshot;

    use super::plugin_component_property_payload;

    #[test]
    fn pane_payload_preserves_frozen_field_editor_metadata() {
        let field_editor = FieldEditorContainer::builtin().resolve(
            InspectorField::new(
                "plugin.weather.CloudLayer.albedo",
                "Albedo",
                "TextureAsset",
                "res://weather/cloud_albedo.png",
                true,
            )
            .unwrap(),
        );
        let property = InspectorPluginComponentPropertySnapshot {
            field_id: "plugin.weather.CloudLayer.albedo".to_string(),
            name: "albedo".to_string(),
            label: "Albedo".to_string(),
            value: "res://weather/cloud_albedo.png".to_string(),
            value_kind: "plugin.weather.CloudAlbedoAsset".to_string(),
            editable: true,
            field_editor,
        };

        let payload = plugin_component_property_payload(&property);

        assert_eq!(payload.field_editor_kind, "asset_reference");
        assert!(payload
            .asset_reference_markers
            .contains(&"texture".to_string()));
    }
}
