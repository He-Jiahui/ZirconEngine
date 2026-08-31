use std::collections::{BTreeMap, BTreeSet};

use crate::ui::template::{
    EditorTemplateAdapter, EditorTemplateRegistry, EditorTemplateRuntimeService,
};
use toml::Value;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::template::UiTemplateInstance;
use zircon_runtime::ui::v2::UiV2CompiledDocument;
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::UiTemplateActionInvocation,
    event_ui::UiNodeId,
    template::{UiActionRef, UiBindingExpression, UiBindingMissingValueResolution, UiTemplateNode},
    v2::UiV2NodeHandle,
};

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use crate::ui::template_runtime::{
    workbench_icon_tooltip_text, RetainedUiBindingProjection, RetainedUiHostBindingProjection,
    RetainedUiHostModel, RetainedUiHostNodeProjection, RetainedUiNodeProjection,
    RetainedUiProjection, RetainedUiProjectionSurfaceMetadataIndex,
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
        .map(|binding| (binding.binding_id.as_str(), binding))
        .collect::<BTreeMap<_, _>>();
    let mut nodes = Vec::new();
    collect_host_nodes(&projection.root, None, "root", &bindings, &mut nodes)?;
    resolve_template_actions(&mut nodes);
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
        .map(|binding| (binding.binding_id.as_str(), binding))
        .collect::<BTreeMap<_, _>>();
    let mut nodes = Vec::new();
    for root_id in &surface.tree.roots {
        collect_surface_host_nodes(surface, *root_id, &bindings, &mut nodes)?;
    }
    merge_projection_only_host_nodes(&mut nodes, projection, &bindings, surface)?;
    resolve_template_actions(&mut nodes);
    Ok(RetainedUiHostModel {
        document_id: projection.document_id.clone(),
        nodes,
    })
}

pub(super) fn build_host_nodes_with_surface(
    projection: &RetainedUiProjection,
    surface: &UiSurface,
    node_ids: &BTreeSet<UiNodeId>,
    metadata_index: &RetainedUiProjectionSurfaceMetadataIndex,
) -> Result<Vec<(UiNodeId, RetainedUiHostNodeProjection)>, EditorUiHostRuntimeError> {
    let bindings = projection
        .bindings
        .iter()
        .map(|binding| (binding.binding_id.as_str(), binding))
        .collect::<BTreeMap<_, _>>();
    let mut nodes = node_ids
        .iter()
        .map(|node_id| {
            let mut node = surface_host_node(surface, *node_id, &bindings)?;
            metadata_index.apply_to(
                node.control_id.as_deref(),
                &mut node.attributes,
                &mut node.style_tokens,
            );
            Ok((*node_id, node))
        })
        .collect::<Result<Vec<_>, EditorUiHostRuntimeError>>()?;
    for (node_id, node) in &mut nodes {
        apply_surface_focus_state_attributes(surface, *node_id, &mut node.attributes);
    }
    Ok(nodes)
}

fn merge_projection_only_host_nodes(
    surface_nodes: &mut Vec<RetainedUiHostNodeProjection>,
    projection: &RetainedUiProjection,
    bindings: &BTreeMap<&str, &RetainedUiBindingProjection>,
    surface: &UiSurface,
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

    let surface_node_ids_by_path = surface
        .tree
        .nodes
        .iter()
        .map(|(node_id, node)| (node.node_path.0.as_str(), *node_id))
        .collect::<BTreeMap<_, _>>();
    for node in surface_nodes {
        if let Some(node_id) = surface_node_ids_by_path.get(node.node_id.as_str()) {
            apply_surface_focus_state_attributes(surface, *node_id, &mut node.attributes);
        }
    }
    Ok(())
}

