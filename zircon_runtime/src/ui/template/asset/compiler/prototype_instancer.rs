use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use toml::{map::Map, Value};

use crate::ui::template::{UiPrototypeStore, UiTemplateInstance};
use zircon_runtime_interface::ui::component::UiValue;
use zircon_runtime_interface::ui::template::{
    parse_component_reference, UiAssetError, UiAssetKind, UiComponentPrototype,
    UiNodeDefinitionKind, UiNodePrototype, UiPrototypeChildMount, UiPrototypeNodeHandle,
    UiRawAssetPrototype, UiTemplateNode,
};

use super::binding_param_resolver::{resolve_binding_params, typed_component_params};
use super::binding_program::compile_binding_program;
use super::control_scope::{
    resolve_binding_control_scope, resolve_optional_control_id, retarget_expanded_root_control_id,
    validate_unique_control_ids, UiComponentControlScope,
};
use super::style_apply::{
    append_mui_style_classes, apply_mui_child_slot_props, apply_mui_root_slot_props_to_node,
    apply_mui_sx_to_node, apply_styles_to_tree, build_style_plan,
};
use super::ui_document_compiler::{ResolvedStyleSheet, UiCompiledDocument, UiDocumentCompiler};
use super::value_normalizer::{
    append_classes, build_prototype_attribute_map, compose_tokens, merge_value_maps,
    merge_value_maps_resolved, normalize_layout, resolve_value, resolve_value_map,
};

impl UiDocumentCompiler {
    pub fn compile_prototype_asset(
        &self,
        asset_id: &str,
        store: &UiPrototypeStore,
    ) -> Result<UiCompiledDocument, UiAssetError> {
        let prototype = store
            .get(asset_id)
            .ok_or_else(|| UiAssetError::UnknownImport {
                reference: asset_id.to_string(),
            })?;
        self.compile_prototype(Arc::clone(&prototype), store)
    }

    pub fn compile_prototype(
        &self,
        prototype: Arc<UiRawAssetPrototype>,
        store: &UiPrototypeStore,
    ) -> Result<UiCompiledDocument, UiAssetError> {
        let root = prototype
            .document
            .root
            .ok_or_else(|| UiAssetError::InvalidDocument {
                asset_id: prototype.asset.id.clone(),
                detail: "layout/widget prototype assets require [root]".to_string(),
            })?;
        let mut artifacts = PrototypeCompilationArtifacts::default();
        let tokens = compose_tokens(&BTreeMap::new(), &prototype.tokens);
        let mut roots = PrototypeInstancer::new(store, &mut artifacts).expand_node(
            Arc::clone(&prototype),
            root,
            tokens,
            BTreeMap::new(),
            BTreeMap::new(),
            None,
            None,
        )?;
        let mut root = roots
            .drain(..)
            .next()
            .ok_or_else(|| UiAssetError::InvalidDocument {
                asset_id: prototype.asset.id.clone(),
                detail: "prototype expansion produced no root nodes".to_string(),
            })?;

        validate_unique_control_ids(&root, &prototype.asset.id)?;

        apply_prototype_styles(&prototype, self, store, &mut root, &artifacts)?;

        let binding_program = compile_binding_program(&root, &prototype.asset.id)?;
        Ok(UiCompiledDocument {
            asset: prototype.asset.clone(),
            instance: UiTemplateInstance::with_binding_program(root, binding_program),
            resource_dependencies: Vec::new(),
            resource_diagnostics: Vec::new(),
        })
    }
}

struct PrototypeInstancer<'a> {
    store: &'a UiPrototypeStore,
    artifacts: &'a mut PrototypeCompilationArtifacts,
}

impl<'a> PrototypeInstancer<'a> {
    fn new(store: &'a UiPrototypeStore, artifacts: &'a mut PrototypeCompilationArtifacts) -> Self {
        Self { store, artifacts }
    }

