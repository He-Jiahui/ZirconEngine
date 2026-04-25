use toml::{Table, Value};

use crate::ui::layouts::windows::workbench_host_window::{PaneBodyPresentation, PanePayload};
use crate::ui::template::{EditorTemplateAdapter, EditorTemplateRegistry};
use crate::ui::template_runtime::SlintUiProjection;
use zircon_runtime::ui::template::UiTemplateNode;

use super::{projection::project_instance, runtime_host::EditorUiHostRuntimeError};

pub(super) fn project_pane_body(
    template_registry: &EditorTemplateRegistry,
    template_adapter: &EditorTemplateAdapter,
    body: &PaneBodyPresentation,
) -> Result<SlintUiProjection, EditorUiHostRuntimeError> {
    let mut instance = template_registry
        .instantiate(&body.document_id)
        .map_err(EditorUiHostRuntimeError::from)?;
    inject_pane_body_attributes(&mut instance.root, body);
    append_hybrid_slot_anchor(&mut instance.root, body);
    project_instance(&body.document_id, &instance, template_adapter)
}

fn inject_pane_body_attributes(root: &mut UiTemplateNode, body: &PaneBodyPresentation) {
    root.attributes.insert(
        "pane_document_id".to_string(),
        Value::String(body.document_id.clone()),
    );
    root.attributes.insert(
        "pane_payload_kind".to_string(),
        Value::String(format!("{:?}", body.payload_kind)),
    );
    root.attributes.insert(
        "pane_route_namespace".to_string(),
        Value::String(format!("{:?}", body.route_namespace)),
    );
    root.attributes.insert(
        "pane_interaction_mode".to_string(),
        Value::String(format!("{:?}", body.interaction_mode)),
    );
    inject_payload_attributes(root, &body.payload);
}

fn inject_payload_attributes(root: &mut UiTemplateNode, payload: &PanePayload) {
    match payload {
        PanePayload::ConsoleV1(payload) => {
            root.attributes.insert(
                "payload_status_text".to_string(),
                Value::String(payload.status_text.clone()),
            );
        }
        PanePayload::InspectorV1(payload) => {
            root.attributes.insert(
                "payload_node_id".to_string(),
                Value::Integer(i64::try_from(payload.node_id).unwrap_or(i64::MAX)),
            );
            root.attributes.insert(
                "payload_name".to_string(),
                Value::String(payload.name.clone()),
            );
            root.attributes.insert(
                "payload_parent".to_string(),
                Value::String(payload.parent.clone()),
            );
            root.attributes.insert(
                "payload_translation".to_string(),
                Value::Array(
                    payload
                        .translation
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect(),
                ),
            );
            root.attributes.insert(
                "payload_translation_x".to_string(),
                Value::String(payload.translation[0].clone()),
            );
            root.attributes.insert(
                "payload_translation_y".to_string(),
                Value::String(payload.translation[1].clone()),
            );
            root.attributes.insert(
                "payload_translation_z".to_string(),
                Value::String(payload.translation[2].clone()),
            );
            root.attributes.insert(
                "payload_delete_enabled".to_string(),
                Value::Boolean(payload.delete_enabled),
            );
        }
        PanePayload::HierarchyV1(payload) => {
            root.attributes.insert(
                "payload_node_count".to_string(),
                Value::Integer(i64::try_from(payload.nodes.len()).unwrap_or(i64::MAX)),
            );
            root.attributes.insert(
                "payload_nodes".to_string(),
                Value::Array(
                    payload
                        .nodes
                        .iter()
                        .map(|node| {
                            let mut table = Table::new();
                            table.insert(
                                "node_id".to_string(),
                                Value::Integer(i64::try_from(node.node_id).unwrap_or(i64::MAX)),
                            );
                            table.insert("name".to_string(), Value::String(node.name.clone()));
                            table
                                .insert("depth".to_string(), Value::Integer(i64::from(node.depth)));
                            table.insert("selected".to_string(), Value::Boolean(node.selected));
                            Value::Table(table)
                        })
                        .collect(),
                ),
            );
        }
        PanePayload::AnimationSequenceV1(payload) => {
            root.attributes.insert(
                "payload_mode".to_string(),
                Value::String(payload.mode.clone()),
            );
            root.attributes.insert(
                "payload_asset_path".to_string(),
                Value::String(payload.asset_path.clone()),
            );
            root.attributes.insert(
                "payload_status".to_string(),
                Value::String(payload.status.clone()),
            );
            root.attributes.insert(
                "payload_selection".to_string(),
                Value::String(payload.selection.clone()),
            );
            root.attributes.insert(
                "payload_current_frame".to_string(),
                Value::Integer(i64::from(payload.current_frame)),
            );
            root.attributes.insert(
                "payload_timeline_start_frame".to_string(),
                Value::Integer(i64::from(payload.timeline_start_frame)),
            );
            root.attributes.insert(
                "payload_timeline_end_frame".to_string(),
                Value::Integer(i64::from(payload.timeline_end_frame)),
            );
            root.attributes.insert(
                "payload_playback_label".to_string(),
                Value::String(payload.playback_label.clone()),
            );
            root.attributes.insert(
                "payload_track_items".to_string(),
                string_array(&payload.track_items),
            );
        }
        PanePayload::AnimationGraphV1(payload) => {
            root.attributes.insert(
                "payload_mode".to_string(),
                Value::String(payload.mode.clone()),
            );
            root.attributes.insert(
                "payload_asset_path".to_string(),
                Value::String(payload.asset_path.clone()),
            );
            root.attributes.insert(
                "payload_status".to_string(),
                Value::String(payload.status.clone()),
            );
            root.attributes.insert(
                "payload_selection".to_string(),
                Value::String(payload.selection.clone()),
            );
            root.attributes.insert(
                "payload_parameter_items".to_string(),
                string_array(&payload.parameter_items),
            );
            root.attributes.insert(
                "payload_node_items".to_string(),
                string_array(&payload.node_items),
            );
            root.attributes.insert(
                "payload_state_items".to_string(),
                string_array(&payload.state_items),
            );
            root.attributes.insert(
                "payload_transition_items".to_string(),
                string_array(&payload.transition_items),
            );
        }
        PanePayload::RuntimeDiagnosticsV1(payload) => {
            root.attributes.insert(
                "payload_summary".to_string(),
                Value::String(payload.summary.clone()),
            );
            root.attributes.insert(
                "payload_render_status".to_string(),
                Value::String(payload.render_status.clone()),
            );
            root.attributes.insert(
                "payload_physics_status".to_string(),
                Value::String(payload.physics_status.clone()),
            );
            root.attributes.insert(
                "payload_animation_status".to_string(),
                Value::String(payload.animation_status.clone()),
            );
            root.attributes.insert(
                "payload_detail_items".to_string(),
                string_array(&payload.detail_items),
            );
        }
    }
}

