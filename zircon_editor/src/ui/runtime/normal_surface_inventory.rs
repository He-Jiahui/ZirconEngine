use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use serde_json::{json, Value};
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiArrangedNode, UiSurfaceFrame},
};

use super::{RetainedUiHostNodeModel, RetainedUiHostProjection, RetainedUiHostValue};

/// Holds the actual Runtime frame for the lifetime of its frozen observations.
#[derive(Clone)]
pub(crate) struct NormalSurfaceInventory {
    _source_frame: Arc<UiSurfaceFrame>,
    observation: Arc<Value>,
}

#[derive(Clone)]
pub(crate) struct NormalPresentInventory {
    surfaces: Vec<NormalSurfaceInventory>,
}

impl NormalPresentInventory {
    pub(crate) fn new(surfaces: Vec<NormalSurfaceInventory>) -> Self {
        Self { surfaces }
    }

    pub(crate) fn observation(&self) -> Value {
        let complete = !self.surfaces.is_empty()
            && self
                .surfaces
                .iter()
                .all(|surface| surface.observation["inventoryAudit"]["complete"] == true);
        json!({
            "schema": "dev.zircon.editor.normal-arranged-inventory",
            "version": 1,
            "authority": "normal-retained-runtime-surfaces",
            "observed": true,
            "inventoryAudit": { "complete": complete, "surfaceCount": self.surfaces.len() },
            "surfaces": self.surfaces.iter().map(|surface| surface.observation.as_ref()).collect::<Vec<_>>(),
            // The mounted pane/source/style/overlay closure has separate owners.
            // Arranged root coverage alone cannot certify the complete window.
            "complete": false,
            "pending": ["mounted-pane-source-style-overlay-closure"],
        })
    }
}

