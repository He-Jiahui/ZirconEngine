use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use toml::Value;
use zircon_runtime_interface::ui::component::UiValue;
use zircon_runtime_interface::ui::layout::UiPixelSnappingPolicy;
use zircon_runtime_interface::ui::template::UiBindingRef;
use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2AssetError, UiV2ChildMount, UiV2ComponentDefinition, UiV2NodeDefinition,
    UiV2Repeat, UiV2Root, UiV2StyleDeclarationBlock,
};

use super::{
    cache::UiV2PrototypeStore,
    component_reference::{
        parse_v2_component_reference, parse_v2_widget_import_reference, UiV2WidgetImportReference,
    },
};
use crate::ui::template::{
    resolve_component_binding_params, resolve_component_param_value,
    resolve_component_param_value_map, validate_typed_component_params,
};

#[derive(Clone, Debug, Default)]
struct MountPatch {
    control_id: Option<String>,
    pixel_snapping: Option<UiPixelSnappingPolicy>,
    classes: Vec<String>,
    props: BTreeMap<String, Value>,
    state: BTreeMap<String, Value>,
    layout: Option<BTreeMap<String, Value>>,
    repeat: Option<UiV2Repeat>,
    style: UiV2StyleDeclarationBlock,
    slots: BTreeMap<String, Value>,
    events: Vec<UiBindingRef>,
}

#[derive(Clone, Debug)]
struct SlotContext {
    caller_document: Arc<UiV2AssetDocument>,
    caller_params: Arc<ComponentParamScope>,
    children_by_slot: BTreeMap<String, Vec<UiV2ChildMount>>,
}

#[derive(Clone, Debug, Default)]
struct ComponentParamScope {
    values: BTreeMap<String, Value>,
    binding_values: BTreeMap<String, UiValue>,
}

#[derive(Clone, Debug)]
struct ComponentPrototype {
    key: String,
    document: Arc<UiV2AssetDocument>,
    definition: UiV2ComponentDefinition,
}

#[derive(Clone, Debug)]
struct ExpandTask {
    document: Arc<UiV2AssetDocument>,
    params: Arc<ComponentParamScope>,
    node_id: String,
    parent_output_id: Option<String>,
    mount_slot: BTreeMap<String, Value>,
    patch: Option<MountPatch>,
    slot_context: Arc<SlotContext>,
    component_stack: Vec<String>,
}

#[derive(Clone, Debug)]
struct InsertedNode {
    output_id: String,
    node: UiV2NodeDefinition,
    source_children: Vec<UiV2ChildMount>,
    parent_output_id: Option<String>,
    mount_slot: BTreeMap<String, Value>,
    source_document: Arc<UiV2AssetDocument>,
    params: Arc<ComponentParamScope>,
    slot_context: Arc<SlotContext>,
    component_stack: Vec<String>,
}

#[derive(Default)]
pub struct UiV2ComponentInstancer;

