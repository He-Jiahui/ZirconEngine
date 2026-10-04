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