/// Freeze all arranged nodes, including hidden and control-less decoration.
/// This is preparation metadata, not proof that a node was painted or presented.
pub(crate) fn capture_normal_surface_inventory(
    scope: &str,
    projection: &RetainedUiHostProjection,
    surface: &UiSurface,
    mount_frame_physical: Option<UiFrame>,
    presentation_scale_factor: f32,
) -> NormalSurfaceInventory {
    let current_frame = surface.surface_frame();
    let frame = projection
        .source_surface_frame
        .clone()
        .unwrap_or_else(|| current_frame.clone());
    let mut issues = Vec::new();
    if !projection
        .source_surface_frame
        .as_ref()
        .is_some_and(|source| Arc::ptr_eq(source, &current_frame))
    {
        issues.push("projection/current Runtime frame mismatch".to_owned());
    }
    if !presentation_scale_factor.is_finite() || presentation_scale_factor <= 0.0 {
        issues.push("invalid presentation scale factor".to_owned());
    }

    let mut arranged_by_id = BTreeMap::new();
    for node in frame.arranged_tree.nodes.iter() {
        if arranged_by_id.insert(node.node_id, node).is_some() {
            issues.push(format!("duplicate arranged node {:?}", node.node_id));
        }
    }
    let raw_ids: BTreeSet<_> = surface
        .tree
        .nodes
        .values()
        .map(|node| node.node_id)
        .collect();
    let arranged_ids: BTreeSet<_> = arranged_by_id.keys().copied().collect();
    if raw_ids != arranged_ids || raw_ids.len() != surface.tree.nodes.len() {
        issues.push("raw/arranged node identity sets differ".to_owned());
    }
    if raw_ids.is_empty() {
        issues.push("empty Runtime tree".to_owned());
    }
    let arranged_roots = frame
        .arranged_tree
        .roots
        .iter()
        .copied()
        .collect::<Vec<_>>();
    let roots: BTreeSet<_> = arranged_roots.iter().copied().collect();
    if arranged_roots != surface.tree.roots || roots.len() != arranged_roots.len() {
        issues.push("raw/arranged root sequences differ or contain duplicates".to_owned());
    }
    let mut seen_paths = BTreeSet::new();
    let mut seen_source_identities = BTreeSet::new();
    let mut projected_by_id: BTreeMap<UiNodeId, Vec<&RetainedUiHostNodeModel>> = BTreeMap::new();
    let mut unmatched_projection_nodes = Vec::new();
    for node in &projection.nodes {
        if let Some(id) = node.surface_node_id {
            projected_by_id.entry(id).or_default().push(node);
            if !arranged_ids.contains(&id) {
                issues.push(format!("projection node {:?} has no arranged node", id));
                unmatched_projection_nodes.push(projected_observation(node));
            }
        } else {
            issues.push(format!(
                "projection node {} lacks a Runtime node ID",
                node.node_id
            ));
            unmatched_projection_nodes.push(projected_observation(node));
        }
    }

    let mut nodes = Vec::with_capacity(frame.arranged_tree.nodes.len());
    for arranged in frame.arranged_tree.nodes.iter() {
        let id = arranged.node_id;
        if !seen_paths.insert(&arranged.node_path.0) {
            issues.push(format!(
                "duplicate arranged node path {}",
                arranged.node_path.0
            ));
        }
        if !valid_frame(arranged.frame) || !valid_frame(arranged.clip_frame) {
            issues.push(format!("invalid arranged geometry {:?}", id));
        }
        let child_ids: BTreeSet<_> = arranged.children.iter().copied().collect();
        if child_ids.len() != arranged.children.len()
            || arranged.children.iter().any(|child| {
                arranged_by_id
                    .get(child)
                    .is_none_or(|child| child.parent != Some(id))
            })
            || (arranged.parent.is_none() != roots.contains(&id))
        {
            issues.push(format!("invalid arranged child/root topology {:?}", id));
        }
        let live = surface.tree.node(id);
        let metadata = live.and_then(|node| node.template_metadata.as_ref());
        let projected = match projected_by_id.get(&id).map(Vec::as_slice) {
            Some([node]) => Some(*node),
            Some(duplicates) => {
                issues.push(format!("duplicate projection node {:?}", id));
                unmatched_projection_nodes
                    .extend(duplicates.iter().map(|node| projected_observation(node)));
                None
            }
            None => {
                issues.push(format!("missing projection node {:?}", id));
                None
            }
        };
        match live {
            Some(live)
                if live.node_path == arranged.node_path
                    && live.parent == arranged.parent
                    && live.children == arranged.children
                    && live.layout_cache.frame == arranged.frame
                    && live.effective_visibility() == arranged.visibility => {}
            _ => issues.push(format!(
                "raw/arranged topology or visibility mismatch {:?}",
                id
            )),
        }
        if metadata.is_none() {
            issues.push(format!("missing template metadata {:?}", id));
        }
        let inspector_repeat = inspector_repeat_observation(surface, id, &mut issues);
        if let Some(metadata) = metadata {
            if metadata.component.is_empty()
                || metadata.source_path.as_deref().is_none_or(str::is_empty)
                || metadata.source_node_id.as_deref().is_none_or(str::is_empty)
                || metadata.instance_path.is_none()
            {
                issues.push(format!("incomplete template source identity {:?}", id));
            }
            let mut source_identity = json!([
                metadata.source_path,
                metadata.source_node_id,
                metadata.instance_path
            ]);
            if let Some(repeat) = inspector_repeat.as_ref() {
                source_identity.as_array_mut().unwrap().push(json!([
                    repeat["ownerSourcePath"],
                    repeat["ownerSourceNodeId"],
                    repeat["ownerInstancePath"],
                    repeat["itemKey"]
                ]));
            }
            if !seen_source_identities.insert(source_identity.to_string()) {
                issues.push(format!("duplicate template source identity {:?}", id));
            }
            if metadata.control_id != arranged.control_id {
                issues.push(format!("raw/arranged control identity mismatch {:?}", id));
            }
            if let Some(projected) = projected {
                let parent_path = arranged
                    .parent
                    .and_then(|id| arranged_by_id.get(&id))
                    .map(|node| node.node_path.0.as_str());
                if projected.node_id != arranged.node_path.0
                    || projected.parent_id.as_deref() != parent_path
                    || projected.control_id != arranged.control_id
                    || projected.component != metadata.component
                    || projected.source_path != metadata.source_path
                    || projected.source_node_id != metadata.source_node_id
                    || projected.instance_path != metadata.instance_path
                    || projected.frame != arranged.frame
                    || projected
                        .clip_frame
                        .is_some_and(|clip| clip != arranged.clip_frame)
                {
                    issues.push(format!(
                        "projection/source identity or geometry mismatch {:?}",
                        id
                    ));
                }
                for key in [
                    "text",
                    "label",
                    "placeholder",
                    "value",
                    "value_text",
                    "options",
                    "items",
                    "entries",
                    "selected",
                    "checked",
                    "popup_open",
                    "visibility",
                    "visible",
                    "disabled",
                    "enabled",
                    "selection_state",
                    "detached",
                    "inspector_property_item_key",
                    "inspector_property_field_id",
                ] {
                    if projected.properties.get(key).map(projected_value)
                        != metadata.attributes.get(key).map(|value| json!(value))
                    {
                        issues.push(format!(
                            "projection/source property mismatch {:?}: {}",
                            id, key
                        ));
                    }
                }
            }
        }
        let ancestor_visible = visible_ancestry(arranged, &arranged_by_id, &roots);
        if ancestor_visible.is_none() {
            issues.push(format!("missing or cyclic arranged ancestry {:?}", id));
        }
        let parent = arranged.parent.map(|id| {
            let raw_parent = surface.tree.node(id);
            let source = raw_parent.and_then(|node| node.template_metadata.as_ref());
            json!({
                "nodeId": raw_parent.map(|node| &node.node_path.0),
                "sourcePath": source.and_then(|source| source.source_path.as_ref()),
                "sourceNodeId": source.and_then(|source| source.source_node_id.as_ref()),
                "instancePath": source.and_then(|source| source.instance_path.as_ref()),
            })
        });
        nodes.push(json!({
            "surfaceNodeId": id,
            "nodeId": arranged.node_path.0,
            "parent": parent,
            "children": arranged.children,
            "sourcePath": metadata.and_then(|source| source.source_path.as_ref()),
            "sourceNodeId": metadata.and_then(|source| source.source_node_id.as_ref()),
            "instancePath": metadata.and_then(|source| source.instance_path.as_ref()),
            "itemKey": metadata.and_then(|source| source.attributes.get("inspector_property_item_key")).and_then(toml::Value::as_str),
            "propertyFieldId": metadata.and_then(|source| source.attributes.get("inspector_property_field_id")).and_then(toml::Value::as_str),
            "inspectorRepeat": inspector_repeat,
            "component": metadata.map(|source| &source.component),
            "controlId": arranged.control_id,
            "bounds": arranged.frame,
            "arrangedClipBounds": arranged.clip_frame,
            "projectedClipBounds": projected.and_then(|node| node.clip_frame),
            "clipToBounds": arranged.clip_to_bounds,
            "selfVisible": arranged.is_render_visible(),
            "ancestorVisible": ancestor_visible,
            "visible": ancestor_visible.map(|visible| visible && intersects(arranged.frame, arranged.clip_frame)),
            "declaredDetached": metadata.map(|source| source.attributes.get("detached").and_then(toml::Value::as_bool).unwrap_or(false)),
            "detached": ancestor_visible.and_then(|_| metadata.map(|source| source.attributes.get("detached").and_then(toml::Value::as_bool).unwrap_or(false))),
            "enabled": arranged.enabled,
            "text": projected.and_then(|node| node.text.as_ref()),
            "textAuthority": "retained-host-projection",
            "rawMetadata": metadata,
            "projected": projected.map(projected_observation),
        }));
    }
    let unarranged_nodes = surface.tree.nodes.values()
        .filter(|node| !arranged_ids.contains(&node.node_id))
        .map(|node| json!({ "surfaceNodeId": node.node_id, "nodeId": node.node_path.0,
            "parent": node.parent, "children": node.children, "rawMetadata": node.template_metadata }))
        .collect::<Vec<_>>();
    let observation = json!({
        "schema": "dev.zircon.editor.normal-surface-arranged-inventory",
        "version": 1,
        "scope": scope,
        "documentId": projection.document_id,
        "coordinateSpace": "surface-logical",
        "mountFramePhysical": mount_frame_physical,
        "presentationScaleFactor": presentation_scale_factor,
        "sourceFrameGeneration": frame.generation,
        "frameAuthority": if projection.source_surface_frame.is_some() { "retained-host-projection" } else { "unbound-current-runtime-frame" },
        "rootIds": arranged_roots,
        "rawNodeCount": surface.tree.nodes.len(),
        "arrangedNodeCount": frame.arranged_tree.nodes.len(),
        "projectedNodeCount": projection.nodes.len(),
        "inventoryAudit": { "complete": issues.is_empty(), "issues": issues },
        "nodes": nodes,
        "unarrangedNodes": unarranged_nodes,
        "unmatchedProjectionNodes": unmatched_projection_nodes,
    });
    NormalSurfaceInventory {
        _source_frame: frame,
        observation: Arc::new(observation),
    }
}