    fn expand_node(
        &mut self,
        asset: Arc<UiRawAssetPrototype>,
        node: UiPrototypeNodeHandle,
        tokens: BTreeMap<String, Value>,
        params: BTreeMap<String, Value>,
        binding_params: BTreeMap<String, UiValue>,
        slot_fills: Option<Arc<BTreeMap<String, Vec<UiTemplateNode>>>>,
        control_scope: Option<UiComponentControlScope>,
    ) -> Result<Vec<UiTemplateNode>, UiAssetError> {
        let mut frames = vec![PrototypeFrame::Expand(PrototypeExpandTask {
            asset,
            node,
            tokens,
            params,
            binding_params,
            slot_fills,
            control_scope,
        })];
        let mut results = Vec::<Vec<UiTemplateNode>>::new();

        while let Some(frame) = frames.pop() {
            match frame {
                PrototypeFrame::Expand(task) => {
                    self.push_expand_frame(task, &mut frames, &mut results)?
                }
                PrototypeFrame::FinalizeNative(frame) => {
                    self.finalize_native(frame, &mut results)?
                }
                PrototypeFrame::FinalizeComponentFills(frame) => {
                    self.finalize_component_fills(frame, &mut frames, &mut results)?;
                }
                PrototypeFrame::FinalizeComponentRoot(frame) => {
                    self.finalize_component_root(frame, &mut results)?;
                }
            }
        }

        results.pop().ok_or_else(|| UiAssetError::InvalidDocument {
            asset_id: "prototype".to_string(),
            detail: "prototype expansion produced no result".to_string(),
        })
    }

    fn push_expand_frame(
        &mut self,
        task: PrototypeExpandTask,
        frames: &mut Vec<PrototypeFrame>,
        results: &mut Vec<Vec<UiTemplateNode>>,
    ) -> Result<(), UiAssetError> {
        let node = task
            .asset
            .node(task.node)
            .ok_or_else(|| UiAssetError::MissingNode {
                asset_id: task.asset.asset.id.clone(),
                node_id: format!("#{}", task.node.0),
            })?
            .clone();

        match node.kind {
            UiNodeDefinitionKind::Native => self.push_native_frame(task, &node, frames),
            UiNodeDefinitionKind::Component => self.push_local_component_frame(task, &node, frames),
            UiNodeDefinitionKind::Reference => {
                self.push_reference_component_frame(task, &node, frames)
            }
            UiNodeDefinitionKind::Slot => {
                let slot_name =
                    node.slot_name
                        .as_deref()
                        .ok_or_else(|| UiAssetError::InvalidDocument {
                            asset_id: task.asset.asset.id.clone(),
                            detail: format!("slot node {} missing slot_name", node.node_id),
                        })?;
                results.push(
                    task.slot_fills
                        .as_ref()
                        .and_then(|fills| fills.get(slot_name))
                        .cloned()
                        .unwrap_or_default(),
                );
                Ok(())
            }
        }
    }

    fn push_native_frame(
        &self,
        task: PrototypeExpandTask,
        node: &UiNodePrototype,
        frames: &mut Vec<PrototypeFrame>,
    ) -> Result<(), UiAssetError> {
        let component = node
            .widget_type
            .as_ref()
            .ok_or_else(|| UiAssetError::InvalidDocument {
                asset_id: task.asset.asset.id.clone(),
                detail: format!("native node {} missing type", node.node_id),
            })?
            .clone();
        let child_mounts = node.children.clone();

        frames.push(PrototypeFrame::FinalizeNative(PrototypeNativeFrame {
            task: task.clone_without_slot_fills(),
            component,
            node: node.clone(),
            child_mounts: child_mounts.clone(),
        }));
        for child in child_mounts.iter().rev() {
            frames.push(PrototypeFrame::Expand(PrototypeExpandTask {
                asset: Arc::clone(&task.asset),
                node: child.child,
                tokens: task.tokens.clone(),
                params: task.params.clone(),
                binding_params: task.binding_params.clone(),
                slot_fills: task.slot_fills.clone(),
                control_scope: task.control_scope.clone(),
            }));
        }
        Ok(())
    }

