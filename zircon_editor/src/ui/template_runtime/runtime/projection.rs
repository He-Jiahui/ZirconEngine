use std::collections::BTreeMap;

use crate::ui::template::{
    EditorTemplateAdapter, EditorTemplateRegistry, EditorTemplateRuntimeService,
};
use toml::Value;
use zircon_runtime::ui::template::UiTemplateInstance;
use zircon_runtime::ui::v2::UiV2CompiledDocument;
use zircon_runtime::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, template::UiTemplateNode, tree::UiTree, v2::UiV2NodeHandle,
};

use crate::ui::template_runtime::{
    RetainedUiBindingProjection, RetainedUiHostBindingProjection, RetainedUiHostModel,
    RetainedUiHostNodeProjection, RetainedUiNodeProjection, RetainedUiProjection,
};

use super::runtime_host::EditorUiHostRuntimeError;

pub(super) fn project_document(
    template_service: &EditorTemplateRuntimeService,
    template_registry: &EditorTemplateRegistry,
    template_adapter: &EditorTemplateAdapter,
    document_id: &str,
) -> Result<RetainedUiProjection, EditorUiHostRuntimeError> {
    let instance = template_service
        .instantiate(template_registry, document_id)
        .map_err(EditorUiHostRuntimeError::from)?;
    project_instance(document_id, &instance, template_adapter)
}

pub(super) fn project_instance(
    document_id: &str,
    instance: &UiTemplateInstance,
    template_adapter: &EditorTemplateAdapter,
) -> Result<RetainedUiProjection, EditorUiHostRuntimeError> {
    let mut bindings = Vec::new();
    let root = project_node(&instance.root, template_adapter, &mut bindings)?;
    Ok(RetainedUiProjection {
        document_id: document_id.to_string(),
        root,
        bindings,
    })
}

pub(super) fn project_v2_document(
    document_id: &str,
    document: &UiV2CompiledDocument,
    template_adapter: &EditorTemplateAdapter,
) -> Result<RetainedUiProjection, EditorUiHostRuntimeError> {
    let mut bindings = Vec::new();
    let Some(root_handle) = document.arena.root else {
        return Ok(RetainedUiProjection {
            document_id: document_id.to_string(),
            root: RetainedUiNodeProjection {
                component: String::new(),
                control_id: None,
                attributes: BTreeMap::new(),
                style_tokens: BTreeMap::new(),
                binding_ids: Vec::new(),
                children: Vec::new(),
            },
            bindings,
        });
    };
    let root = project_v2_tree(document, root_handle, template_adapter, &mut bindings)?;
    Ok(RetainedUiProjection {
        document_id: document_id.to_string(),
        root,
        bindings,
    })
}

pub(super) fn build_host_model(
    projection: &RetainedUiProjection,
) -> Result<RetainedUiHostModel, EditorUiHostRuntimeError> {
    let bindings = projection
        .bindings
        .iter()
        .cloned()
        .map(|binding| (binding.binding_id.clone(), binding))
        .collect::<BTreeMap<_, _>>();
    let mut nodes = Vec::new();
    collect_host_nodes(&projection.root, None, "root", &bindings, &mut nodes)?;
    Ok(RetainedUiHostModel {
        document_id: projection.document_id.clone(),
        nodes,
    })
}

pub(super) fn build_host_model_with_surface(
    projection: &RetainedUiProjection,
    surface: &UiSurface,
) -> Result<RetainedUiHostModel, EditorUiHostRuntimeError> {
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
    merge_projection_only_host_nodes(&mut nodes, projection, &bindings)?;
    Ok(RetainedUiHostModel {
        document_id: projection.document_id.clone(),
        nodes,
    })
}