/// Inspector rows reuse one authored leaf. Only a current Runtime assignment
/// supplies the additional model identity; the physical slot stays diagnostic.
fn inspector_repeat_observation(
    surface: &UiSurface,
    node_id: UiNodeId,
    issues: &mut Vec<String>,
) -> Option<Value> {
    let node = surface.tree.node(node_id)?;
    let metadata = node.template_metadata.as_ref()?;
    let key_value = metadata.attributes.get("inspector_property_item_key");
    let field_value = metadata.attributes.get("inspector_property_field_id");
    let owner_id = node.parent;
    let binding = owner_id.and_then(|owner| surface.virtual_list_binding_for_node(owner, node_id));
    if key_value.is_none() && field_value.is_none() && binding.is_none() {
        return None;
    }
    let key = key_value.and_then(toml::Value::as_str);
    let field = field_value.and_then(toml::Value::as_str);
    let canonical_key = key.and_then(|key| {
        key.parse::<u128>()
            .ok()
            .filter(|value| value.to_string() == key)
    });
    if canonical_key.is_none() || field.is_none_or(|field| field.trim().is_empty()) {
        issues.push(format!("invalid Inspector property identity {:?}", node_id));
    }
    let owner = owner_id
        .and_then(|id| surface.tree.node(id))
        .and_then(|owner| owner.template_metadata.as_ref());
    if owner.is_none_or(|owner| {
        owner.source_path.as_deref().is_none_or(str::is_empty)
            || owner.source_node_id.as_deref().is_none_or(str::is_empty)
            || owner.instance_path.is_none()
    }) {
        issues.push(format!("missing Inspector property owner {:?}", node_id));
    }
    let binding_current = binding.is_some_and(|binding| {
        surface.virtual_list_binding_is_current(node_id, binding)
            && canonical_key == Some(binding.item_key.get())
    });
    if !binding_current {
        issues.push(format!(
            "Inspector property Runtime binding mismatch {:?}",
            node_id
        ));
    }
    Some(json!({
        "ownerSourcePath": owner.and_then(|owner| owner.source_path.as_ref()),
        "ownerSourceNodeId": owner.and_then(|owner| owner.source_node_id.as_ref()),
        "ownerInstancePath": owner.and_then(|owner| owner.instance_path.as_ref()),
        "itemKey": key,
        "propertyFieldId": field,
        "runtimeBindingCurrent": binding_current,
        "ownerSurfaceNodeId": owner_id,
        "slotRootId": binding.map(|binding| binding.slot_root_id),
        "assignmentGeneration": binding.map(|binding| binding.assignment_generation),
    }))
}

