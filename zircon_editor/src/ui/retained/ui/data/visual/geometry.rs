use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};

use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::ui::pane_data_conversion::pane_component_projection::host_template_node;
use crate::ui::template_runtime::RetainedUiHostModel;

use super::contract::{canonical_hash, ReviewCase};
use super::source_identity::{AuthoredNodeIdentity, SourceIdentityIndex};

pub(super) struct GeometryEvidence {
    pub(super) value: Value,
    pub(super) issues: Vec<String>,
}

pub(super) fn build(
    model: &RetainedUiHostModel,
    identity_index: &SourceIdentityIndex,
    case_value: &Value,
    case: &ReviewCase,
) -> Result<GeometryEvidence, String> {
    let mut issues = Vec::new();
    let mut identities = BTreeMap::<String, AuthoredNodeIdentity>::new();
    for node in &model.nodes {
        if node.control_id.is_none() {
            if node_is_visible(node) && node.frame.width > 0.0 && node.frame.height > 0.0 {
                issues.push(format!(
                    "visible retained node {} has no authored control identity",
                    node.node_id
                ));
            }
            continue;
        }
        match identity_index.resolve(node) {
            Ok(identity) => {
                identities.insert(node.node_id.clone(), identity.clone());
            }
            Err(error) => issues.push(error),
        }
    }

    let mut semantic_nodes = Vec::new();
    let mut host_nodes = Vec::new();
    for node in &model.nodes {
        let Some(identity) = identities.get(&node.node_id) else {
            continue;
        };
        let Some(projected) = host_template_node(node.clone()) else {
            issues.push(format!(
                "authored node {} could not produce a native control projection",
                node.node_id
            ));
            continue;
        };
        if node.parent_id.is_some()
            && (node.parent_source_path.is_none()
                || node.parent_source_node_id.is_none()
                || node.parent_instance_path.is_none())
        {
            issues.push(format!(
                "authored parent identity is incomplete for retained node {}",
                node.node_id
            ));
        }
        let parent_instance_path = node
            .parent_instance_path
            .as_deref()
            .map(serde_json::to_string)
            .transpose()
            .unwrap_or_else(|_| {
                issues.push(format!(
                    "authored parent instance identity is invalid for retained node {}",
                    node.node_id
                ));
                None
            });
        let clip_bounds = node.clip_frame.map(
            |clip| json!({"x": clip.x, "y": clip.y, "width": clip.width, "height": clip.height}),
        );
        let text = (!projected.text.is_empty()).then(|| projected.text.to_string());
        let visible = node_is_visible(node);
        let style_inventory = incomplete_style_inventory();
        semantic_nodes.push(json!({
            "nodeId": node.node_id,
            "sourcePath": identity.source_path,
            "sourceNodeId": identity.source_node_id,
            "instancePath": identity.instance_path,
            "hostNodeId": node.node_id,
            "parentNodeId": node.parent_id,
            "parentSourcePath": node.parent_source_path,
            "parentSourceNodeId": node.parent_source_node_id,
            "parentInstancePath": parent_instance_path,
            "controlId": identity.control_id,
            "propertyFieldId": node.attributes.get("inspector_property_field_id").and_then(toml::Value::as_str),
            "itemKey": node.attributes.get("inspector_property_item_key").and_then(toml::Value::as_str),
            "component": node.component,
            "visible": visible,
            "detached": node_is_detached(node),
            "clip": node.clip_frame.is_some(),
            "clipBounds": clip_bounds,
            "text": text,
            "bounds": {
                "x": node.frame.x,
                "y": node.frame.y,
                "width": node.frame.width,
                "height": node.frame.height,
            },
            "effectiveStyle": {
                "complete": false,
                "properties": {},
            },
            "styleInventory": style_inventory,
        }));
        host_nodes.push(json!({
            "hostNodeId": node.node_id,
            "hostParentNodeId": node.parent_id,
            "nodeId": node.node_id,
            "sourcePath": identity.source_path,
            "sourceNodeId": identity.source_node_id,
            "instancePath": identity.instance_path,
            "parentNodeId": node.parent_id,
            "parentSourcePath": node.parent_source_path,
            "parentSourceNodeId": node.parent_source_node_id,
            "parentInstancePath": parent_instance_path,
            "controlId": identity.control_id,
            "propertyFieldId": node.attributes.get("inspector_property_field_id").and_then(toml::Value::as_str),
            "itemKey": node.attributes.get("inspector_property_item_key").and_then(toml::Value::as_str),
            "component": node.component,
            "bounds": {
                "x": node.frame.x,
                "y": node.frame.y,
                "width": node.frame.width,
                "height": node.frame.height,
            },
            "text": text,
            "visible": visible,
            "detached": node_is_detached(node),
            "clip": node.clip_frame.is_some(),
            "clipBounds": clip_bounds,
            "effectiveStyle": {
                "complete": false,
                "properties": {},
            },
            "styleInventory": style_inventory,
        }));
    }

    if semantic_nodes.is_empty() {
        issues.push("retained host produced no source-identified semantic nodes".into());
    }
    let mut source_keys = BTreeSet::new();
    for node in &semantic_nodes {
        let key = (
            node["sourcePath"].as_str().unwrap_or_default().to_owned(),
            node["sourceNodeId"].as_str().unwrap_or_default().to_owned(),
            node["instancePath"].as_str().unwrap_or_default().to_owned(),
        );
        if !source_keys.insert(key.clone()) {
            issues.push(format!(
                "source semantic identity is duplicated: {}#{}@{}",
                key.0, key.1, key.2
            ));
        }
    }

    Ok(GeometryEvidence {
        value: json!({
            "renderer": "zircon-editor-retained-host",
            "case": case_value,
            "caseSha256": canonical_hash(case_value)?,
            "caseId": case.id,
            "coordinateSpace": "logical",
            "dpi": case.dpi,
            "windowMetrics": {
                "logicalSize": {"width": case.viewport.width, "height": case.viewport.height},
                "physicalSize": {
                    "width": case.physical_viewport()?.width,
                    "height": case.physical_viewport()?.height,
                },
            },
            "semanticAudit": {
                "complete": issues.is_empty(),
                "expectedNodeCount": semantic_nodes.len(),
            },
            "layout": { "semanticNodes": semantic_nodes },
            "nodes": host_nodes,
        }),
        issues,
    })
}