fn merge_projection_only_host_nodes(
    surface_nodes: &mut Vec<RetainedUiHostNodeProjection>,
    projection: &RetainedUiProjection,
    bindings: &BTreeMap<String, RetainedUiBindingProjection>,
) -> Result<(), EditorUiHostRuntimeError> {
    let mut projection_nodes = Vec::new();
    collect_host_nodes(
        &projection.root,
        None,
        "root",
        bindings,
        &mut projection_nodes,
    )?;

    let mut surface_by_control_id = surface_nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            node.control_id
                .as_ref()
                .map(|control_id| (control_id.clone(), index))
        })
        .collect::<BTreeMap<_, _>>();
    let projection_control_id_by_node_id = projection_nodes
        .iter()
        .filter_map(|node| {
            node.control_id
                .as_ref()
                .map(|control_id| (node.node_id.clone(), control_id.clone()))
        })
        .collect::<BTreeMap<_, _>>();

    for projection_node in &projection_nodes {
        let Some(control_id) = projection_node.control_id.as_ref() else {
            continue;
        };
        let Some(surface_index) = surface_by_control_id.get(control_id).copied() else {
            continue;
        };
        merge_projection_metadata(&mut surface_nodes[surface_index], projection_node);
    }

    for mut projection_node in projection_nodes {
        let Some(control_id) = projection_node.control_id.clone() else {
            continue;
        };
        if surface_by_control_id.contains_key(&control_id) {
            continue;
        }

        // Pane payload projection can inject synthetic host nodes after the shared surface has
        // already been built from the authored document. Keep those nodes on the surface-backed
        // host path so retained/native-slot bridges see the same projection contract.
        if let Some(parent_id) = projection_node.parent_id.as_ref() {
            if let Some(parent_control_id) = projection_control_id_by_node_id.get(parent_id) {
                if let Some(parent_index) = surface_by_control_id.get(parent_control_id) {
                    projection_node.parent_id = Some(surface_nodes[*parent_index].node_id.clone());
                }
            }
        }
        let surface_index = surface_nodes.len();
        surface_nodes.push(projection_node);
        surface_by_control_id.insert(control_id, surface_index);
    }

    Ok(())
}

fn merge_projection_metadata(
    surface_node: &mut RetainedUiHostNodeProjection,
    projection_node: &RetainedUiHostNodeProjection,
) {
    for (key, value) in &projection_node.attributes {
        surface_node
            .attributes
            .entry(key.clone())
            .or_insert_with(|| value.clone());
    }
    for (key, value) in &projection_node.style_tokens {
        surface_node
            .style_tokens
            .entry(key.clone())
            .or_insert_with(|| value.clone());
    }
}