impl UiV2ComponentInstancer {
    pub fn instantiate_document(
        document: &UiV2AssetDocument,
        store: &UiV2PrototypeStore,
    ) -> Result<UiV2AssetDocument, UiV2AssetError> {
        let Some(root) = document.root_node_id() else {
            return Ok(document.clone());
        };
        validate_source_graph(document, root)?;

        let source_document = Arc::new(document.clone());
        let params = Arc::new(ComponentParamScope::default());
        let slot_context = Arc::new(SlotContext {
            caller_document: Arc::clone(&source_document),
            caller_params: Arc::clone(&params),
            children_by_slot: BTreeMap::new(),
        });
        let mut output = UiV2AssetDocument {
            root: None,
            nodes: BTreeMap::new(),
            components: BTreeMap::new(),
            ..document.clone()
        };
        let mut next_id = 0usize;
        let mut stack = vec![ExpandTask {
            document: source_document,
            params,
            node_id: root.to_string(),
            parent_output_id: None,
            mount_slot: BTreeMap::new(),
            patch: None,
            slot_context,
            component_stack: Vec::new(),
        }];

        while let Some(task) = stack.pop() {
            let Some(source_node) = task.document.nodes.get(&task.node_id).cloned() else {
                return Err(UiV2AssetError::MissingNode {
                    asset_id: task.document.asset.id.clone(),
                    node_id: task.node_id,
                });
            };

            if is_slot_placeholder(&source_node) {
                push_slot_children(&mut stack, &task, &source_node);
                continue;
            }

            if let Some(prototype) =
                resolve_component(&task.document, &source_node.component, store)?
            {
                validate_source_graph(&prototype.document, &prototype.definition.root)?;
                let mut stack_key = task.component_stack.clone();
                if stack_key.contains(&prototype.key) {
                    return Err(UiV2AssetError::InvalidDocument {
                        asset_id: task.document.asset.id.clone(),
                        detail: format!("ui v2 component cycle at {}", prototype.key),
                    });
                }
                stack_key.push(prototype.key);
                let component_params = Arc::new(resolve_component_param_scope(
                    &prototype.definition,
                    &source_node.params,
                    task.params.as_ref(),
                    &task.document.asset.id,
                )?);
                let component_slots = children_by_slot(&source_node.children);
                validate_component_slots(
                    &task.document,
                    &source_node.component,
                    &prototype.definition,
                    &component_slots,
                )?;
                let component_slot_context = Arc::new(SlotContext {
                    caller_document: Arc::clone(&task.document),
                    caller_params: Arc::clone(&task.params),
                    children_by_slot: component_slots,
                });
                stack.push(ExpandTask {
                    document: prototype.document,
                    params: component_params,
                    node_id: prototype.definition.root.clone(),
                    parent_output_id: task.parent_output_id,
                    mount_slot: task.mount_slot,
                    patch: Some(patch_for_component_mount(
                        &source_node,
                        &prototype.definition,
                        task.params.as_ref(),
                        &task.document.asset.id,
                    )?),
                    slot_context: component_slot_context,
                    component_stack: stack_key,
                });
                continue;
            }
            if !source_node.params.is_empty() {
                return Err(UiV2AssetError::InvalidDocument {
                    asset_id: task.document.asset.id.clone(),
                    detail: format!(
                        "node {} supplies params to non-prototype component {}",
                        task.node_id, source_node.component
                    ),
                });
            }

            let inserted = inserted_node(task, source_node, &mut next_id)?;
            if inserted.parent_output_id.is_none() {
                output.root = Some(UiV2Root {
                    node: inserted.output_id.clone(),
                });
            } else if let Some(parent_id) = inserted.parent_output_id.as_deref() {
                let Some(parent) = output.nodes.get_mut(parent_id) else {
                    return Err(UiV2AssetError::MissingNode {
                        asset_id: output.asset.id.clone(),
                        node_id: parent_id.to_string(),
                    });
                };
                parent.children.push(UiV2ChildMount {
                    node: inserted.output_id.clone(),
                    slot: inserted.mount_slot.clone(),
                });
            }

            for child in inserted.source_children.iter().rev() {
                stack.push(ExpandTask {
                    document: Arc::clone(&inserted.source_document),
                    params: Arc::clone(&inserted.params),
                    node_id: child.node.clone(),
                    parent_output_id: Some(inserted.output_id.clone()),
                    mount_slot: child.slot.clone(),
                    patch: None,
                    slot_context: Arc::clone(&inserted.slot_context),
                    component_stack: inserted.component_stack.clone(),
                });
            }
            output.nodes.insert(inserted.output_id, inserted.node);
        }

        Ok(output)
    }
}

