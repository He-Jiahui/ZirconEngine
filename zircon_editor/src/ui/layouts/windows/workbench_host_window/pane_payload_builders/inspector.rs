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
        drawer_available: component.drawer_available,
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
    InspectorPluginComponentPropertyPayload {
        field_id: property.field_id.clone(),
        name: property.name.clone(),
        label: property.label.clone(),
        value: property.value.clone(),
        value_kind: property.value_kind.clone(),
        editable: property.editable,
    }
}