pub(super) fn build_product_presentation(
    presentation: &HostWindowPresentationData,
    identity_index: &SourceIdentityIndex,
    case_value: &Value,
    case: &ReviewCase,
) -> Result<GeometryEvidence, String> {
    let dpi = case.dpi as f32;
    if !dpi.is_finite() || dpi <= 0.0 {
        return Err("product presentation geometry needs a finite positive DPI".into());
    }
    let mut issues = Vec::new();
    let mut semantic_nodes = Vec::new();
    let mut host_nodes = Vec::new();
    let mut source_keys = BTreeSet::new();

    for model in presentation.paint_node_models() {
        for row_index in 0..model.row_count() {
            let Some(row) = model.get(row_index) else {
                issues.push(format!("product paint model omitted row {row_index}"));
                continue;
            };
            let frame = logical_frame(&row.frame, dpi);
            let clip_frame = row
                .has_clip_frame
                .then(|| logical_frame(&row.clip_frame, dpi));
            let visible = frame_is_visible(&frame, clip_frame.as_ref());
            if row.control_id.is_empty() {
                if visible {
                    issues.push(format!(
                        "visible product template row {} has no authored control id",
                        row.node_id
                    ));
                }
                continue;
            }
            let identity = match identity_index.resolve_template_row(
                row.source_path.as_str(),
                row.source_node_id.as_str(),
                row.control_id.as_str(),
                row.instance_path.as_str(),
            ) {
                Ok(identity) => identity,
                Err(error) => {
                    issues.push(format!("product template row {}: {error}", row.node_id));
                    continue;
                }
            };
            let has_parent = !row.parent_node_id.is_empty();
            let parent_source_path = nonempty(&row.parent_source_path);
            let parent_source_node_id = nonempty(&row.parent_source_node_id);
            let parent_instance_path = nonempty(&row.parent_instance_path);
            if has_parent
                && (parent_source_path.is_none()
                    || parent_source_node_id.is_none()
                    || parent_instance_path.is_none())
            {
                issues.push(format!(
                    "product authored parent identity is incomplete for row {}",
                    row.node_id
                ));
            }
            let clip = clip_frame.as_ref().map(frame_json);
            let text = (!row.text.is_empty()).then(|| row.text.to_string());
            let style_inventory = incomplete_style_inventory();
            let logical_bounds = frame_json(&frame);
            let generated_parent = nonempty(&row.parent_node_id);
            let key = (
                identity.source_path.clone(),
                identity.source_node_id.clone(),
                identity.instance_path.clone(),
            );
            if !source_keys.insert(key.clone()) {
                issues.push(format!(
                    "product source semantic identity is duplicated: {}#{}@{}",
                    key.0, key.1, key.2
                ));
            }
            semantic_nodes.push(json!({
                "nodeId": row.node_id,
                "sourcePath": identity.source_path,
                "sourceNodeId": identity.source_node_id,
                "instancePath": identity.instance_path,
                "hostNodeId": row.node_id,
                "parentNodeId": generated_parent,
                "parentSourcePath": parent_source_path,
                "parentSourceNodeId": parent_source_node_id,
                "parentInstancePath": parent_instance_path,
                "controlId": identity.control_id,
            "propertyFieldId": nonempty(&row.inspector_property_field_id),
            "itemKey": nonempty(&row.inspector_property_item_key),
                "component": identity.component,
                "visible": visible,
                "detached": false,
                "clip": clip_frame.is_some(),
                "clipBounds": clip,
                "text": text,
                "bounds": logical_bounds,
                "effectiveStyle": {"complete": false, "properties": {}},
                "styleInventory": style_inventory,
            }));
            host_nodes.push(json!({
                "hostNodeId": row.node_id,
                "hostParentNodeId": generated_parent,
                "nodeId": row.node_id,
                "sourcePath": identity.source_path,
                "sourceNodeId": identity.source_node_id,
                "instancePath": identity.instance_path,
                "parentNodeId": generated_parent,
                "parentSourcePath": parent_source_path,
                "parentSourceNodeId": parent_source_node_id,
                "parentInstancePath": parent_instance_path,
                "controlId": identity.control_id,
            "propertyFieldId": nonempty(&row.inspector_property_field_id),
            "itemKey": nonempty(&row.inspector_property_item_key),
                "component": identity.component,
                "bounds": logical_bounds,
                "text": text,
                "visible": visible,
                "detached": false,
                "clip": clip_frame.is_some(),
                "clipBounds": clip,
                "effectiveStyle": {"complete": false, "properties": {}},
                "styleInventory": style_inventory,
            }));
        }
    }

    if semantic_nodes.is_empty() {
        issues.push("product presentation has no source-identified semantic rows".into());
    }
    Ok(GeometryEvidence {
        value: json!({
            "renderer": "zircon-editor-retained-host",
            "case": case_value,
            "caseSha256": canonical_hash(case_value)?,
            "caseId": case.id,
            "coordinateSpace": "logical",
            "dpi": case.dpi,
            "windowMetrics": {
                "logicalSize": {"width": case.viewport.width, "height": case.viewport.height},
                "physicalSize": {
                    "width": case.physical_viewport()?.width,
                    "height": case.physical_viewport()?.height,
                },
            },
            "semanticAudit": {
                "complete": issues.is_empty(),
                "expectedNodeCount": semantic_nodes.len(),
            },
            "layout": {"semanticNodes": semantic_nodes},
            "nodes": host_nodes,
        }),
        issues,
    })
}