fn inserted_node(
    task: ExpandTask,
    mut source_node: UiV2NodeDefinition,
    next_id: &mut usize,
) -> Result<InsertedNode, UiV2AssetError> {
    resolve_node_param_scope(
        &mut source_node,
        task.params.as_ref(),
        &task.document.asset.id,
    )?;
    let original_children = std::mem::take(&mut source_node.children);
    let preserve_source_id = task.patch.is_none() && task.component_stack.is_empty();
    if let Some(patch) = task.patch {
        apply_patch_to_node(&mut source_node, patch);
    }
    let output_id = if preserve_source_id {
        task.node_id.clone()
    } else {
        let output_id = format!("v2n{}", *next_id);
        *next_id += 1;
        output_id
    };
    source_node.children = Vec::new();
    Ok(InsertedNode {
        output_id,
        node: source_node,
        source_children: original_children,
        parent_output_id: task.parent_output_id,
        mount_slot: task.mount_slot,
        source_document: task.document,
        params: task.params,
        slot_context: task.slot_context,
        component_stack: task.component_stack,
    })
}

fn resolve_component(
    current_document: &Arc<UiV2AssetDocument>,
    component: &str,
    store: &UiV2PrototypeStore,
) -> Result<Option<ComponentPrototype>, UiV2AssetError> {
    if let Some(definition) = current_document.components.get(component).cloned() {
        return Ok(Some(ComponentPrototype {
            key: format!("{}#{component}", current_document.asset.id),
            document: Arc::clone(current_document),
            definition,
        }));
    }

    if component.contains('#') {
        let (asset_id, component_name) =
            parse_v2_component_reference(&current_document.asset.id, component)?;
        if let Some(document) = store.get(asset_id) {
            if let Some(definition) = document.components.get(component_name).cloned() {
                return Ok(Some(ComponentPrototype {
                    key: format!("{asset_id}#{component_name}"),
                    document,
                    definition,
                }));
            }
        }
    }

    for reference in &current_document.imports.widgets {
        match parse_v2_widget_import_reference(&current_document.asset.id, reference)? {
            UiV2WidgetImportReference::Component {
                asset_id,
                component: imported_component,
            } => {
                if imported_component != component {
                    continue;
                }
                let Some(document) = store.get(asset_id) else {
                    continue;
                };
                let Some(definition) = document.components.get(component).cloned() else {
                    continue;
                };
                return Ok(Some(ComponentPrototype {
                    key: format!("{asset_id}#{component}"),
                    document,
                    definition,
                }));
            }
            UiV2WidgetImportReference::WholeAsset(asset_id) => {
                let Some(document) = store.get(asset_id) else {
                    continue;
                };
                let Some(definition) = document.components.get(component).cloned() else {
                    continue;
                };
                return Ok(Some(ComponentPrototype {
                    key: format!("{asset_id}#{component}"),
                    document,
                    definition,
                }));
            }
        }
    }

    Ok(None)
}

fn patch_for_component_mount(
    node: &UiV2NodeDefinition,
    definition: &UiV2ComponentDefinition,
    params: &ComponentParamScope,
    asset_id: &str,
) -> Result<MountPatch, UiV2AssetError> {
    let mut node = node.clone();
    resolve_node_param_scope(&mut node, params, asset_id)?;
    Ok(MountPatch {
        control_id: node.control_id.clone(),
        pixel_snapping: node.pixel_snapping,
        classes: definition
            .default_classes
            .iter()
            .chain(node.classes.iter())
            .cloned()
            .collect(),
        props: node.props,
        state: node.state,
        layout: node.layout,
        repeat: node.repeat.clone(),
        style: node.style,
        slots: node.slots,
        events: node.events,
    })
}

