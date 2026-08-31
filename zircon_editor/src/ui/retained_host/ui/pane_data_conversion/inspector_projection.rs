use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    InspectorPaneViewData, InspectorPluginComponentPropertyViewData,
    InspectorPluginComponentViewData, PaneContentSize, PaneData, PanePayload,
};
use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::EditorUiHostRuntime;
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::template_node_conversion::to_host_contract_template_node;
use super::inspector_fields::{inspector_field_nodes, InspectorVisualFields};
use super::pane_component_projection::host_template_node;
use super::pane_template_runtime;
use super::pane_value_conversion::{value_as_bool, value_as_string};
use super::template_node_projection::project_node_vec;

struct InspectorProjectionIdentity {
    name: String,
    parent: String,
    x: String,
    y: String,
    z: String,
    delete_enabled: bool,
}

fn into_inspector_projection_identity(
    fields: InspectorVisualFields,
) -> InspectorProjectionIdentity {
    let InspectorVisualFields {
        name,
        parent,
        x,
        y,
        z,
        delete_enabled,
        ..
    } = fields;
    InspectorProjectionIdentity {
        name,
        parent,
        x,
        y,
        z,
        delete_enabled,
    }
}

fn to_host_contract_inspector_pane(
    data: &InspectorPaneViewData,
    content_size: PaneContentSize,
) -> host_contract::InspectorPaneData {
    let fields = InspectorVisualFields::from_view_data(&data);
    let mut nodes = project_node_vec(&data.nodes, to_host_contract_template_node);
    let inspector_nodes = inspector_field_nodes(&fields, &nodes, content_size);
    nodes.extend(inspector_nodes);

    host_contract::InspectorPaneData {
        nodes: model_rc(nodes),
        info: data.info.clone(),
        inspector_name: data.inspector_name.clone(),
        inspector_parent: data.inspector_parent.clone(),
        inspector_x: data.inspector_x.clone(),
        inspector_y: data.inspector_y.clone(),
        inspector_z: data.inspector_z.clone(),
        delete_enabled: data.delete_enabled,
    }
}

pub(crate) fn to_host_contract_inspector_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::InspectorPaneData {
    inspector_template_projection(data, content_size, None).unwrap_or_else(|| {
        to_host_contract_inspector_pane(&data.native_body.inspector, content_size)
    })
}

pub(crate) fn to_host_contract_inspector_pane_from_host_pane_with_runtime(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> host_contract::InspectorPaneData {
    inspector_template_projection(data, content_size, Some(runtime)).unwrap_or_else(|| {
        to_host_contract_inspector_pane(&data.native_body.inspector, content_size)
    })
}

fn inspector_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
) -> Option<host_contract::InspectorPaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    let PanePayload::InspectorV1(payload) = &presentation.body.payload else {
        return None;
    };

    let runtime = pane_template_runtime(runtime)?;
    let projection = runtime.project_pane_body(&presentation.body).ok()?;
    let mut surface = runtime
        .build_shared_surface(&presentation.body.document_id)
        .ok()?;
    surface
        .compute_layout(UiSize::new(
            content_size.width.max(0.0),
            content_size.height.max(0.0),
        ))
        .ok()?;
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .ok()?;

    let inspector_name = projection
        .root
        .attributes
        .get("payload_name")
        .and_then(value_as_string)
        .unwrap_or_else(|| payload.name.clone());
    let inspector_parent = projection
        .root
        .attributes
        .get("payload_parent")
        .and_then(value_as_string)
        .unwrap_or_else(|| payload.parent.clone());
    let inspector_x = projection
        .root
        .attributes
        .get("payload_translation_x")
        .and_then(value_as_string)
        .unwrap_or_else(|| payload.translation[0].clone());
    let inspector_y = projection
        .root
        .attributes
        .get("payload_translation_y")
        .and_then(value_as_string)
        .unwrap_or_else(|| payload.translation[1].clone());
    let inspector_z = projection
        .root
        .attributes
        .get("payload_translation_z")
        .and_then(value_as_string)
        .unwrap_or_else(|| payload.translation[2].clone());
    let delete_enabled = projection
        .root
        .attributes
        .get("payload_delete_enabled")
        .and_then(value_as_bool)
        .unwrap_or(payload.delete_enabled);
    let fields = InspectorVisualFields {
        info: data.info.to_string(),
        name: inspector_name,
        parent: inspector_parent,
        x: inspector_x,
        y: inspector_y,
        z: inspector_z,
        delete_enabled,
        plugin_components: payload
            .plugin_components
            .iter()
            .map(|component| InspectorPluginComponentViewData {
                component_id: component.component_id.clone(),
                display_name: component.display_name.clone(),
                customization_available: component.customization_available,
                customization_ui_document: component.customization_ui_document.clone(),
                customization_template_id: component.customization_template_id.clone(),
                diagnostic: component.diagnostic.clone(),
                properties: component
                    .properties
                    .iter()
                    .map(|property| InspectorPluginComponentPropertyViewData {
                        field_id: property.field_id.clone(),
                        label: property.label.clone(),
                        value: property.value.clone(),
                        value_kind: property.value_kind.clone(),
                        editable: property.editable,
                    })
                    .collect(),
            })
            .collect(),
    };
    let mut nodes = host_model
        .nodes
        .into_iter()
        .filter_map(host_template_node)
        .collect::<Vec<_>>();
    let inspector_nodes = inspector_field_nodes(&fields, &nodes, content_size);
    nodes.extend(inspector_nodes);
    let identity = into_inspector_projection_identity(fields);

    Some(host_contract::InspectorPaneData {
        nodes: model_rc(nodes),
        info: data.info.clone(),
        inspector_name: identity.name.into(),
        inspector_parent: identity.parent.into(),
        inspector_x: identity.x.into(),
        inspector_y: identity.y.into(),
        inspector_z: identity.z.into(),
        delete_enabled: identity.delete_enabled,
    })
}

#[cfg(test)]
#[path = "inspector_projection/owned_identity_tests.rs"]
mod owned_identity_tests;