    fn push_local_component_frame(
        &self,
        task: PrototypeExpandTask,
        node: &UiNodePrototype,
        frames: &mut Vec<PrototypeFrame>,
    ) -> Result<(), UiAssetError> {
        let component_name =
            node.component
                .as_deref()
                .ok_or_else(|| UiAssetError::InvalidDocument {
                    asset_id: task.asset.asset.id.clone(),
                    detail: format!("component node {} missing component name", node.node_id),
                })?;
        let component = task
            .asset
            .components
            .get(component_name)
            .ok_or_else(|| UiAssetError::UnknownComponent {
                asset_id: task.asset.asset.id.clone(),
                component: component_name.to_string(),
            })?
            .clone();
        let component_asset = Arc::clone(&task.asset);
        self.push_component_frame(
            task,
            node,
            component_asset,
            component_name,
            component,
            None,
            frames,
        )
    }

    fn push_reference_component_frame(
        &self,
        task: PrototypeExpandTask,
        node: &UiNodePrototype,
        frames: &mut Vec<PrototypeFrame>,
    ) -> Result<(), UiAssetError> {
        let reference =
            node.component_ref
                .as_deref()
                .ok_or_else(|| UiAssetError::InvalidDocument {
                    asset_id: task.asset.asset.id.clone(),
                    detail: format!("reference node {} missing component_ref", node.node_id),
                })?;
        let _ = parse_component_reference(reference)?;
        let (asset, component_name) = self.store.component_prototype(reference)?;
        let component = asset
            .components
            .get(&component_name)
            .ok_or_else(|| UiAssetError::UnknownComponent {
                asset_id: asset.asset.id.clone(),
                component: component_name.clone(),
            })?
            .clone();
        let tokens = compose_tokens(&task.tokens, &asset.tokens);
        self.push_component_frame(
            task,
            node,
            asset,
            &component_name,
            component,
            Some(tokens),
            frames,
        )
    }

    fn push_component_frame(
        &self,
        task: PrototypeExpandTask,
        node: &UiNodePrototype,
        component_asset: Arc<UiRawAssetPrototype>,
        component_name: &str,
        component: UiComponentPrototype,
        call_tokens: Option<BTreeMap<String, Value>>,
        frames: &mut Vec<PrototypeFrame>,
    ) -> Result<(), UiAssetError> {
        validate_prototype_slot_mounts(&task.asset, component_name, &component, &node.children)?;

        let child_mounts = node.children.clone();
        let call_tokens = call_tokens.unwrap_or_else(|| task.tokens.clone());
        let caller_tokens = task.tokens.clone();
        let params = task.params.clone();
        let binding_params = task.binding_params.clone();
        let component_root =
            component_asset
                .node(component.root)
                .ok_or_else(|| UiAssetError::MissingNode {
                    asset_id: component_asset.asset.id.clone(),
                    node_id: format!("#{}", component.root.0),
                })?;
        let component_scope = UiComponentControlScope::child(
            task.control_scope.as_ref(),
            &node.node_id,
            component_root.control_id.as_deref(),
            node.control_id.as_deref(),
        );
        frames.push(PrototypeFrame::FinalizeComponentFills(
            PrototypeComponentFillsFrame {
                caller_asset_id: task.asset.asset.id.clone(),
                component_asset,
                component_name: component_name.to_string(),
                component,
                instance_node: node.clone(),
                call_tokens,
                params: params.clone(),
                binding_params: binding_params.clone(),
                child_mounts: child_mounts.clone(),
                caller_scope: task.control_scope.clone(),
                component_scope,
            },
        ));
        for child in child_mounts.iter().rev() {
            frames.push(PrototypeFrame::Expand(PrototypeExpandTask {
                asset: Arc::clone(&task.asset),
                node: child.child,
                tokens: caller_tokens.clone(),
                params: params.clone(),
                binding_params: binding_params.clone(),
                slot_fills: None,
                control_scope: task.control_scope.clone(),
            }));
        }
        Ok(())
    }