fn resolve_component_param_scope(
    definition: &UiV2ComponentDefinition,
    provided: &BTreeMap<String, Value>,
    caller: &ComponentParamScope,
    asset_id: &str,
) -> Result<ComponentParamScope, UiV2AssetError> {
    for name in provided.keys() {
        if !definition.params.contains_key(name) {
            return Err(invalid_param_document(
                asset_id,
                format!("resolved unknown component param {name}"),
            ));
        }
    }

    let mut values = BTreeMap::new();
    for (name, schema) in &definition.params {
        let value = provided
            .get(name)
            .or(schema.default.as_ref())
            .ok_or_else(|| {
                invalid_param_document(asset_id, format!("missing required component param {name}"))
            })?;
        values.insert(
            name.clone(),
            resolve_component_param_value(value, &BTreeMap::new(), &caller.values),
        );
    }
    ensure_param_values_resolved(&values, asset_id, "component param")?;
    let binding_values = validate_typed_component_params(&definition.params, &values, asset_id)
        .map_err(|error| invalid_param_document(asset_id, error.to_string()))?;
    Ok(ComponentParamScope {
        values,
        binding_values,
    })
}

fn resolve_node_param_scope(
    node: &mut UiV2NodeDefinition,
    params: &ComponentParamScope,
    asset_id: &str,
) -> Result<(), UiV2AssetError> {
    node.props = resolve_param_value_map(&node.props, params);
    node.state = resolve_param_value_map(&node.state, params);
    node.layout = node
        .layout
        .as_ref()
        .map(|layout| resolve_param_value_map(layout, params));
    node.style = resolve_param_style(&node.style, params);
    node.slots = resolve_param_value_map(&node.slots, params);
    ensure_param_values_resolved(&node.props, asset_id, "node prop")?;
    ensure_param_values_resolved(&node.state, asset_id, "node state")?;
    if let Some(layout) = &node.layout {
        ensure_param_values_resolved(layout, asset_id, "node layout")?;
    }
    ensure_param_values_resolved(&node.style.self_values, asset_id, "node style")?;
    ensure_param_values_resolved(&node.style.slot, asset_id, "node slot style")?;
    ensure_param_values_resolved(&node.slots, asset_id, "node slot")?;
    node.events = resolve_component_binding_params(
        std::mem::take(&mut node.events),
        &params.binding_values,
        asset_id,
    )
    .map_err(|error| invalid_param_document(asset_id, error.to_string()))?;
    node.params.clear();
    Ok(())
}

fn resolve_param_style(
    style: &UiV2StyleDeclarationBlock,
    params: &ComponentParamScope,
) -> UiV2StyleDeclarationBlock {
    UiV2StyleDeclarationBlock {
        self_values: resolve_param_value_map(&style.self_values, params),
        slot: resolve_param_value_map(&style.slot, params),
    }
}

fn resolve_param_value_map(
    values: &BTreeMap<String, Value>,
    params: &ComponentParamScope,
) -> BTreeMap<String, Value> {
    resolve_component_param_value_map(values, &BTreeMap::new(), &params.values)
}

fn ensure_param_values_resolved(
    values: &BTreeMap<String, Value>,
    asset_id: &str,
    context: &str,
) -> Result<(), UiV2AssetError> {
    for (name, value) in values {
        if let Some(reference) = unresolved_param_reference(value) {
            return Err(invalid_param_document(
                asset_id,
                format!("{context} {name} references missing component param {reference}"),
            ));
        }
    }
    Ok(())
}

fn unresolved_param_reference(value: &Value) -> Option<&str> {
    match value {
        Value::String(value) => value.strip_prefix("$param."),
        Value::Array(values) => values.iter().find_map(unresolved_param_reference),
        Value::Table(values) => values.values().find_map(unresolved_param_reference),
        _ => None,
    }
}

fn invalid_param_document(asset_id: &str, detail: String) -> UiV2AssetError {
    UiV2AssetError::InvalidDocument {
        asset_id: asset_id.to_string(),
        detail,
    }
}

fn apply_patch_to_node(node: &mut UiV2NodeDefinition, patch: MountPatch) {
    if patch.control_id.is_some() {
        node.control_id = patch.control_id;
    }
    if patch.pixel_snapping.is_some() {
        node.pixel_snapping = patch.pixel_snapping;
    }
    node.classes.extend(patch.classes);
    node.props.extend(patch.props);
    node.state.extend(patch.state);
    if let Some(layout_patch) = patch.layout {
        if let Some(layout) = &mut node.layout {
            merge_value_map(layout, &layout_patch);
        } else {
            node.layout = Some(layout_patch);
        }
    }
    if patch.repeat.is_some() {
        node.repeat = patch.repeat;
    }
    node.style.self_values.extend(patch.style.self_values);
    node.style.slot.extend(patch.style.slot);
    node.slots.extend(patch.slots);
    node.events.extend(patch.events);
}

