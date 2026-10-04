use std::collections::BTreeMap;

use toml::{map::Map, Value};

use zircon_runtime_interface::ui::template::{
    parse_component_reference, UiAssetDocument, UiAssetError, UiAssetKind, UiChildMount,
    UiComponentDefinition, UiNodeDefinition, UiNodeDefinitionKind, UiTemplateNode,
};

use super::binding_param_resolver::{resolve_node_binding_params, typed_component_params};
use super::control_scope::{
    resolve_binding_control_scope, resolve_optional_control_id, retarget_expanded_root_control_id,
    UiComponentControlScope,
};
use super::ui_document_compiler::{CompilationArtifacts, UiDocumentCompiler};
use super::value_normalizer::{
    append_classes, merge_value_maps, merge_value_maps_resolved, normalize_layout, resolve_value,
    resolve_value_map,
};

impl UiDocumentCompiler {
    // 组件内部节点使用定义资产的参数、token 和控件域；调用点填写的子节点仍按调用方文档与控件域展开。
    pub(super) fn expand_component_instance(
        &self,
        document: &UiAssetDocument,
        component_name: &str,
        instance_node: &UiNodeDefinition,
        tokens: &BTreeMap<String, Value>,
        caller_document: &UiAssetDocument,
        caller_tokens: &BTreeMap<String, Value>,
        params: &BTreeMap<String, Value>,
        caller_scope: Option<&UiComponentControlScope>,
        artifacts: &mut CompilationArtifacts,
    ) -> Result<Vec<UiTemplateNode>, UiAssetError> {
        let component = document.components.get(component_name).ok_or_else(|| {
            UiAssetError::UnknownComponent {
                asset_id: document.asset.id.clone(),
                component: component_name.to_string(),
            }
        })?;
        if document.asset.kind == UiAssetKind::Widget {
            artifacts.record_widget_styles(document, tokens);
        }

        validate_slot_mounts(component_name, component, &instance_node.children)?;

        let component_tokens = super::value_normalizer::compose_tokens(tokens, &document.tokens);
        let component_params =
            resolve_component_params(component, &instance_node.params, &component_tokens, params);
        let component_binding_params =
            typed_component_params(&component.params, &component_params, &document.asset.id)?;
        let mut component_root = component.root.clone();
        let component_scope = UiComponentControlScope::child(
            caller_scope,
            &instance_node.node_id,
            component.root.control_id.as_deref(),
            instance_node.control_id.as_deref(),
        );
        resolve_node_binding_params(
            &mut component_root,
            &component_binding_params,
            &document.asset.id,
        )?;
        // 插槽占位节点会被填充内容取代，先保留它的父容器尺寸约束；填写方的挂载覆写随后具有更高优先级。
        let slot_placeholder_attributes =
            component_slot_placeholder_attributes(component, &component_tokens, &component_params);
        let mut fills = BTreeMap::new();
        for child in &instance_node.children {
            let mount_name = child.mount.clone().unwrap_or_default();
            let mut expanded = self.expand_node(
                caller_document,
                &child.node,
                caller_tokens,
                params,
                None,
                caller_scope,
                artifacts,
            )?;
            if let Some(placeholder_attributes) = slot_placeholder_attributes.get(&mount_name) {
                apply_slot_placeholder_attributes(&mut expanded, placeholder_attributes);
            }
            fills
                .entry(mount_name)
                .or_insert_with(Vec::new)
                // TODO: [CR-UI-TEMPLATE-COMP-0001] 确认导入组件同名 token 时挂载与实例覆写应使用哪个域；子节点已用 caller_tokens 展开，但这里用导入合成域；下一步补同名 token 的挂载/覆写回归并对照原型路径。
                .extend(apply_child_mount(expanded, child, tokens, params));
        }

        let mut roots = self.expand_node(
            document,
            &component_root,
            &component_tokens,
            &component_params,
            Some(&fills),
            Some(&component_scope),
            artifacts,
        )?;
        if roots.len() != 1 {
            return Err(UiAssetError::InvalidDocument {
                asset_id: document.asset.id.clone(),
                detail: format!("component {component_name} must expand to exactly one root node"),
            });
        }

        let mut root = roots.remove(0);
        decorate_component_root(
            &mut root,
            instance_node,
            tokens,
            params,
            caller_scope,
            &caller_document.asset.id,
        )?;
        Ok(vec![root])
    }
}