    fn finalize_native(
        &self,
        frame: PrototypeNativeFrame,
        results: &mut Vec<Vec<UiTemplateNode>>,
    ) -> Result<(), UiAssetError> {
        let mut children = pop_child_results(results, frame.child_mounts.len())?;
        let mut mounted = Vec::new();
        for (nodes, mount) in children.drain(..).zip(&frame.child_mounts) {
            mounted.extend(apply_prototype_child_mount(
                nodes,
                mount,
                &frame.task.tokens,
                &frame.task.params,
            ));
        }

        let attributes =
            build_prototype_attribute_map(&frame.node, &frame.task.tokens, &frame.task.params);
        let bindings = resolve_binding_control_scope(
            resolve_binding_params(
                frame.node.bindings,
                &frame.task.binding_params,
                &frame.task.asset.asset.id,
            )?,
            frame.task.control_scope.as_ref(),
            &frame.task.asset.asset.id,
        )?;
        let binding_source_asset_ids = vec![frame.task.asset.asset.id.clone(); bindings.len()];

        results.push(vec![UiTemplateNode {
            source_asset_id: Some(frame.task.asset.asset.id.clone()),
            component: Some(frame.component),
            template: None,
            slot: None,
            control_id: resolve_optional_control_id(
                frame.task.control_scope.as_ref(),
                frame.node.control_id.as_deref(),
            ),
            classes: frame.node.classes,
            bindings,
            binding_source_asset_ids,
            children: mounted,
            slots: BTreeMap::new(),
            attributes,
            slot_attributes: BTreeMap::new(),
            style_overrides: resolve_value_map(
                &frame.node.style_overrides.self_values,
                &frame.task.tokens,
                &frame.task.params,
            ),
            style_tokens: BTreeMap::new(),
            focus: frame.node.focus.unwrap_or_default(),
            navigation: frame.node.navigation.unwrap_or_default(),
            picking: frame.node.picking.unwrap_or_default(),
            a11y: frame.node.a11y.unwrap_or_default(),
            widget: frame.node.widget.unwrap_or_default(),
        }]);
        Ok(())
    }

    fn finalize_component_fills(
        &mut self,
        frame: PrototypeComponentFillsFrame,
        frames: &mut Vec<PrototypeFrame>,
        results: &mut Vec<Vec<UiTemplateNode>>,
    ) -> Result<(), UiAssetError> {
        if frame.component_asset.asset.kind == UiAssetKind::Widget {
            self.artifacts
                .record_widget_styles(&frame.component_asset, &frame.call_tokens);
        }

        let mut child_results = pop_child_results(results, frame.child_mounts.len())?;
        let mut fills = BTreeMap::<String, Vec<UiTemplateNode>>::new();
        for (nodes, mount) in child_results.drain(..).zip(&frame.child_mounts) {
            let mount_name = mount.mount.clone().unwrap_or_default();
            fills
                .entry(mount_name)
                .or_default()
                .extend(apply_prototype_child_mount(
                    nodes,
                    mount,
                    &frame.call_tokens,
                    &frame.params,
                ));
        }

        let component_params = resolve_prototype_component_params(
            &frame.component,
            &frame.instance_node.params,
            &frame.call_tokens,
            &frame.params,
        );
        let component_binding_params = typed_component_params(
            &frame.component.params,
            &component_params,
            &frame.component_asset.asset.id,
        )?;
        let component_tokens = compose_tokens(&frame.call_tokens, &frame.component_asset.tokens);
        frames.push(PrototypeFrame::FinalizeComponentRoot(
            PrototypeComponentRootFrame {
                asset_id: frame.component_asset.asset.id.clone(),
                caller_asset_id: frame.caller_asset_id,
                component_name: frame.component_name,
                instance_node: frame.instance_node,
                tokens: frame.call_tokens,
                params: frame.params,
                binding_params: frame.binding_params,
                caller_scope: frame.caller_scope,
            },
        ));
        frames.push(PrototypeFrame::Expand(PrototypeExpandTask {
            asset: frame.component_asset,
            node: frame.component.root,
            tokens: component_tokens,
            params: component_params,
            binding_params: component_binding_params,
            slot_fills: Some(Arc::new(fills)),
            control_scope: Some(frame.component_scope),
        }));
        Ok(())
    }