fn is_slot_placeholder(node: &UiV2NodeDefinition) -> bool {
    node.component == "Slot"
}

fn push_slot_children(stack: &mut Vec<ExpandTask>, task: &ExpandTask, node: &UiV2NodeDefinition) {
    let slot_name = slot_placeholder_name(node);
    let Some(children) = task.slot_context.children_by_slot.get(&slot_name) else {
        return;
    };
    for child in children.iter().rev() {
        let mut mount_slot = slot_placeholder_mount_slot(node);
        merge_mount_slot(&mut mount_slot, &child.slot);
        stack.push(ExpandTask {
            document: Arc::clone(&task.slot_context.caller_document),
            params: Arc::clone(&task.slot_context.caller_params),
            node_id: child.node.clone(),
            parent_output_id: task.parent_output_id.clone(),
            mount_slot,
            patch: None,
            slot_context: empty_slot_context(
                Arc::clone(&task.slot_context.caller_document),
                Arc::clone(&task.slot_context.caller_params),
            ),
            component_stack: task.component_stack.clone(),
        });
    }
}

/// A filled child replaces the placeholder node in the output graph, but the
/// placeholder still owns the child's layout contract inside the component.
/// Transfer that contract before the placeholder disappears; caller-authored
/// mount values remain the final override.
fn slot_placeholder_mount_slot(node: &UiV2NodeDefinition) -> BTreeMap<String, Value> {
    let mut slot = BTreeMap::new();
    if let Some(layout) = &node.layout {
        slot.insert(
            "layout".to_string(),
            Value::Table(layout.clone().into_iter().collect()),
        );
    }
    slot
}

fn merge_mount_slot(target: &mut BTreeMap<String, Value>, source: &BTreeMap<String, Value>) {
    merge_value_map(target, source);
}

/// Component roots own structural defaults such as their container kind while
/// instance sites commonly override only one axis. Merge nested layout maps so
/// omitted prototype structure survives and the instance remains authoritative
/// at every leaf it explicitly authors.
fn merge_value_map(target: &mut BTreeMap<String, Value>, source: &BTreeMap<String, Value>) {
    for (key, value) in source {
        match (target.get_mut(key), value) {
            (Some(Value::Table(target_table)), Value::Table(source_table)) => {
                merge_toml_table(target_table, source_table);
            }
            _ => {
                target.insert(key.clone(), value.clone());
            }
        }
    }
}

fn merge_toml_table(
    target: &mut toml::map::Map<String, Value>,
    source: &toml::map::Map<String, Value>,
) {
    for (key, value) in source {
        match (target.get_mut(key), value) {
            (Some(Value::Table(target_table)), Value::Table(source_table)) => {
                merge_toml_table(target_table, source_table);
            }
            _ => {
                target.insert(key.clone(), value.clone());
            }
        }
    }
}

fn empty_slot_context(
    caller_document: Arc<UiV2AssetDocument>,
    caller_params: Arc<ComponentParamScope>,
) -> Arc<SlotContext> {
    Arc::new(SlotContext {
        caller_document,
        caller_params,
        children_by_slot: BTreeMap::new(),
    })
}

fn slot_placeholder_name(node: &UiV2NodeDefinition) -> String {
    node.props
        .get("name")
        .or_else(|| node.props.get("slot_name"))
        .and_then(Value::as_str)
        .unwrap_or("default")
        .to_string()
}