fn merge_projection_metadata(
    surface_node: &mut RetainedUiHostNodeProjection,
    projection_node: &RetainedUiHostNodeProjection,
) {
    for (key, value) in &projection_node.attributes {
        surface_node.attributes.insert(key.clone(), value.clone());
    }
    for (key, value) in &projection_node.style_tokens {
        surface_node.style_tokens.insert(key.clone(), value.clone());
    }
    for (key, value) in &projection_node.style_overrides {
        surface_node
            .style_overrides
            .insert(key.clone(), value.clone());
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
            template_action: binding_ref.action.clone(),
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
            template_action: binding_ref.action.clone(),
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
    bindings: &BTreeMap<&str, &RetainedUiBindingProjection>,
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
            surface_node_id: None,
            has_workbench_icon_tooltip: false,
            parent_id: frame.parent_id.clone(),
            component: frame.node.component.clone(),
            control_id: frame.node.control_id.clone(),
            frame: Default::default(),
            clip_frame: None,
            z_index: 0,
            attributes: frame.node.attributes.clone(),
            style_overrides: BTreeMap::new(),
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
    surface: &UiSurface,
    node_id: UiNodeId,
    bindings: &BTreeMap<&str, &RetainedUiBindingProjection>,
    host_nodes: &mut Vec<RetainedUiHostNodeProjection>,
) -> Result<(), EditorUiHostRuntimeError> {
    let tree = &surface.tree;
    let mut stack = vec![node_id];
    while let Some(node_id) = stack.pop() {
        let node = tree
            .node(node_id)
            .expect("surface traversal should only visit valid nodes");
        host_nodes.push(surface_host_node(surface, node_id, bindings)?);

        for child_id in node.children.iter().rev() {
            stack.push(*child_id);
        }
    }

    Ok(())
}

fn surface_host_node(
    surface: &UiSurface,
    node_id: UiNodeId,
    bindings: &BTreeMap<&str, &RetainedUiBindingProjection>,
) -> Result<RetainedUiHostNodeProjection, EditorUiHostRuntimeError> {
    let tree = &surface.tree;
    let node = tree
        .node(node_id)
        .expect("surface projection should only visit valid nodes");
    let arranged_node = surface.arranged_node(node_id);
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

    Ok(RetainedUiHostNodeProjection {
        node_id: node.node_path.0.clone(),
        surface_node_id: Some(node_id),
        has_workbench_icon_tooltip: workbench_icon_tooltip_text(metadata).is_some(),
        parent_id: node
            .parent
            .and_then(|parent_id| tree.node(parent_id))
            .map(|parent| parent.node_path.0.clone()),
        component: metadata.component.clone(),
        control_id: metadata.control_id.clone(),
        frame: arranged_node
            .map(|arranged_node| arranged_node.frame)
            .unwrap_or(node.layout_cache.frame),
        clip_frame: arranged_node
            .map(|arranged_node| arranged_node.clip_frame)
            .or(node.layout_cache.clip_frame),
        z_index: arranged_node
            .map(|arranged_node| arranged_node.z_index)
            .unwrap_or(node.z_index),
        attributes: surface_host_attributes(surface, node_id, metadata),
        style_overrides: metadata.style_overrides.clone(),
        style_tokens: metadata.style_tokens.clone(),
        bindings: node_bindings,
    })
}

fn surface_host_attributes(
    surface: &UiSurface,
    node_id: UiNodeId,
    metadata: &zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata,
) -> BTreeMap<String, Value> {
    let mut attributes = metadata.attributes.clone();
    apply_surface_focus_state_attributes(surface, node_id, &mut attributes);
    attributes
}

fn apply_surface_focus_state_attributes(
    surface: &UiSurface,
    node_id: UiNodeId,
    attributes: &mut BTreeMap<String, Value>,
) {
    let Some(component_state) = surface.component_state(node_id) else {
        return;
    };
    let runtime_focus_changed =
        surface.focus.focused == Some(node_id) || surface.focus.previous == Some(node_id);
    if !runtime_focus_changed && !component_state.flags.focus_visible {
        return;
    }
    attributes.insert(
        "focused".to_string(),
        Value::Boolean(component_state.flags.focused || component_state.flags.focus_visible),
    );
    attributes.insert(
        "focus_visible".to_string(),
        Value::Boolean(component_state.flags.focus_visible),
    );
    attributes.insert("focus_visible_known".to_string(), Value::Boolean(true));
}

fn node_bindings_from_ids(
    binding_ids: &[String],
    bindings: &BTreeMap<&str, &RetainedUiBindingProjection>,
) -> Result<Vec<RetainedUiHostBindingProjection>, EditorUiHostRuntimeError> {
    binding_ids
        .iter()
        .map(|binding_id| {
            bindings
                .get(binding_id.as_str())
                .map(|binding| RetainedUiHostBindingProjection {
                    binding_id: binding.binding_id.clone(),
                    action_id: retained_action_id_for_binding(&binding.binding),
                    event_kind: binding.binding.path().event_kind,
                    route_id: binding.route_id,
                    template_action_source: binding.template_action.clone(),
                    template_action: None,
                })
                .ok_or_else(|| EditorUiHostRuntimeError::MissingProjectionBinding {
                    binding_id: binding_id.clone(),
                })
        })
        .collect()
}

fn retained_action_id_for_binding(binding: &EditorUiBinding) -> String {
    match &binding.payload {
        EditorUiBindingPayload::MenuAction { action_id } => action_id.clone(),
        EditorUiBindingPayload::EditorCommand { command_id } => command_id.clone(),
        _ => String::new(),
    }
}

fn resolve_template_actions(nodes: &mut [RetainedUiHostNodeProjection]) {
    let mut control_indices = BTreeMap::<String, usize>::new();
    for (index, node) in nodes.iter().enumerate() {
        if let Some(control_id) = node.control_id.as_deref() {
            // Match the previous BTreeMap collect semantics: a later duplicate wins.
            control_indices.insert(control_id.to_string(), index);
        }
    }

    let resolved_actions = nodes
        .iter()
        .map(|node| {
            node.bindings
                .iter()
                .map(|binding| {
                    binding.template_action_source.as_ref().and_then(|action| {
                        resolve_template_action_with_control_lookup(
                            action,
                            &node.attributes,
                            |control_id, property| {
                                control_indices
                                    .get(control_id)
                                    .and_then(|index| nodes.get(*index))
                                    .and_then(|node| node.attributes.get(property))
                                    .map(UiValue::from_toml)
                            },
                        )
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    for (node, actions) in nodes.iter_mut().zip(resolved_actions) {
        for (binding, action) in node.bindings.iter_mut().zip(actions) {
            binding.template_action = action;
        }
    }
}

pub(super) fn resolve_template_action(
    action: &UiActionRef,
    source_attributes: &BTreeMap<String, Value>,
    attributes_by_control: &BTreeMap<String, BTreeMap<String, Value>>,
) -> Option<UiTemplateActionInvocation> {
    resolve_template_action_with_control_lookup(
        action,
        source_attributes,
        |control_id, property| {
            attributes_by_control
                .get(control_id)
                .and_then(|attributes| attributes.get(property))
                .map(UiValue::from_toml)
        },
    )
}

fn resolve_template_action_with_control_lookup(
    action: &UiActionRef,
    source_attributes: &BTreeMap<String, Value>,
    control_property: impl Fn(&str, &str) -> Option<UiValue>,
) -> Option<UiTemplateActionInvocation> {
    let route = action
        .route
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty());
    let action_id = action
        .action
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty());
    match (route, action_id) {
        (None, Some(action_id)) if action.payload.is_empty() => {
            return Some(UiTemplateActionInvocation::action(action_id));
        }
        (Some(_), None) => {}
        _ => return None,
    }
    let mut payload = BTreeMap::new();
    for (key, value) in &action.payload {
        match action
            .payload_missing_policy
            .resolve(resolve_template_action_value_with_lookup(
                value,
                source_attributes,
                &control_property,
            )) {
            UiBindingMissingValueResolution::Value(value) => {
                payload.insert(key.clone(), value);
            }
            UiBindingMissingValueResolution::Omitted => {}
            UiBindingMissingValueResolution::RequiredMissing
            | UiBindingMissingValueResolution::ExplicitError => return None,
        }
    }
    Some(UiTemplateActionInvocation::route(route?, payload))
}

fn resolve_template_action_value_with_lookup(
    value: &Value,
    source_attributes: &BTreeMap<String, Value>,
    control_property: &impl Fn(&str, &str) -> Option<UiValue>,
) -> Option<UiValue> {
    let Value::String(expression_text) = value else {
        return Some(UiValue::from_toml(value));
    };
    if !expression_text.trim_start().starts_with('=') {
        return Some(UiValue::String(expression_text.clone()));
    }
    UiBindingExpression::parse(expression_text)
        .ok()
        .and_then(|expression| {
            expression
                .evaluate_with(
                    |_| None,
                    |property| source_attributes.get(property).map(UiValue::from_toml),
                    |control_id, property| control_property(control_id, property),
                )
                .ok()
        })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use toml::Value;
    use zircon_runtime_interface::ui::{
        component::UiValue,
        dispatch::UiTemplateActionInvocation,
        template::{UiActionRef, UiBindingMissingValuePolicy},
    };

    use super::resolve_template_action;

    #[test]
    fn resolves_typed_action_payload_from_a_control_property_snapshot() {
        let action = UiActionRef {
            route: Some("plugin.operation".to_string()),
            action: None,
            payload: BTreeMap::from([(
                "entity".to_string(),
                Value::String("=control.RowList.prop.selected_row_identity".to_string()),
            )]),
            payload_missing_policy: Default::default(),
        };
        let control_attributes = BTreeMap::from([(
            "RowList".to_string(),
            BTreeMap::from([("selected_row_identity".to_string(), Value::Integer(73))]),
        )]);

        assert_eq!(
            resolve_template_action(&action, &BTreeMap::new(), &control_attributes),
            Some(UiTemplateActionInvocation::route(
                "plugin.operation",
                BTreeMap::from([("entity".to_string(), UiValue::Int(73))]),
            ))
        );
    }

    #[test]
    fn authored_editor_action_keeps_action_identity_without_a_route_alias() {
        let action = UiActionRef {
            route: None,
            action: Some("view.console.clear".to_string()),
            payload: BTreeMap::new(),
            payload_missing_policy: Default::default(),
        };

        assert_eq!(
            resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new()),
            Some(UiTemplateActionInvocation::action("view.console.clear"))
        );
    }

    #[test]
    fn authored_action_and_route_aliases_are_rejected_as_ambiguous() {
        let action = UiActionRef {
            route: Some("view.console.clear".to_string()),
            action: Some("view.console.clear".to_string()),
            payload: BTreeMap::new(),
            payload_missing_policy: Default::default(),
        };

        assert_eq!(
            resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new()),
            None
        );
    }

    #[test]
    fn authored_editor_action_with_route_payload_is_rejected() {
        let action = UiActionRef {
            route: None,
            action: Some("view.console.clear".to_string()),
            payload: BTreeMap::from([(
                "legacy_route_argument".to_string(),
                toml::Value::Boolean(true),
            )]),
            payload_missing_policy: Default::default(),
        };

        assert_eq!(
            resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new()),
            None
        );
    }

    #[test]
    fn source_action_missing_value_policy_distinguishes_omit_substitute_and_reject() {
        let mut action = UiActionRef {
            route: Some("plugin.operation".to_string()),
            action: None,
            payload: BTreeMap::from([(
                "entity".to_string(),
                Value::String("=prop.missing".to_string()),
            )]),
            payload_missing_policy: UiBindingMissingValuePolicy::Optional,
        };

        let optional = resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new())
            .expect("optional missing payload should preserve its route");
        assert!(optional.payload.is_empty());

        action.payload_missing_policy = UiBindingMissingValuePolicy::Fallback {
            value: UiValue::Int(73),
        };
        assert_eq!(
            resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new())
                .and_then(|invocation| invocation.payload.get("entity").cloned()),
            Some(UiValue::Int(73))
        );

        action.payload_missing_policy = UiBindingMissingValuePolicy::Error;
        assert!(resolve_template_action(&action, &BTreeMap::new(), &BTreeMap::new()).is_none());
    }

    #[test]
    fn console_editor_commands_are_authored_as_actions_not_routes() {
        let source = include_str!("../../../../assets/ui/editor/host/console_body.zui");

        for command_id in [
            "view.console.filter.all",
            "view.console.filter.error",
            "view.console.filter.warning",
            "view.console.filter.info",
            "view.console.source.all",
            "view.console.source.editor",
            "view.console.source.runtime",
            "view.console.source.play",
            "view.console.source.plugin",
            "view.console.source.import",
            "view.console.source.script_build",
            "view.console.clear",
        ] {
            assert!(
                source.contains(&format!("action = {{ action = \"{command_id}\" }}")),
                "{command_id} must use UiActionRef.action"
            );
            assert!(
                !source.contains(&format!("route = \"{command_id}\"")),
                "{command_id} must not retain a route alias"
            );
        }
    }

    #[test]
    fn host_projection_indexes_bindings_by_reference() {
        let source = include_str!("projection.rs");
        let builders = source
            .split("pub(super) fn build_host_model")
            .nth(1)
            .expect("host model builders")
            .split("fn merge_projection_only_host_nodes")
            .next()
            .expect("host model builder bodies");
        let cloned_rows = [".cloned", "()"].concat();

        assert!(!builders.contains(&cloned_rows));
    }
}