    fn finalize_component_root(
        &self,
        frame: PrototypeComponentRootFrame,
        results: &mut Vec<Vec<UiTemplateNode>>,
    ) -> Result<(), UiAssetError> {
        let mut roots = results.pop().ok_or_else(|| UiAssetError::InvalidDocument {
            asset_id: frame.asset_id.clone(),
            detail: format!("component {} produced no root node", frame.component_name),
        })?;
        if roots.len() != 1 {
            return Err(UiAssetError::InvalidDocument {
                asset_id: frame.asset_id,
                detail: format!(
                    "component {} must expand to exactly one root node",
                    frame.component_name
                ),
            });
        }
        let mut root = roots.remove(0);
        decorate_prototype_component_root(
            &mut root,
            &frame.instance_node,
            &frame.tokens,
            &frame.params,
            &frame.binding_params,
            &frame.caller_asset_id,
            frame.caller_scope.as_ref(),
        )?;
        results.push(vec![root]);
        Ok(())
    }
}

#[derive(Clone)]
struct PrototypeExpandTask {
    asset: Arc<UiRawAssetPrototype>,
    node: UiPrototypeNodeHandle,
    tokens: BTreeMap<String, Value>,
    params: BTreeMap<String, Value>,
    binding_params: BTreeMap<String, UiValue>,
    slot_fills: Option<Arc<BTreeMap<String, Vec<UiTemplateNode>>>>,
    control_scope: Option<UiComponentControlScope>,
}

impl PrototypeExpandTask {
    fn clone_without_slot_fills(&self) -> Self {
        Self {
            asset: Arc::clone(&self.asset),
            node: self.node,
            tokens: self.tokens.clone(),
            params: self.params.clone(),
            binding_params: self.binding_params.clone(),
            slot_fills: None,
            control_scope: self.control_scope.clone(),
        }
    }
}

enum PrototypeFrame {
    Expand(PrototypeExpandTask),
    FinalizeNative(PrototypeNativeFrame),
    FinalizeComponentFills(PrototypeComponentFillsFrame),
    FinalizeComponentRoot(PrototypeComponentRootFrame),
}

struct PrototypeNativeFrame {
    task: PrototypeExpandTask,
    component: String,
    node: UiNodePrototype,
    child_mounts: Vec<UiPrototypeChildMount>,
}

struct PrototypeComponentFillsFrame {
    caller_asset_id: String,
    component_asset: Arc<UiRawAssetPrototype>,
    component_name: String,
    component: UiComponentPrototype,
    instance_node: UiNodePrototype,
    call_tokens: BTreeMap<String, Value>,
    params: BTreeMap<String, Value>,
    binding_params: BTreeMap<String, UiValue>,
    child_mounts: Vec<UiPrototypeChildMount>,
    caller_scope: Option<UiComponentControlScope>,
    component_scope: UiComponentControlScope,
}

struct PrototypeComponentRootFrame {
    asset_id: String,
    caller_asset_id: String,
    component_name: String,
    instance_node: UiNodePrototype,
    tokens: BTreeMap<String, Value>,
    params: BTreeMap<String, Value>,
    binding_params: BTreeMap<String, UiValue>,
    caller_scope: Option<UiComponentControlScope>,
}

#[derive(Default)]
struct PrototypeCompilationArtifacts {
    widget_styles: Vec<ResolvedStyleSheet>,
    seen_widget_assets: BTreeSet<String>,
}

impl PrototypeCompilationArtifacts {
    fn record_widget_styles(
        &mut self,
        prototype: &UiRawAssetPrototype,
        inherited: &BTreeMap<String, Value>,
    ) {
        if !self.seen_widget_assets.insert(prototype.asset.id.clone()) {
            return;
        }
        let tokens = compose_tokens(inherited, &prototype.tokens);
        for style in &prototype.styles {
            self.widget_styles.push(ResolvedStyleSheet {
                stylesheet: style.stylesheet.clone(),
                tokens: tokens.clone(),
            });
        }
    }
}

fn pop_child_results(
    results: &mut Vec<Vec<UiTemplateNode>>,
    count: usize,
) -> Result<Vec<Vec<UiTemplateNode>>, UiAssetError> {
    if results.len() < count {
        return Err(UiAssetError::InvalidDocument {
            asset_id: "prototype".to_string(),
            detail: "prototype expansion result stack underflow".to_string(),
        });
    }
    let mut children = Vec::with_capacity(count);
    for _ in 0..count {
        children.push(results.pop().expect("result length checked"));
    }
    children.reverse();
    Ok(children)
}