#[derive(Clone, Copy)]
struct LogicalFrame {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn logical_frame(
    frame: &crate::ui::retained_host::host_contract::data::TemplateNodeFrameData,
    dpi: f32,
) -> LogicalFrame {
    LogicalFrame {
        x: frame.x / dpi,
        y: frame.y / dpi,
        width: frame.width / dpi,
        height: frame.height / dpi,
    }
}

fn frame_is_visible(frame: &LogicalFrame, clip: Option<&LogicalFrame>) -> bool {
    if ![frame.x, frame.y, frame.width, frame.height]
        .iter()
        .all(|value| value.is_finite())
        || frame.width <= 0.0
        || frame.height <= 0.0
    {
        return false;
    }
    let Some(clip) = clip else { return true };
    clip.width > 0.0
        && clip.height > 0.0
        && frame.x + frame.width > clip.x
        && frame.y + frame.height > clip.y
        && frame.x < clip.x + clip.width
        && frame.y < clip.y + clip.height
}

fn frame_json(frame: &LogicalFrame) -> Value {
    json!({"x": frame.x, "y": frame.y, "width": frame.width, "height": frame.height})
}

fn nonempty(value: &crate::ui::retained_host::primitives::SharedString) -> Option<String> {
    (!value.is_empty()).then(|| value.to_string())
}

fn incomplete_style_inventory() -> Value {
    json!({
        "version": 1,
        "complete": false,
        "properties": {
            "foregroundColor": null,
            "backgroundColor": null,
            "borderColor": null,
            "borderWidth": null,
            "borderRadius": null,
            "opacity": null,
            "boxShadow": null
        }
    })
}

fn node_is_visible(node: &crate::ui::template_runtime::RetainedUiHostNodeProjection) -> bool {
    if node.frame.width <= 0.0 || node.frame.height <= 0.0 {
        return false;
    }
    if let Some(clip) = node.clip_frame {
        if clip.width <= 0.0
            || clip.height <= 0.0
            || node.frame.x + node.frame.width <= clip.x
            || node.frame.y + node.frame.height <= clip.y
            || clip.x + clip.width <= node.frame.x
            || clip.y + clip.height <= node.frame.y
        {
            return false;
        }
    }
    if node
        .attributes
        .get("hidden")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
    {
        return false;
    }
    match node
        .attributes
        .get("visibility")
        .and_then(toml::Value::as_str)
    {
        Some("collapsed" | "hidden") => false,
        _ => true,
    }
}

fn node_is_detached(node: &crate::ui::template_runtime::RetainedUiHostNodeProjection) -> bool {
    node.attributes
        .get("detached")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