fn append_hybrid_slot_anchor(root: &mut UiTemplateNode, body: &PaneBodyPresentation) {
    let Some((control_id, slot_name)) = hybrid_slot_anchor(body) else {
        return;
    };

    let mut attributes = std::collections::BTreeMap::new();
    attributes.insert(
        "slot_name".to_string(),
        Value::String(slot_name.to_string()),
    );
    attributes.insert(
        "pane_document_id".to_string(),
        Value::String(body.document_id.clone()),
    );
    attributes.insert(
        "pane_payload_kind".to_string(),
        Value::String(format!("{:?}", body.payload_kind)),
    );
    attributes.insert(
        "pane_route_namespace".to_string(),
        Value::String(format!("{:?}", body.route_namespace)),
    );
    attributes.insert(
        "pane_interaction_mode".to_string(),
        Value::String(format!("{:?}", body.interaction_mode)),
    );

    root.children.push(UiTemplateNode {
        component: Some("HybridSlotAnchor".to_string()),
        control_id: Some(control_id.to_string()),
        attributes,
        ..Default::default()
    });
}

fn hybrid_slot_anchor(body: &PaneBodyPresentation) -> Option<(&'static str, &'static str)> {
    match &body.payload {
        PanePayload::HierarchyV1(_) => Some(("HierarchyTreeSlotAnchor", "hierarchy_tree_slot")),
        PanePayload::AnimationSequenceV1(_) => {
            Some(("AnimationTimelineSlotAnchor", "animation_timeline_slot"))
        }
        PanePayload::AnimationGraphV1(_) => Some((
            "AnimationGraphCanvasSlotAnchor",
            "animation_graph_canvas_slot",
        )),
        _ => None,
    }
}

fn string_array(items: &[String]) -> Value {
    Value::Array(items.iter().cloned().map(Value::String).collect())
}