fn apply_prototype_child_mount(
    nodes: Vec<UiTemplateNode>,
    child: &UiPrototypeChildMount,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) -> Vec<UiTemplateNode> {
    let mut slot = resolve_value_map(&child.slot, tokens, params);
    if let Some(mount) = child.mount.as_deref().filter(|mount| !mount.is_empty()) {
        slot.entry("mui_slot".to_string())
            .or_insert_with(|| Value::String(mount.to_string()));
    }
    normalize_layout(&mut slot);
    nodes
        .into_iter()
        .map(|mut node| {
            merge_value_maps(&mut node.slot_attributes, &slot);
            node
        })
        .collect()
}

fn resolve_prototype_component_params(
    component: &UiComponentPrototype,
    provided: &BTreeMap<String, Value>,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    let mut resolved = BTreeMap::new();
    for (name, schema) in &component.params {
        if let Some(default) = &schema.default {
            let _ = resolved.insert(name.clone(), resolve_value(default, tokens, params));
        }
    }
    for (name, value) in provided {
        let _ = resolved.insert(name.clone(), resolve_value(value, tokens, params));
    }
    resolved
}

fn decorate_prototype_component_root(
    root: &mut UiTemplateNode,
    instance_node: &UiNodePrototype,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
    binding_params: &BTreeMap<String, UiValue>,
    asset_id: &str,
    control_scope: Option<&UiComponentControlScope>,
) -> Result<(), UiAssetError> {
    if let Some(control_id) =
        resolve_optional_control_id(control_scope, instance_node.control_id.as_deref())
    {
        retarget_expanded_root_control_id(root, control_id, asset_id)?;
    }
    append_classes(&mut root.classes, &instance_node.classes);
    let instance_bindings = resolve_binding_control_scope(
        resolve_binding_params(instance_node.bindings.clone(), binding_params, asset_id)?,
        control_scope,
        asset_id,
    )?;
    root.binding_source_asset_ids.extend(std::iter::repeat_n(
        asset_id.to_string(),
        instance_bindings.len(),
    ));
    root.bindings.extend(instance_bindings);
    merge_prototype_instance_props_override(&mut root.attributes, instance_node, tokens, params);
    merge_prototype_instance_layout_override(
        &mut root.style_overrides,
        instance_node,
        tokens,
        params,
    );
    let inline = resolve_value_map(&instance_node.style_overrides.self_values, tokens, params);
    merge_value_maps(&mut root.style_overrides, &inline);
    merge_value_maps_resolved(
        &mut root.slot_attributes,
        &instance_node.style_overrides.slot,
        tokens,
        params,
    );
    if let Some(focus) = &instance_node.focus {
        root.focus = focus.clone();
    }
    if let Some(navigation) = &instance_node.navigation {
        root.navigation = navigation.clone();
    }
    if let Some(picking) = instance_node.picking {
        root.picking = picking;
    }
    if let Some(a11y) = &instance_node.a11y {
        root.a11y = a11y.clone();
    }
    if let Some(widget) = &instance_node.widget {
        root.widget = widget.clone();
    }
    Ok(())
}

fn merge_prototype_instance_props_override(
    target: &mut BTreeMap<String, Value>,
    instance_node: &UiNodePrototype,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) {
    let props = resolve_value_map(&instance_node.props, tokens, params);
    merge_value_maps(target, &props);
}

fn merge_prototype_instance_layout_override(
    target: &mut BTreeMap<String, Value>,
    instance_node: &UiNodePrototype,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) {
    let Some(layout) = &instance_node.layout else {
        return;
    };

    let mut inline = BTreeMap::new();
    let layout = resolve_value_map(layout, tokens, params)
        .into_iter()
        .collect::<Map<_, _>>();
    inline.insert("layout".to_string(), Value::Table(layout));
    normalize_layout(&mut inline);
    merge_value_maps(target, &inline);
}

