use std::collections::BTreeMap;

use toml::{Table, Value};

use crate::ui::layouts::windows::workbench_host_window::{PaneBodyPresentation, PanePayload};
use crate::ui::template_runtime::RetainedUiNodeProjection;
use zircon_runtime_interface::ui::component::{UiComponentProjectionPatch, UiValue};

pub(super) fn inject_pane_projection_attributes(
    root: &mut RetainedUiNodeProjection,
    body: &PaneBodyPresentation,
) {
    let pane_attributes = pane_body_attributes(body);
    inject_module_plugin_detail_patches(root, body);
    let Some((control_id, slot_name)) = hybrid_slot_anchor(body) else {
        root.attributes.extend(pane_attributes);
        inject_template_v2_component_patches(root, &body.payload);
        return;
    };
    root.attributes.extend(pane_attributes.clone());
    inject_template_v2_component_patches(root, &body.payload);
    append_hybrid_slot_anchor_projection(root, control_id, slot_name, pane_attributes);
}

pub(super) fn template_v2_component_patch_attributes(
    body: &PaneBodyPresentation,
) -> BTreeMap<String, BTreeMap<String, Value>> {
    let PanePayload::TemplateV2(payload) = &body.payload else {
        return BTreeMap::new();
    };
    let mut control_attributes = BTreeMap::new();
    for patch in &payload.component_patches {
        let attributes = control_attributes
            .entry(patch.control_id.clone())
            .or_insert_with(BTreeMap::new);
        attributes.extend(
            patch
                .attributes
                .iter()
                .map(|(key, value)| (key.clone(), ui_value_to_toml(value))),
        );
        attributes.extend(
            patch
                .state_values
                .iter()
                .map(|(key, value)| (key.clone(), ui_value_to_toml(value))),
        );
    }
    control_attributes
}

/// The same control values feed the retained projection and the native surface.
pub(super) fn pane_component_patch_attributes(
    body: &PaneBodyPresentation,
) -> BTreeMap<String, BTreeMap<String, Value>> {
    let PanePayload::ModulePluginsV1(payload) = &body.payload else {
        return template_v2_component_patch_attributes(body);
    };
    let mut controls = BTreeMap::new();
    let open = payload.details.is_some();
    for (control, visible) in [
        ("ModulePluginListPanel", !open),
        ("ModulePluginDetailsBody", open),
    ] {
        controls.insert(
            control.to_owned(),
            BTreeMap::from([
                ("visible".into(), Value::Boolean(visible)),
                (
                    "visibility".into(),
                    Value::String(if visible { "Visible" } else { "Collapsed" }.into()),
                ),
            ]),
        );
    }
    let Some(detail) = payload.details.as_ref() else {
        return controls;
    };
    for (control, text) in [
        ("ModulePluginDetailsTitle", &detail.title),
        ("ModulePluginDetailsBack", &detail.back_label),
        ("ModulePluginDetailsPrevious", &detail.previous_label),
        ("ModulePluginDetailsNext", &detail.next_label),
        ("ModulePluginDetailsPage", &detail.page_label),
        ("ModulePluginFeatureEmpty", &detail.empty_label),
        ("ModulePluginFeatureIdentity", &detail.selected_identity),
        ("ModulePluginFeatureName", &detail.selected_name),
        ("ModulePluginFeatureStates", &detail.selected_states),
        ("ModulePluginFeatureReason", &detail.selected_reason),
        ("ModulePluginDetailsApply", &detail.action_label),
    ] {
        controls.insert(
            control.into(),
            BTreeMap::from([
                ("text".into(), Value::String(text.clone())),
                ("label".into(), Value::String(text.clone())),
            ]),
        );
    }
    let rows = Value::Array(
        detail
            .rows
            .iter()
            .map(|row| {
                Value::Table(Table::from_iter([
                    ("feature_id".into(), Value::String(row.feature_id.clone())),
                    ("label".into(), Value::String(row.label.clone())),
                ]))
            })
            .collect(),
    );
    controls.insert(
        "ModulePluginFeatureRows".into(),
        BTreeMap::from([
            ("rows".into(), rows),
            (
                "row_identity_field".into(),
                Value::String("feature_id".into()),
            ),
            (
                "selected_row_identity".into(),
                Value::String(detail.selected_feature_id.clone()),
            ),
            (
                "value".into(),
                Value::String(detail.selected_feature_id.clone()),
            ),
            (
                "value_text".into(),
                Value::String(detail.selected_feature_id.clone()),
            ),
            (
                "detail_generation".into(),
                Value::String(payload.detail_generation.to_string()),
            ),
            (
                "focused".into(),
                Value::Boolean(payload.focused_control == "features"),
            ),
            ("visible".into(), Value::Boolean(!detail.rows.is_empty())),
            (
                "height".into(),
                Value::Float((detail.rows.len().max(1) * 128) as f64),
            ),
        ]),
    );
    controls
        .get_mut("ModulePluginFeatureEmpty")
        .unwrap()
        .insert("visible".into(), Value::Boolean(detail.rows.is_empty()));
    for (control, focus, disabled) in [
        ("ModulePluginDetailsBack", "back", false),
        ("ModulePluginDetailsPrevious", "previous", detail.page == 0),
        (
            "ModulePluginDetailsNext",
            "next",
            (detail.page + 1) * 8 >= detail.total_features,
        ),
        (
            "ModulePluginDetailsApply",
            "apply",
            detail.action_id.is_empty(),
        ),
    ] {
        let attributes = controls.get_mut(control).unwrap();
        attributes.insert(
            "focused".into(),
            Value::Boolean(payload.focused_control == focus),
        );
        attributes.insert("disabled".into(), Value::Boolean(disabled));
    }
    controls
        .get_mut("ModulePluginDetailsApply")
        .unwrap()
        .insert("intent_id".into(), Value::String(detail.intent_id.clone()));
    controls
}