fn visible_ancestry(
    node: &UiArrangedNode,
    nodes: &BTreeMap<UiNodeId, &UiArrangedNode>,
    roots: &BTreeSet<UiNodeId>,
) -> Option<bool> {
    let mut next = Some(node.node_id);
    let mut seen = BTreeSet::new();
    let mut visible = true;
    while let Some(id) = next {
        if !seen.insert(id) {
            return None;
        }
        let node = nodes.get(&id)?;
        visible &= node.is_render_visible();
        if node.parent.is_none() && !roots.contains(&id) {
            return None;
        }
        next = node.parent;
    }
    Some(visible)
}

fn valid_frame(frame: UiFrame) -> bool {
    [frame.x, frame.y, frame.width, frame.height]
        .iter()
        .all(|value| value.is_finite())
        && frame.width >= 0.0
        && frame.height >= 0.0
}

fn intersects(a: UiFrame, b: UiFrame) -> bool {
    [a.x, a.y, a.width, a.height, b.x, b.y, b.width, b.height]
        .iter()
        .all(|value| value.is_finite())
        && a.width > 0.0
        && a.height > 0.0
        && b.width > 0.0
        && b.height > 0.0
        && a.x < b.x + b.width
        && b.x < a.x + a.width
        && a.y < b.y + b.height
        && b.y < a.y + a.height
}

fn projected_observation(node: &RetainedUiHostNodeModel) -> Value {
    json!({
        "nodeId": node.node_id, "surfaceNodeId": node.surface_node_id,
        "parentId": node.parent_id, "component": node.component, "controlId": node.control_id,
        "sourcePath": node.source_path, "sourceNodeId": node.source_node_id,
        "instancePath": node.instance_path, "bounds": node.frame, "clipBounds": node.clip_frame,
        "text": node.text, "icon": node.icon, "disabled": node.disabled,
        "properties": node.properties.iter().map(|(key, value)| (key.clone(), projected_value(value))).collect::<serde_json::Map<_, _>>(),
    })
}

fn projected_value(value: &RetainedUiHostValue) -> Value {
    match value {
        RetainedUiHostValue::String(value) | RetainedUiHostValue::Datetime(value) => json!(value),
        RetainedUiHostValue::Integer(value) => json!(value),
        RetainedUiHostValue::Float(value) => json!(value),
        RetainedUiHostValue::Bool(value) => json!(value),
        RetainedUiHostValue::Array(values) => {
            Value::Array(values.iter().map(projected_value).collect())
        }
        RetainedUiHostValue::Table(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), projected_value(value)))
                .collect(),
        ),
    }
}

#[cfg(test)]
#[path = "tests/normal_surface_inventory.rs"]
mod tests;