fn validate_prototype_slot_mounts(
    asset: &UiRawAssetPrototype,
    component_name: &str,
    component: &UiComponentPrototype,
    children: &[UiPrototypeChildMount],
) -> Result<(), UiAssetError> {
    let mut counts = BTreeMap::<String, usize>::new();
    for child in children {
        let slot_name = child.mount.clone().unwrap_or_default();
        let slot = component
            .slots
            .get(&slot_name)
            .ok_or_else(|| UiAssetError::UnknownSlot {
                component: component_name.to_string(),
                slot_name: slot_name.clone(),
            })?;
        let count = counts.entry(slot_name.clone()).or_insert(0);
        *count += 1;
        if !slot.multiple && *count > 1 {
            return Err(UiAssetError::SlotDoesNotAcceptMultiple {
                component: component_name.to_string(),
                slot_name,
            });
        }
        let child_node = asset
            .node(child.child)
            .ok_or_else(|| UiAssetError::MissingNode {
                asset_id: asset.asset.id.clone(),
                node_id: format!("#{}", child.child.0),
            })?;
        let child_component =
            prototype_child_component_name(child_node)?.unwrap_or("<unresolved>".to_string());
        if !slot.accepts_component(&child_component) {
            return Err(UiAssetError::SlotDoesNotAcceptComponent {
                component: component_name.to_string(),
                slot_name: child.mount.clone().unwrap_or_default(),
                child_component,
            });
        }
    }

    for (slot_name, slot) in &component.slots {
        if slot.required && !counts.contains_key(slot_name) {
            return Err(UiAssetError::MissingRequiredSlot {
                component: component_name.to_string(),
                slot_name: slot_name.clone(),
            });
        }
    }

    Ok(())
}

fn prototype_child_component_name(node: &UiNodePrototype) -> Result<Option<String>, UiAssetError> {
    match node.kind {
        UiNodeDefinitionKind::Component => Ok(node.component.clone()),
        UiNodeDefinitionKind::Reference => node
            .component_ref
            .as_deref()
            .map(|reference| {
                parse_component_reference(reference).map(|(_, component)| component.to_string())
            })
            .transpose(),
        UiNodeDefinitionKind::Native => Ok(node.widget_type.clone()),
        UiNodeDefinitionKind::Slot => Ok(None),
    }
}

fn apply_prototype_styles(
    prototype: &UiRawAssetPrototype,
    compiler: &UiDocumentCompiler,
    store: &UiPrototypeStore,
    root: &mut UiTemplateNode,
    artifacts: &PrototypeCompilationArtifacts,
) -> Result<(), UiAssetError> {
    let mut sheets = artifacts.widget_styles.clone();
    for reference in &prototype.imports.styles {
        if let Some(imported) = store.get(reference) {
            let tokens = compose_tokens(&prototype.tokens, &imported.tokens);
            sheets.extend(imported.styles.iter().map(|style| ResolvedStyleSheet {
                stylesheet: style.stylesheet.clone(),
                tokens: tokens.clone(),
            }));
        } else if let Some(document) = compiler.style_imports.get(reference) {
            let tokens = compose_tokens(&prototype.tokens, &document.tokens);
            sheets.extend(
                document
                    .stylesheets
                    .iter()
                    .map(|stylesheet| ResolvedStyleSheet {
                        stylesheet: stylesheet.clone(),
                        tokens: tokens.clone(),
                    }),
            );
        }
    }
    for style in &prototype.styles {
        sheets.push(ResolvedStyleSheet {
            stylesheet: style.stylesheet.clone(),
            tokens: prototype.tokens.clone(),
        });
    }
    if sheets.is_empty() {
        apply_prototype_inline_styles_iterative(root);
        return Ok(());
    }

    let parsed = build_style_plan(&sheets)?;
    let mut path = Vec::new();
    apply_styles_to_tree(root, &parsed, &mut path);
    Ok(())
}

fn apply_prototype_inline_styles_iterative(root: &mut UiTemplateNode) {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        apply_mui_root_slot_props_to_node(node);
        append_mui_style_classes(node);
        if !node.style_overrides.is_empty() {
            apply_mui_sx_to_node(node);
            let (attributes, style_overrides) = (&mut node.attributes, &node.style_overrides);
            merge_value_maps(attributes, style_overrides);
        } else {
            apply_mui_sx_to_node(node);
        }
        apply_mui_child_slot_props(node);
        stack.extend(node.children.iter_mut());
    }
}