fn children_by_slot(children: &[UiV2ChildMount]) -> BTreeMap<String, Vec<UiV2ChildMount>> {
    let mut grouped: BTreeMap<String, Vec<UiV2ChildMount>> = BTreeMap::new();
    for child in children {
        let slot_name = child
            .slot
            .get("name")
            .or_else(|| child.slot.get("slot_name"))
            .and_then(Value::as_str)
            .unwrap_or("default")
            .to_string();
        grouped.entry(slot_name).or_default().push(child.clone());
    }
    grouped
}

fn validate_source_graph(document: &UiV2AssetDocument, root: &str) -> Result<(), UiV2AssetError> {
    if !document.nodes.contains_key(root) {
        return Err(UiV2AssetError::MissingNode {
            asset_id: document.asset.id.clone(),
            node_id: root.to_string(),
        });
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut stack = vec![VisitFrame::Enter(root.to_string())];
    while let Some(frame) = stack.pop() {
        match frame {
            VisitFrame::Enter(node_id) => {
                if visited.contains(&node_id) {
                    continue;
                }
                if !visiting.insert(node_id.clone()) {
                    return Err(UiV2AssetError::InvalidDocument {
                        asset_id: document.asset.id.clone(),
                        detail: format!("ui v2 graph contains a cycle at {node_id}"),
                    });
                }
                let node =
                    document
                        .nodes
                        .get(&node_id)
                        .ok_or_else(|| UiV2AssetError::MissingNode {
                            asset_id: document.asset.id.clone(),
                            node_id: node_id.clone(),
                        })?;
                stack.push(VisitFrame::Exit(node_id));
                for child in node.children.iter().rev() {
                    stack.push(VisitFrame::Enter(child.node.clone()));
                }
            }
            VisitFrame::Exit(node_id) => {
                let _ = visiting.remove(&node_id);
                let _ = visited.insert(node_id);
            }
        }
    }

    Ok(())
}

fn validate_component_slots(
    document: &UiV2AssetDocument,
    component: &str,
    definition: &UiV2ComponentDefinition,
    children_by_slot: &BTreeMap<String, Vec<UiV2ChildMount>>,
) -> Result<(), UiV2AssetError> {
    for slot_name in children_by_slot.keys() {
        let slot_schema = if let Some(slot_schema) = definition.slots.get(slot_name) {
            slot_schema
        } else if slot_name == "default" && definition.slots.is_empty() {
            continue;
        } else {
            return Err(UiV2AssetError::UnknownSlot {
                asset_id: document.asset.id.clone(),
                component: component.to_string(),
                slot_name: slot_name.clone(),
            });
        };
        if !slot_schema.multiple && children_by_slot[slot_name].len() > 1 {
            return Err(UiV2AssetError::SlotDoesNotAcceptMultiple {
                asset_id: document.asset.id.clone(),
                component: component.to_string(),
                slot_name: slot_name.clone(),
            });
        }
        for child in &children_by_slot[slot_name] {
            let child_component = document
                .nodes
                .get(&child.node)
                .ok_or_else(|| UiV2AssetError::MissingNode {
                    asset_id: document.asset.id.clone(),
                    node_id: child.node.clone(),
                })?
                .component
                .as_str();
            let child_component = if child_component.contains('#') {
                parse_v2_component_reference(&document.asset.id, child_component)?.1
            } else {
                child_component
            };
            if !slot_schema.accepts_component(child_component) {
                return Err(UiV2AssetError::SlotDoesNotAcceptComponent {
                    asset_id: document.asset.id.clone(),
                    component: component.to_string(),
                    slot_name: slot_name.clone(),
                    child_component: child_component.to_string(),
                });
            }
        }
    }

    for (slot_name, slot_schema) in &definition.slots {
        if slot_schema.required && !children_by_slot.contains_key(slot_name) {
            return Err(UiV2AssetError::MissingRequiredSlot {
                asset_id: document.asset.id.clone(),
                component: component.to_string(),
                slot_name: slot_name.clone(),
            });
        }
    }

    Ok(())
}

enum VisitFrame {
    Enter(String),
    Exit(String),
}