fn project_node(
    node: &UiTemplateNode,
    adapter: &EditorTemplateAdapter,
    bindings: &mut Vec<RetainedUiBindingProjection>,
) -> Result<RetainedUiNodeProjection, EditorUiHostRuntimeError> {
    let mut binding_ids = Vec::new();
    for binding_ref in &node.bindings {
        let binding = adapter
            .resolve_binding(binding_ref)
            .map_err(EditorUiHostRuntimeError::from)?;
        binding_ids.push(binding_ref.id.clone());
        bindings.push(RetainedUiBindingProjection {
            binding_id: binding_ref.id.clone(),
            binding,
            route_id: None,
        });
    }

    Ok(RetainedUiNodeProjection {
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

fn project_v2_tree(
    document: &UiV2CompiledDocument,
    root: UiV2NodeHandle,
    adapter: &EditorTemplateAdapter,
    bindings: &mut Vec<RetainedUiBindingProjection>,
) -> Result<RetainedUiNodeProjection, EditorUiHostRuntimeError> {
    let mut stack = vec![V2ProjectionFrame::Enter(root)];
    let mut binding_ids_by_handle = BTreeMap::<UiV2NodeHandle, Vec<String>>::new();
    let mut projected_by_handle = BTreeMap::<UiV2NodeHandle, RetainedUiNodeProjection>::new();

    while let Some(frame) = stack.pop() {
        match frame {
            V2ProjectionFrame::Enter(handle) => {
                let node = document.arena.node(handle).ok_or_else(|| {
                    EditorUiHostRuntimeError::MissingSurfaceMetadata {
                        node_path: format!("v2/{}", handle.index()),
                    }
                })?;
                let binding_ids = project_v2_binding_ids(node, adapter, bindings)?;
                binding_ids_by_handle.insert(handle, binding_ids);
                stack.push(V2ProjectionFrame::Exit(handle));
                for child in node.children.iter().rev() {
                    stack.push(V2ProjectionFrame::Enter(child.child));
                }
            }
            V2ProjectionFrame::Exit(handle) => {
                let node = document.arena.node(handle).ok_or_else(|| {
                    EditorUiHostRuntimeError::MissingSurfaceMetadata {
                        node_path: format!("v2/{}", handle.index()),
                    }
                })?;
                let binding_ids = binding_ids_by_handle.remove(&handle).unwrap_or_default();
                let children = node
                    .children
                    .iter()
                    .map(|child| {
                        projected_by_handle.remove(&child.child).ok_or_else(|| {
                            EditorUiHostRuntimeError::MissingSurfaceMetadata {
                                node_path: format!("v2/{}", child.child.index()),
                            }
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                projected_by_handle.insert(
                    handle,
                    RetainedUiNodeProjection {
                        component: node.component.clone(),
                        control_id: node.control_id.clone(),
                        attributes: v2_node_attributes(node),
                        style_tokens: BTreeMap::new(),
                        binding_ids,
                        children,
                    },
                );
            }
        }
    }

    projected_by_handle.remove(&root).ok_or_else(|| {
        EditorUiHostRuntimeError::MissingSurfaceMetadata {
            node_path: format!("v2/{}", root.index()),
        }
    })
}

fn project_v2_binding_ids(
    node: &zircon_runtime_interface::ui::v2::UiV2ArenaNode,
    adapter: &EditorTemplateAdapter,
    bindings: &mut Vec<RetainedUiBindingProjection>,
) -> Result<Vec<String>, EditorUiHostRuntimeError> {
    let mut binding_ids = Vec::new();
    for binding_ref in &node.events {
        let binding = adapter
            .resolve_binding(binding_ref)
            .map_err(EditorUiHostRuntimeError::from)?;
        binding_ids.push(binding_ref.id.clone());
        bindings.push(RetainedUiBindingProjection {
            binding_id: binding_ref.id.clone(),
            binding,
            route_id: None,
        });
    }
    Ok(binding_ids)
}

fn v2_node_attributes(
    node: &zircon_runtime_interface::ui::v2::UiV2ArenaNode,
) -> BTreeMap<String, Value> {
    let mut attributes = node.props.clone();
    attributes.extend(node.state.clone());
    if let Some(layout) = &node.layout {
        attributes.insert(
            "layout".to_string(),
            Value::Table(layout.clone().into_iter().collect()),
        );
    }
    attributes
}

enum V2ProjectionFrame {
    Enter(UiV2NodeHandle),
    Exit(UiV2NodeHandle),
}

fn collect_host_nodes(
    node: &RetainedUiNodeProjection,
    parent_id: Option<&str>,
    node_id: &str,
    bindings: &BTreeMap<String, RetainedUiBindingProjection>,
    host_nodes: &mut Vec<RetainedUiHostNodeProjection>,
) -> Result<(), EditorUiHostRuntimeError> {
    let mut stack = vec![HostProjectionFrame {
        node,
        parent_id: parent_id.map(str::to_string),
        node_id: node_id.to_string(),
    }];
    while let Some(frame) = stack.pop() {
        let node_bindings = node_bindings_from_ids(&frame.node.binding_ids, bindings)?;
        host_nodes.push(RetainedUiHostNodeProjection {
            node_id: frame.node_id.clone(),
            parent_id: frame.parent_id.clone(),
            component: frame.node.component.clone(),
            control_id: frame.node.control_id.clone(),
            frame: Default::default(),
            clip_frame: None,
            z_index: 0,
            attributes: frame.node.attributes.clone(),
            style_tokens: frame.node.style_tokens.clone(),
            bindings: node_bindings,
        });

        for (index, child) in frame.node.children.iter().enumerate().rev() {
            stack.push(HostProjectionFrame {
                node: child,
                parent_id: Some(frame.node_id.clone()),
                node_id: format!("{}.{index}", frame.node_id),
            });
        }
    }
    Ok(())
}

struct HostProjectionFrame<'a> {
    node: &'a RetainedUiNodeProjection,
    parent_id: Option<String>,
    node_id: String,
}

fn collect_surface_host_nodes(
    tree: &UiTree,
    node_id: UiNodeId,
    bindings: &BTreeMap<String, RetainedUiBindingProjection>,
    host_nodes: &mut Vec<RetainedUiHostNodeProjection>,
) -> Result<(), EditorUiHostRuntimeError> {
    let mut stack = vec![node_id];
    while let Some(node_id) = stack.pop() {
        let node = tree
            .node(node_id)
            .expect("surface traversal should only visit valid nodes");
        let metadata = node.template_metadata.as_ref().ok_or_else(|| {
            EditorUiHostRuntimeError::MissingSurfaceMetadata {
                node_path: node.node_path.0.clone(),
            }
        })?;
        let binding_ids = metadata
            .bindings
            .iter()
            .map(|binding_ref| binding_ref.id.clone())
            .collect::<Vec<_>>();
        let node_bindings = node_bindings_from_ids(&binding_ids, bindings)?;

        host_nodes.push(RetainedUiHostNodeProjection {
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

        for child_id in node.children.iter().rev() {
            stack.push(*child_id);
        }
    }

    Ok(())
}

fn node_bindings_from_ids(
    binding_ids: &[String],
    bindings: &BTreeMap<String, RetainedUiBindingProjection>,
) -> Result<Vec<RetainedUiHostBindingProjection>, EditorUiHostRuntimeError> {
    binding_ids
        .iter()
        .map(|binding_id| {
            bindings
                .get(binding_id)
                .map(|binding| RetainedUiHostBindingProjection {
                    binding_id: binding.binding_id.clone(),
                    event_kind: binding.binding.path().event_kind,
                    route_id: binding.route_id,
                })
                .ok_or_else(|| EditorUiHostRuntimeError::MissingProjectionBinding {
                    binding_id: binding_id.clone(),
                })
        })
        .collect()
}
