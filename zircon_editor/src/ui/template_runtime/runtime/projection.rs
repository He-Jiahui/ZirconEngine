use std::collections::BTreeMap;

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
    template::{UiActionRef, UiBindingExpression, UiTemplateNode},
    v2::UiV2NodeHandle,
};

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
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
    merge_projection_only_host_nodes(&mut nodes, projection, &bindings)?;
    resolve_template_actions(&mut nodes);
    Ok(RetainedUiHostModel {
        document_id: projection.document_id.clone(),
        nodes,
    })
}

fn merge_projection_only_host_nodes(
    surface_nodes: &mut Vec<RetainedUiHostNodeProjection>,
    projection: &RetainedUiProjection,
    bindings: &BTreeMap<&str, &RetainedUiBindingProjection>,
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
        let arranged_node = surface.arranged_tree.get(node_id);
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
            frame: arranged_node
                .map(|arranged_node| arranged_node.frame)
                .unwrap_or(node.layout_cache.frame),
            clip_frame: arranged_node
                .map(|arranged_node| arranged_node.clip_frame)
                .or(node.layout_cache.clip_frame),
            z_index: arranged_node
                .map(|arranged_node| arranged_node.z_index)
                .unwrap_or(node.z_index),
            attributes: metadata.attributes.clone(),
            style_overrides: metadata.style_overrides.clone(),
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
    let attributes_by_control = nodes
        .iter()
        .filter_map(|node| {
            node.control_id
                .as_ref()
                .map(|control_id| (control_id.clone(), node.attributes.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    for node in nodes {
        for binding in &mut node.bindings {
            binding.template_action = binding.template_action_source.as_ref().and_then(|action| {
                resolve_template_action(action, &node.attributes, &attributes_by_control)
            });
        }
    }
}

pub(super) fn resolve_template_action(
    action: &UiActionRef,
    source_attributes: &BTreeMap<String, Value>,
    attributes_by_control: &BTreeMap<String, BTreeMap<String, Value>>,
) -> Option<UiTemplateActionInvocation> {
    let route = action.route.as_deref().or(action.action.as_deref())?.trim();
    (!route.is_empty()).then_some(())?;
    let payload = action
        .payload
        .iter()
        .map(|(key, value)| {
            Some((
                key.clone(),
                resolve_template_action_value(value, source_attributes, attributes_by_control)?,
            ))
        })
        .collect::<Option<_>>()?;
    Some(UiTemplateActionInvocation::new(route, payload))
}

fn resolve_template_action_value(
    value: &Value,
    source_attributes: &BTreeMap<String, Value>,
    attributes_by_control: &BTreeMap<String, BTreeMap<String, Value>>,
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
            resolve_template_action_expression(
                &expression,
                source_attributes,
                attributes_by_control,
            )
        })
}

fn resolve_template_action_expression(
    expression: &UiBindingExpression,
    source_attributes: &BTreeMap<String, Value>,
    attributes_by_control: &BTreeMap<String, BTreeMap<String, Value>>,
) -> Option<UiValue> {
    match expression {
        UiBindingExpression::Literal(value) => Some(value.clone()),
        UiBindingExpression::ParamRef(_) => None,
        UiBindingExpression::PropRef(property) => {
            source_attributes.get(property).map(UiValue::from_toml)
        }
        UiBindingExpression::ControlPropRef {
            control_id,
            property,
        } => attributes_by_control
            .get(control_id.as_str())
            .and_then(|attributes| attributes.get(property))
            .map(UiValue::from_toml),
        UiBindingExpression::Equals(lhs, rhs) => Some(UiValue::Bool(
            resolve_template_action_expression(lhs, source_attributes, attributes_by_control)?
                == resolve_template_action_expression(
                    rhs,
                    source_attributes,
                    attributes_by_control,
                )?,
        )),
        UiBindingExpression::NotEquals(lhs, rhs) => Some(UiValue::Bool(
            resolve_template_action_expression(lhs, source_attributes, attributes_by_control)?
                != resolve_template_action_expression(
                    rhs,
                    source_attributes,
                    attributes_by_control,
                )?,
        )),
        UiBindingExpression::And(lhs, rhs) => Some(UiValue::Bool(
            template_action_bool(&resolve_template_action_expression(
                lhs,
                source_attributes,
                attributes_by_control,
            )?)? && template_action_bool(&resolve_template_action_expression(
                rhs,
                source_attributes,
                attributes_by_control,
            )?)?,
        )),
        UiBindingExpression::Or(lhs, rhs) => Some(UiValue::Bool(
            template_action_bool(&resolve_template_action_expression(
                lhs,
                source_attributes,
                attributes_by_control,
            )?)? || template_action_bool(&resolve_template_action_expression(
                rhs,
                source_attributes,
                attributes_by_control,
            )?)?,
        )),
        UiBindingExpression::Not(value) => Some(UiValue::Bool(!template_action_bool(
            &resolve_template_action_expression(value, source_attributes, attributes_by_control)?,
        )?)),
    }
}

fn template_action_bool(value: &UiValue) -> Option<bool> {
    match value {
        UiValue::Bool(value) => Some(*value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use toml::Value;
    use zircon_runtime_interface::ui::{
        component::UiValue, dispatch::UiTemplateActionInvocation, template::UiActionRef,
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
        };
        let control_attributes = BTreeMap::from([(
            "RowList".to_string(),
            BTreeMap::from([("selected_row_identity".to_string(), Value::Integer(73))]),
        )]);

        assert_eq!(
            resolve_template_action(&action, &BTreeMap::new(), &control_attributes),
            Some(UiTemplateActionInvocation::new(
                "plugin.operation",
                BTreeMap::from([("entity".to_string(), UiValue::Int(73))]),
            ))
        );
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
