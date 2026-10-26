//! Publish the UiSurface owner-focus value consumed by ordinary native TreeRows.
//! Normal present builds this stream and its paint evidence from the same immutable
//! presentation. The ordinary row clone keeps the prototype's owner-focus value;
//! native pane pointer focus and keyboard focus-visible are separate contracts.
use serde_json::{json, Value};

use super::super::super::data::{HostWindowPresentationData, TemplatePaneNodeData};

fn identity(node: &TemplatePaneNodeData) -> Option<Value> {
    (!node.source_path.is_empty()
        && !node.source_node_id.is_empty()
        && !node.instance_path.is_empty())
    .then(|| {
        json!({"sourcePath":node.source_path.as_str(),
            "sourceNodeId":node.source_node_id.as_str(),
            "instancePath":node.instance_path.as_str(),
            "controlId":node.control_id.as_str()})
    })
}

fn same_source(value: &Value, node: &TemplatePaneNodeData) -> bool {
    value["sourcePath"].as_str() == Some(node.source_path.as_str())
        && value["sourceNodeId"].as_str() == Some(node.source_node_id.as_str())
        && value["instancePath"].as_str() == Some(node.instance_path.as_str())
}

pub(super) fn publication(
    presentation: &HostWindowPresentationData,
    sources: &[Option<Value>],
    styles: Option<&Vec<Value>>,
) -> Value {
    let mut result = json!({
        "authority":"normal-template-tree-row-painter-input",
        "sourceAuthority":"UiSurface.focus/tree_selection_owner",
        "value":null,"observed":false,"sourceOwner":null,"originComponent":null,
        "sourceOwnerMappingComplete":false,"sourceFrame":null,
        "paintedNativeControlId":null,"submittedOwnerCommandIndices":[],
        "unavailableReason":"missing-or-ambiguous-tree-owner"
    });
    let models = presentation.paint_node_models();
    let owners = models
        .iter()
        .flat_map(|model| model.iter())
        .filter(|node| node.control_id.as_str() == "WorkbenchSceneTree")
        .collect::<Vec<_>>();
    let [owner] = owners.as_slice() else {
        return result;
    };
    let Some(owner_identity) = identity(owner) else {
        return result;
    };
    result["sourceOwner"] = owner_identity;
    result["unavailableReason"] = json!("authored-hierarchy-unavailable");
    let Some(hierarchy) =
        crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_hierarchy(presentation)
    else { return result; };
    let prototype = hierarchy.prototype;
    let Some(origin) = identity(prototype) else {
        return result;
    };
    result["originComponent"] = origin;
    result["unavailableReason"] = json!("ambiguous-row-prototype-or-owner");
    // A different/ambiguous tree or prototype cannot borrow this owner's focus.
    if prototype.parent_node_id != owner.node_id
        || models
            .iter()
            .flat_map(|model| model.iter())
            .filter(|node| node.node_id == prototype.node_id || same_node_source(node, prototype))
            .count()
            != 1
    {
        return result;
    }
    result["unavailableReason"] = json!("missing-source-surface-frame");
    let (Some(owner_frame), Some(owner_id), Some(row_id)) = (
        owner.source_surface_frame.as_ref(),
        owner.surface_node_id,
        prototype.surface_node_id,
    ) else {
        return result;
    };
    result["unavailableReason"] = json!("source-surface-frame-or-focus-mismatch");
    let (Some(actual_owner), Some(actual_row)) = (
        owner_frame.arranged_tree.get(owner_id),
        owner_frame.arranged_tree.get(row_id),
    ) else {
        return result;
    };
    // The submitted owner's immutable surface proves state ownership. A row
    // prototype need not have an independent replayable primitive; sparse
    // projection also retains unchanged nodes' outer frames. Neither condition
    // changes the focus value consumed by the actual native row painter.
    if actual_owner.node_path.0 != owner.node_id.as_str()
        || actual_owner.control_id.as_deref() != Some(owner.control_id.as_str())
        || actual_row.node_path.0 != prototype.node_id.as_str()
        || actual_row.control_id.as_deref() != Some(prototype.control_id.as_str())
        || actual_row.parent != Some(owner_id)
        || !actual_owner.children.contains(&row_id)
        || !actual_owner.is_render_visible()
        || !actual_row.is_render_visible()
        || actual_owner.enabled == owner.disabled
        || actual_row.enabled == prototype.disabled
        || prototype.parent_source_path != owner.source_path
        || prototype.parent_source_node_id != owner.source_node_id
        || prototype.parent_instance_path != owner.instance_path
        || owner.tree_owner_focused != prototype.tree_owner_focused
        || prototype
            .source_surface_frame
            .as_ref()
            .is_some_and(|row_frame| {
                row_frame.tree_id != owner_frame.tree_id
                    || row_frame.arranged_tree.get(row_id) != Some(actual_row)
                    || row_frame.focus_state != owner_frame.focus_state
            })
    {
        return result;
    }
    result["unavailableReason"] = json!("missing-or-ambiguous-painted-prototype");
    let Some(styles) = styles else {
        return result;
    };
    let rows = styles
        .iter()
        .filter(|style| same_source(style, prototype))
        .collect::<Vec<_>>();
    let [row] = rows.as_slice() else {
        return result;
    };
    result["unavailableReason"] = json!("painted-prototype-is-not-a-current-native-entity");
    let Some(entity_id) = row["controlId"]
        .as_str()
        .and_then(|control| control.strip_prefix("NativeHierarchyEntity:"))
    else {
        return result;
    };
    if row["nodeId"].as_str() != Some(format!("{}/entity/{entity_id}", prototype.node_id).as_str())
        || !hierarchy
            .pane
            .hierarchy
            .hierarchy_nodes
            .iter()
            .any(|entity| entity.id.as_str() == entity_id)
    {
        return result;
    }
    result["unavailableReason"] = json!("submitted-owner-command-not-observed-in-source-frame");
    // The owner itself must also be in this exact submitted source frame.
    // This is owner-command admission, not a claim of row primitive identity.
    let indices = sources
        .iter()
        .enumerate()
        .filter_map(|(index, source)| {
            let source = source.as_ref()?;
            (same_source(source, owner)
                && source["controlId"].as_str() == Some(owner.control_id.as_str())
                && source["sourceRef"]["authority"] == "surface-node-owner"
                && source["sourceRef"]["treeId"].as_str() == Some(owner_frame.tree_id.0.as_str())
                && source["sourceRef"]["generation"].as_u64() == Some(owner_frame.generation)
                && source["sourceRef"]["nodeId"].as_u64() == Some(owner_id.0))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    if indices.is_empty() {
        return result;
    }
    result["sourceFrame"] = json!({"treeId":owner_frame.tree_id,
        "generation":owner_frame.generation,"ownerNodeId":owner_id,"rowPrototypeNodeId":row_id});
    result["paintedNativeControlId"] = row["controlId"].clone();
    result["submittedOwnerCommandIndices"] = json!(indices);
    result["sourceOwnerMappingComplete"] = json!(true);
    result["observed"] = json!(prototype.tree_owner_focused.is_some());
    result["value"] = json!(prototype.tree_owner_focused);
    result["unavailableReason"] = if prototype.tree_owner_focused.is_some() {
        Value::Null
    } else {
        json!("owner-focus-not-observed")
    };
    result
}

fn same_node_source(left: &TemplatePaneNodeData, right: &TemplatePaneNodeData) -> bool {
    left.source_path == right.source_path
        && left.source_node_id == right.source_node_id
        && left.instance_path == right.instance_path
}

#[cfg(test)]
#[path = "tests/tree_owner_focus.rs"]
mod tests;