/// 把父调用点的布局/命名插槽约束附到已经展开的子根；可接收空或多个根，挂载属性属于父容器而非子节点自身样式。
pub(super) fn apply_child_mount(
    nodes: Vec<UiTemplateNode>,
    child: &UiChildMount,
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

// 默认值先形成实例参数环境，调用点覆写随后生效；结果还需转换成 schema 规定的绑定参数类型。
fn resolve_component_params(
    component: &UiComponentDefinition,
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

// 调用点只覆写组件暴露的根；内部绑定保留原来源，附加绑定归调用资产，根别名变化也要重定向内部控件引用。
fn decorate_component_root(
    root: &mut UiTemplateNode,
    instance_node: &UiNodeDefinition,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
    control_scope: Option<&UiComponentControlScope>,
    asset_id: &str,
) -> Result<(), UiAssetError> {
    if let Some(control_id) =
        resolve_optional_control_id(control_scope, instance_node.control_id.as_deref())
    {
        retarget_expanded_root_control_id(root, control_id, asset_id)?;
    }
    append_classes(&mut root.classes, &instance_node.classes);
    let instance_bindings =
        resolve_binding_control_scope(instance_node.bindings.clone(), control_scope, asset_id)?;
    root.binding_source_asset_ids.extend(std::iter::repeat_n(
        asset_id.to_string(),
        instance_bindings.len(),
    ));
    root.bindings.extend(instance_bindings);
    merge_instance_props_override(&mut root.attributes, instance_node, tokens, params);
    merge_instance_layout_override(&mut root.style_overrides, instance_node, tokens, params);
    let inline = resolve_value_map(&instance_node.style_overrides.self_values, tokens, params);
    merge_value_maps(&mut root.style_overrides, &inline);
    merge_value_maps_resolved(
        &mut root.slot_attributes,
        &instance_node.style_overrides.slot,
        tokens,
        params,
    );
    apply_instance_contract_overrides(root, instance_node);
    Ok(())
}

/// Captures the layout contract authored on each component slot placeholder.
/// Mounted children replace the placeholder during expansion, so its own
/// parent-slot sizing must be transferred before the placeholder disappears.
fn component_slot_placeholder_attributes(
    component: &UiComponentDefinition,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) -> BTreeMap<String, BTreeMap<String, Value>> {
    let mut attributes = BTreeMap::new();
    collect_slot_placeholder_attributes(&component.root, tokens, params, &mut attributes);
    attributes
}

fn collect_slot_placeholder_attributes(
    node: &UiNodeDefinition,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
    attributes: &mut BTreeMap<String, BTreeMap<String, Value>>,
) {
    if node.kind == zircon_runtime_interface::ui::template::UiNodeDefinitionKind::Slot {
        if let Some(slot_name) = node.slot_name.as_deref() {
            let mut slot_attributes = BTreeMap::new();
            if let Some(layout) = &node.layout {
                let layout = resolve_value_map(layout, tokens, params);
                slot_attributes.insert(
                    "layout".to_string(),
                    Value::Table(layout.into_iter().collect()),
                );
                normalize_layout(&mut slot_attributes);
            }
            if !slot_attributes.is_empty() {
                attributes.insert(slot_name.to_string(), slot_attributes);
            }
        }
    }
    for child in &node.children {
        collect_slot_placeholder_attributes(&child.node, tokens, params, attributes);
    }
}

// 槽位本身提供基础父布局约束；已展开子节点自带的挂载约束覆盖基础值，不能丢掉两者的嵌套表内容。
fn apply_slot_placeholder_attributes(
    nodes: &mut [UiTemplateNode],
    placeholder_attributes: &BTreeMap<String, Value>,
) {
    for node in nodes {
        let mut inherited = placeholder_attributes.clone();
        merge_value_maps(&mut inherited, &node.slot_attributes);
        node.slot_attributes = inherited;
    }
}

fn merge_instance_props_override(
    target: &mut BTreeMap<String, Value>,
    instance_node: &UiNodeDefinition,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) {
    let props = resolve_value_map(&instance_node.props, tokens, params);
    merge_value_maps(target, &props);
}

// 只传播显式给出的交互契约，缺省调用点继承组件根已有设置，避免覆盖定义中的可访问性与导航行为。
fn apply_instance_contract_overrides(root: &mut UiTemplateNode, instance_node: &UiNodeDefinition) {
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
}

fn merge_instance_layout_override(
    target: &mut BTreeMap<String, Value>,
    instance_node: &UiNodeDefinition,
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

// 接纳规则检查调用方填写的组件身份；先拒绝未知、过量或缺失槽，再展开内部节点，以便把错误定位到调用边界。
fn validate_slot_mounts(
    component_name: &str,
    component: &UiComponentDefinition,
    children: &[UiChildMount],
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
        let child_component =
            child_component_name(&child.node)?.unwrap_or("<unresolved>".to_string());
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

fn child_component_name(node: &UiNodeDefinition) -> Result<Option<String>, UiAssetError> {
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
