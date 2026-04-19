use std::collections::BTreeMap;

use crate::ui::{EditorTemplateAdapter, EditorTemplateRegistry};
use zircon_ui::template::UiTemplateNode;
use zircon_ui::{UiSurface, UiTree};

use crate::ui::template_runtime::{
    SlintUiBindingProjection, SlintUiHostBindingProjection, SlintUiHostModel,
    SlintUiHostNodeProjection, SlintUiNodeProjection, SlintUiProjection,
};

use super::runtime_host::EditorUiHostRuntimeError;

pub(super) fn project_document(
    template_registry: &EditorTemplateRegistry,
    template_adapter: &EditorTemplateAdapter,
    document_id: &str,
) -> Result<SlintUiProjection, EditorUiHostRuntimeError> {
    let instance = template_registry
        .instantiate(document_id)
        .map_err(EditorUiHostRuntimeError::from)?;

    let mut bindings = Vec::new();
    let root = project_node(&instance.root, template_adapter, &mut bindings)?;
    Ok(SlintUiProjection {
        document_id: document_id.to_string(),
        root,
        bindings,
    })
}

pub(super) fn build_host_model(
    projection: &SlintUiProjection,
) -> Result<SlintUiHostModel, EditorUiHostRuntimeError> {
    let bindings = projection
        .bindings
        .iter()
        .cloned()
        .map(|binding| (binding.binding_id.clone(), binding))
        .collect::<BTreeMap<_, _>>();
    let mut nodes = Vec::new();
    collect_host_nodes(&projection.root, None, "root", &bindings, &mut nodes)?;
    Ok(SlintUiHostModel {
        document_id: projection.document_id.clone(),
        nodes,
    })
}

pub(super) fn build_host_model_with_surface(
    projection: &SlintUiProjection,
    surface: &UiSurface,
) -> Result<SlintUiHostModel, EditorUiHostRuntimeError> {
    let bindings = projection
        .bindings
        .iter()
        .cloned()
        .map(|binding| (binding.binding_id.clone(), binding))
        .collect::<BTreeMap<_, _>>();
    let mut nodes = Vec::new();
    for root_id in &surface.tree.roots {
        collect_surface_host_nodes(&surface.tree, *root_id, &bindings, &mut nodes)?;
    }
    Ok(SlintUiHostModel {
        document_id: projection.document_id.clone(),
        nodes,
    })
}

fn project_node(
    node: &UiTemplateNode,
    adapter: &EditorTemplateAdapter,
    bindings: &mut Vec<SlintUiBindingProjection>,
) -> Result<SlintUiNodeProjection, EditorUiHostRuntimeError> {
    let mut binding_ids = Vec::new();
    for binding_ref in &node.bindings {
        let binding = adapter
            .resolve_binding(binding_ref)
            .map_err(EditorUiHostRuntimeError::from)?;
        binding_ids.push(binding_ref.id.clone());
        bindings.push(SlintUiBindingProjection {
            binding_id: binding_ref.id.clone(),
            binding,
            route_id: None,
        });
    }

    Ok(SlintUiNodeProjection {
        component: node.component.clone().unwrap_or_default(),
        control_id: node.control_id.clone(),
        attributes: node.attributes.clone(),
        style_tokens: node.style_tokens.clone(),
        binding_ids,
        children: node
            .children
            .iter()
            .map(|child| project_node(child, adapter, bindings))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn collect_host_nodes(
    node: &SlintUiNodeProjection,
    parent_id: Option<&str>,
    node_id: &str,
    bindings: &BTreeMap<String, SlintUiBindingProjection>,
    host_nodes: &mut Vec<SlintUiHostNodeProjection>,
) -> Result<(), EditorUiHostRuntimeError> {
    let node_bindings = node
        .binding_ids
        .iter()
        .map(|binding_id| {
            bindings
                .get(binding_id)
                .map(|binding| SlintUiHostBindingProjection {
                    binding_id: binding.binding_id.clone(),
                    event_kind: binding.binding.path().event_kind,
                    route_id: binding.route_id,
                })
                .ok_or_else(|| EditorUiHostRuntimeError::MissingProjectionBinding {
                    binding_id: binding_id.clone(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    host_nodes.push(SlintUiHostNodeProjection {
        node_id: node_id.to_string(),
        parent_id: parent_id.map(str::to_string),
        component: node.component.clone(),
        control_id: node.control_id.clone(),
        frame: Default::default(),
        clip_frame: None,
        z_index: 0,
        attributes: node.attributes.clone(),
        style_tokens: node.style_tokens.clone(),
        bindings: node_bindings,
    });

    for (index, child) in node.children.iter().enumerate() {
        let child_id = format!("{node_id}.{index}");
        collect_host_nodes(child, Some(node_id), &child_id, bindings, host_nodes)?;
    }
    Ok(())
}

fn collect_surface_host_nodes(
    tree: &UiTree,
    node_id: zircon_ui::event_ui::UiNodeId,
    bindings: &BTreeMap<String, SlintUiBindingProjection>,
    host_nodes: &mut Vec<SlintUiHostNodeProjection>,
) -> Result<(), EditorUiHostRuntimeError> {
    let node = tree
        .node(node_id)
        .expect("surface traversal should only visit valid nodes");
    let metadata = node.template_metadata.as_ref().ok_or_else(|| {
        EditorUiHostRuntimeError::MissingSurfaceMetadata {
            node_path: node.node_path.0.clone(),
        }
    })?;
    let node_bindings = metadata
        .bindings
        .iter()
        .map(|binding_ref| {
            bindings
                .get(&binding_ref.id)
                .map(|binding| SlintUiHostBindingProjection {
                    binding_id: binding.binding_id.clone(),
                    event_kind: binding.binding.path().event_kind,
                    route_id: binding.route_id,
                })
                .ok_or_else(|| EditorUiHostRuntimeError::MissingProjectionBinding {
                    binding_id: binding_ref.id.clone(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    host_nodes.push(SlintUiHostNodeProjection {
        node_id: node.node_path.0.clone(),
        parent_id: node
            .parent
            .and_then(|parent_id| tree.node(parent_id))
            .map(|parent| parent.node_path.0.clone()),
        component: metadata.component.clone(),
        control_id: metadata.control_id.clone(),
        frame: node.layout_cache.frame,
        clip_frame: node.layout_cache.clip_frame,
        z_index: node.z_index,
        attributes: metadata.attributes.clone(),
        style_tokens: metadata.style_tokens.clone(),
        bindings: node_bindings,
    });

    for child_id in &node.children {
        collect_surface_host_nodes(tree, *child_id, bindings, host_nodes)?;
    }

    Ok(())
}