fn inject_module_plugin_detail_patches(
    root: &mut RetainedUiNodeProjection,
    body: &PaneBodyPresentation,
) {
    if !matches!(&body.payload, PanePayload::ModulePluginsV1(_)) {
        return;
    }
    fn apply(
        node: &mut RetainedUiNodeProjection,
        controls: &mut BTreeMap<String, BTreeMap<String, Value>>,
    ) {
        if let Some(attributes) = node.control_id.as_ref().and_then(|id| controls.remove(id)) {
            node.attributes.extend(attributes);
        }
        for child in &mut node.children {
            apply(child, controls);
        }
    }
    apply(root, &mut pane_component_patch_attributes(body));
}

fn pane_body_attributes(body: &PaneBodyPresentation) -> BTreeMap<String, Value> {
    let mut attributes = BTreeMap::new();
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
    inject_payload_attributes(&mut attributes, &body.payload);
    attributes
}

fn inject_payload_attributes(attributes: &mut BTreeMap<String, Value>, payload: &PanePayload) {
    match payload {
        PanePayload::TemplateV2(payload) => {
            attributes.insert(
                "template_data".to_string(),
                Value::Table(
                    payload
                        .values
                        .iter()
                        .map(|(key, value)| (key.clone(), ui_value_to_toml(value)))
                        .collect(),
                ),
            );
        }
        PanePayload::ConsoleV1(payload) => {
            attributes.insert(
                "payload_console_info_count".to_string(),
                Value::Integer(i64::try_from(payload.output.counts().info).unwrap_or(i64::MAX)),
            );
            attributes.insert(
                "payload_console_warning_count".to_string(),
                Value::Integer(i64::try_from(payload.output.counts().warning).unwrap_or(i64::MAX)),
            );
            attributes.insert(
                "payload_console_error_count".to_string(),
                Value::Integer(i64::try_from(payload.output.counts().error).unwrap_or(i64::MAX)),
            );
            attributes.insert(
                "payload_console_filter".to_string(),
                Value::String(payload.output.filter().as_str().to_string()),
            );
        }
        PanePayload::InspectorV1(payload) => {
            attributes.insert(
                "payload_node_id".to_string(),
                Value::Integer(i64::try_from(payload.node_id).unwrap_or(i64::MAX)),
            );
            attributes.insert(
                "payload_name".to_string(),
                Value::String(payload.name.clone()),
            );
            attributes.insert(
                "payload_parent".to_string(),
                Value::String(payload.parent.clone()),
            );
            attributes.insert(
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
            attributes.insert(
                "payload_translation_x".to_string(),
                Value::String(payload.translation[0].clone()),
            );
            attributes.insert(
                "payload_translation_y".to_string(),
                Value::String(payload.translation[1].clone()),
            );
            attributes.insert(
                "payload_translation_z".to_string(),
                Value::String(payload.translation[2].clone()),
            );
            attributes.insert(
                "payload_delete_enabled".to_string(),
                Value::Boolean(payload.delete_enabled),
            );
        }
        PanePayload::HierarchyV1(payload) => {
            attributes.insert(
                "payload_node_count".to_string(),
                Value::Integer(i64::try_from(payload.nodes.len()).unwrap_or(i64::MAX)),
            );
            attributes.insert(
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
            attributes.insert(
                "payload_mode".to_string(),
                Value::String(payload.mode.clone()),
            );
            attributes.insert(
                "payload_asset_path".to_string(),
                Value::String(payload.asset_path.clone()),
            );
            attributes.insert(
                "payload_status".to_string(),
                Value::String(payload.status.clone()),
            );
            attributes.insert(
                "payload_selection".to_string(),
                Value::String(payload.selection.clone()),
            );
            attributes.insert(
                "payload_current_frame".to_string(),
                Value::Integer(i64::from(payload.current_frame)),
            );
            attributes.insert(
                "payload_timeline_start_frame".to_string(),
                Value::Integer(i64::from(payload.timeline_start_frame)),
            );
            attributes.insert(
                "payload_timeline_end_frame".to_string(),
                Value::Integer(i64::from(payload.timeline_end_frame)),
            );
            attributes.insert(
                "payload_playback_label".to_string(),
                Value::String(payload.playback_label.clone()),
            );
            attributes.insert(
                "payload_track_items".to_string(),
                string_array(&payload.track_items),
            );
        }
        PanePayload::AnimationGraphV1(payload) => {
            attributes.insert(
                "payload_mode".to_string(),
                Value::String(payload.mode.clone()),
            );
            attributes.insert(
                "payload_asset_path".to_string(),
                Value::String(payload.asset_path.clone()),
            );
            attributes.insert(
                "payload_status".to_string(),
                Value::String(payload.status.clone()),
            );
            attributes.insert(
                "payload_selection".to_string(),
                Value::String(payload.selection.clone()),
            );
            attributes.insert(
                "payload_parameter_items".to_string(),
                string_array(&payload.parameter_items),
            );
            attributes.insert(
                "payload_node_items".to_string(),
                string_array(&payload.node_items),
            );
            attributes.insert(
                "payload_state_items".to_string(),
                string_array(&payload.state_items),
            );
            attributes.insert(
                "payload_transition_items".to_string(),
                string_array(&payload.transition_items),
            );
        }
        PanePayload::RuntimeDiagnosticsV1(payload) => {
            attributes.insert(
                "payload_summary".to_string(),
                Value::String(payload.summary.clone()),
            );
            attributes.insert(
                "payload_render_status".to_string(),
                Value::String(payload.render_status.clone()),
            );
            attributes.insert(
                "payload_physics_status".to_string(),
                Value::String(payload.physics_status.clone()),
            );
            attributes.insert(
                "payload_animation_status".to_string(),
                Value::String(payload.animation_status.clone()),
            );
            attributes.insert(
                "payload_detail_items".to_string(),
                string_array(&payload.detail_items),
            );
            attributes.insert(
                "payload_ui_debug_reflector_summary".to_string(),
                Value::String(payload.ui_debug_reflector_summary.clone()),
            );
            attributes.insert(
                "payload_ui_debug_reflector_nodes".to_string(),
                string_array(&payload.ui_debug_reflector_nodes),
            );
            attributes.insert(
                "payload_ui_debug_reflector_details".to_string(),
                string_array(&payload.ui_debug_reflector_details),
            );
            attributes.insert(
                "payload_ui_debug_reflector_sections".to_string(),
                string_array(&payload.ui_debug_reflector_sections),
            );
            attributes.insert(
                "payload_ui_debug_reflector_export_status".to_string(),
                Value::String(payload.ui_debug_reflector_export_status.clone()),
            );
        }
        PanePayload::PerformanceTimelineV1(payload) => {
            attributes.insert(
                "payload_summary".to_string(),
                Value::String(payload.summary.clone()),
            );
            attributes.insert(
                "payload_session_label".to_string(),
                Value::String(payload.session_label.clone()),
            );
            attributes.insert(
                "payload_output_label".to_string(),
                Value::String(payload.output_label.clone()),
            );
            attributes.insert(
                "payload_frame_rows".to_string(),
                Value::Array(
                    payload
                        .frame_rows
                        .iter()
                        .map(|row| {
                            let mut table = Table::new();
                            table.insert("stream".to_string(), Value::String(row.stream.clone()));
                            table.insert("name".to_string(), Value::String(row.name.clone()));
                            table.insert(
                                "frame_index".to_string(),
                                Value::Integer(i64::try_from(row.frame_index).unwrap_or(i64::MAX)),
                            );
                            table.insert(
                                "duration_label".to_string(),
                                Value::String(row.duration_label.clone()),
                            );
                            table.insert(
                                "budget_label".to_string(),
                                Value::String(row.budget_label.clone()),
                            );
                            table
                                .insert("over_budget".to_string(), Value::Boolean(row.over_budget));
                            Value::Table(table)
                        })
                        .collect(),
                ),
            );
            attributes.insert(
                "payload_span_summary_rows".to_string(),
                Value::Array(
                    payload
                        .span_summary_rows
                        .iter()
                        .map(|row| {
                            let mut table = Table::new();
                            table.insert("stream".to_string(), Value::String(row.stream.clone()));
                            table.insert(
                                "category".to_string(),
                                Value::String(row.category.clone()),
                            );
                            table.insert("name".to_string(), Value::String(row.name.clone()));
                            table.insert("path".to_string(), Value::String(row.path.clone()));
                            table.insert(
                                "duration_label".to_string(),
                                Value::String(row.duration_label.clone()),
                            );
                            table.insert("depth".to_string(), Value::Integer(i64::from(row.depth)));
                            Value::Table(table)
                        })
                        .collect(),
                ),
            );
            attributes.insert(
                "payload_hotspot_rows".to_string(),
                Value::Array(
                    payload
                        .hotspot_rows
                        .iter()
                        .map(|row| {
                            let mut table = Table::new();
                            table.insert("stream".to_string(), Value::String(row.stream.clone()));
                            table.insert(
                                "category".to_string(),
                                Value::String(row.category.clone()),
                            );
                            table.insert("name".to_string(), Value::String(row.name.clone()));
                            table.insert("path".to_string(), Value::String(row.path.clone()));
                            table.insert(
                                "total_label".to_string(),
                                Value::String(row.total_label.clone()),
                            );
                            table.insert(
                                "average_label".to_string(),
                                Value::String(row.average_label.clone()),
                            );
                            table.insert(
                                "count_label".to_string(),
                                Value::String(row.count_label.clone()),
                            );
                            Value::Table(table)
                        })
                        .collect(),
                ),
            );
            attributes.insert(
                "payload_capture_controls".to_string(),
                Value::Array(
                    payload
                        .capture_controls
                        .iter()
                        .map(|control| {
                            let mut table = Table::new();
                            table.insert("label".to_string(), Value::String(control.label.clone()));
                            table.insert(
                                "action_id".to_string(),
                                Value::String(control.action_id.clone()),
                            );
                            table.insert("enabled".to_string(), Value::Boolean(control.enabled));
                            Value::Table(table)
                        })
                        .collect(),
                ),
            );
        }
        PanePayload::ModulePluginsV1(payload) => {
            attributes.insert(
                "payload_diagnostics".to_string(),
                Value::String(payload.diagnostics.clone()),
            );
            attributes.insert(
                "payload_plugin_count".to_string(),
                Value::Integer(i64::try_from(payload.plugins.len()).unwrap_or(i64::MAX)),
            );
            attributes.insert(
                "payload_plugins".to_string(),
                Value::Array(
                    payload
                        .plugins
                        .iter()
                        .map(|plugin| {
                            let mut table = Table::new();
                            table.insert(
                                "plugin_id".to_string(),
                                Value::String(plugin.plugin_id.clone()),
                            );
                            table.insert(
                                "display_name".to_string(),
                                Value::String(plugin.display_name.clone()),
                            );
                            table.insert(
                                "package_source".to_string(),
                                Value::String(plugin.package_source.clone()),
                            );
                            table.insert(
                                "load_state".to_string(),
                                Value::String(plugin.load_state.clone()),
                            );
                            table.insert("enabled".to_string(), Value::Boolean(plugin.enabled));
                            table.insert("required".to_string(), Value::Boolean(plugin.required));
                            table.insert(
                                "target_modes".to_string(),
                                Value::String(plugin.target_modes.clone()),
                            );
                            table.insert(
                                "packaging".to_string(),
                                Value::String(plugin.packaging.clone()),
                            );
                            table.insert(
                                "runtime_crate".to_string(),
                                Value::String(plugin.runtime_crate.clone()),
                            );
                            table.insert(
                                "editor_crate".to_string(),
                                Value::String(plugin.editor_crate.clone()),
                            );
                            table.insert(
                                "runtime_capabilities".to_string(),
                                Value::String(plugin.runtime_capabilities.clone()),
                            );
                            table.insert(
                                "editor_capabilities".to_string(),
                                Value::String(plugin.editor_capabilities.clone()),
                            );
                            table.insert(
                                "diagnostics".to_string(),
                                Value::String(plugin.diagnostics.clone()),
                            );
                            Value::Table(table)
                        })
                        .collect(),
                ),
            );
        }
        PanePayload::BuildExportV1(payload) => {
            attributes.insert(
                "payload_diagnostics".to_string(),
                Value::String(payload.diagnostics.clone()),
            );
            attributes.insert(
                "payload_target_count".to_string(),
                Value::Integer(i64::try_from(payload.targets.len()).unwrap_or(i64::MAX)),
            );
            attributes.insert(
                "payload_targets".to_string(),
                Value::Array(
                    payload
                        .targets
                        .iter()
                        .map(|target| {
                            let mut table = Table::new();
                            table.insert(
                                "profile_name".to_string(),
                                Value::String(target.profile_name.clone()),
                            );
                            table.insert(
                                "platform".to_string(),
                                Value::String(target.platform.clone()),
                            );
                            table.insert(
                                "target_mode".to_string(),
                                Value::String(target.target_mode.clone()),
                            );
                            table.insert(
                                "strategies".to_string(),
                                Value::String(target.strategies.clone()),
                            );
                            table
                                .insert("status".to_string(), Value::String(target.status.clone()));
                            table.insert(
                                "diagnostics".to_string(),
                                Value::String(target.diagnostics.clone()),
                            );
                            table.insert("fatal".to_string(), Value::Boolean(target.fatal));
                            Value::Table(table)
                        })
                        .collect(),
                ),
            );
        }
        PanePayload::GeneratedBottomV1(payload) => {
            attributes.insert(
                "payload_status".to_string(),
                Value::String(payload.status.clone()),
            );
        }
        PanePayload::UiComponentShowcaseV1(payload) => {
            attributes.insert(
                "payload_state_summary".to_string(),
                Value::String(payload.state_summary.clone()),
            );
        }
    }
}

fn inject_template_v2_component_patches(
    root: &mut RetainedUiNodeProjection,
    payload: &PanePayload,
) {
    let PanePayload::TemplateV2(payload) = payload else {
        return;
    };
    let mut patches_by_control = BTreeMap::<&str, Vec<&UiComponentProjectionPatch>>::new();
    for patch in &payload.component_patches {
        patches_by_control
            .entry(patch.control_id.as_str())
            .or_default()
            .push(patch);
    }
    apply_component_projection_patches(root, &mut patches_by_control);
}

fn apply_component_projection_patches<'a>(
    node: &mut RetainedUiNodeProjection,
    patches: &mut BTreeMap<&'a str, Vec<&'a UiComponentProjectionPatch>>,
) {
    if let Some(control_id) = node.control_id.as_deref() {
        if let Some(control_patches) = patches.remove(control_id) {
            for patch in control_patches {
                node.attributes.extend(
                    patch
                        .attributes
                        .iter()
                        .map(|(key, value)| (key.clone(), ui_value_to_toml(value))),
                );
                node.attributes.extend(
                    patch
                        .state_values
                        .iter()
                        .map(|(key, value)| (key.clone(), ui_value_to_toml(value))),
                );
            }
        }
    }
    if patches.is_empty() {
        return;
    }
    for child in &mut node.children {
        apply_component_projection_patches(child, patches);
        if patches.is_empty() {
            break;
        }
    }
}

fn ui_value_to_toml(value: &UiValue) -> Value {
    value.to_toml()
}

fn append_hybrid_slot_anchor_projection(
    root: &mut RetainedUiNodeProjection,
    control_id: &'static str,
    slot_name: &'static str,
    mut pane_attributes: BTreeMap<String, Value>,
) {
    pane_attributes.insert(
        "slot_name".to_string(),
        Value::String(slot_name.to_string()),
    );

    root.children.push(RetainedUiNodeProjection {
        component: "HybridSlotAnchor".to_string(),
        control_id: Some(control_id.to_string()),
        source_path: None,
        source_node_id: None,
        instance_path: None,
        attributes: pane_attributes,
        style_tokens: BTreeMap::new(),
        binding_ids: Vec::new(),
        children: Vec::new(),
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
        PanePayload::ModulePluginsV1(payload) if payload.details.is_none() => {
            Some(("ModulePluginListSlotAnchor", "module_plugin_list_slot"))
        }
        PanePayload::BuildExportV1(_) => {
            Some(("BuildExportTargetsSlotAnchor", "build_export_targets_slot"))
        }
        PanePayload::PerformanceTimelineV1(_) => Some((
            "PerformanceTimelineFrameListSlotAnchor",
            "performance_timeline_frame_list",
        )),
        _ => None,
    }
}

fn string_array(items: &[String]) -> Value {
    Value::Array(items.iter().cloned().map(Value::String).collect())
}

#[cfg(test)]
#[path = "tests/pane_payload_project_perf.rs"]
mod performance_tests;
