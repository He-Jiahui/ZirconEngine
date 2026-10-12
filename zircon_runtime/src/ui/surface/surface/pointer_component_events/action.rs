use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::UiValue,
    dispatch::UiTemplateActionInvocation,
    event_ui::UiNodeId,
    template::{
        UiBindingExpression, UiBindingMissingValueResolution, UiBindingRef,
        UiCompiledBindingHandle, UiCompiledNodeId, UiPropertyId,
    },
};

use super::UiSurface;

// 将已验证来源的作者绑定投影为模板动作调用，不在这里执行动作；缺失、歧义或禁止发布统一返回 None。
// 普通/编译绑定各用本身索引，属性优先读取运行时组件值；跨控件取值必须唯一，避免选中任意同名控件。
impl UiSurface {
    pub(crate) fn template_action_for_binding(
        &self,
        source_node_id: UiNodeId,
        binding: &UiBindingRef,
    ) -> Option<UiTemplateActionInvocation> {
        self.template_action_for_binding_with_overrides(
            source_node_id,
            binding,
            &std::collections::BTreeMap::new(),
        )
    }

    /// 原始绑定入口先解析缺失策略，再合并具名覆盖；生产默认调用使用空覆盖集合。
    // TODO: [CR-UI-SURFACE-0011] 核实覆盖是否应满足 required payload：编译入口先取覆盖再判断缺失，
    // 原始入口却在覆盖合并前因缺失返回 None；当前生产调用为空，需补两条路径的非空覆盖契约测试。
    pub(crate) fn template_action_for_binding_with_overrides(
        &self,
        source_node_id: UiNodeId,
        binding: &UiBindingRef,
        payload_overrides: &std::collections::BTreeMap<String, UiValue>,
    ) -> Option<UiTemplateActionInvocation> {
        if !self.tree.node(source_node_id)?.state_flags.enabled {
            return None;
        }
        let action = binding.action.as_ref();
        let route = action
            .and_then(|action| action.route.as_deref())
            .or(binding.route.as_deref())
            .map(str::trim)
            .filter(|id| !id.is_empty());
        let action_id = action
            .and_then(|action| action.action.as_deref())
            .map(str::trim)
            .filter(|id| !id.is_empty());
        let route = match (route, action_id) {
            (None, Some(action_id)) if action.is_some_and(|action| action.payload.is_empty()) => {
                return Some(UiTemplateActionInvocation::action(action_id));
            }
            (Some(route), None) => route,
            _ => return None,
        };

        let mut payload = BTreeMap::new();
        if let Some(action) = action {
            for (key, value) in &action.payload {
                match action
                    .payload_missing_policy
                    .resolve(self.template_action_payload_value(source_node_id, value))
                {
                    UiBindingMissingValueResolution::Value(value) => {
                        payload.insert(key.clone(), value);
                    }
                    UiBindingMissingValueResolution::Omitted => {}
                    UiBindingMissingValueResolution::RequiredMissing
                    | UiBindingMissingValueResolution::ExplicitError => return None,
                }
            }
        }
        payload.extend(payload_overrides.clone());
        Some(UiTemplateActionInvocation::route(route, payload))
    }

    pub(crate) fn template_action_for_compiled_binding_with_overrides(
        &self,
        source_node_id: UiNodeId,
        handle: UiCompiledBindingHandle,
        mut payload_overrides: BTreeMap<UiPropertyId, UiValue>,
    ) -> Option<UiTemplateActionInvocation> {
        if !self.tree.node(source_node_id)?.state_flags.enabled {
            return None;
        }
        let binding = self.compiled_bindings.binding(handle)?;
        match (binding.route_id, binding.action_id) {
            (None, Some(action_id)) if binding.payload_fields.is_empty() => {
                return self
                    .compiled_bindings
                    .action_name(action_id)
                    .map(UiTemplateActionInvocation::action);
            }
            (Some(route_id), None) => {
                let route = self.compiled_bindings.route_name(route_id)?;
                let mut payload = BTreeMap::new();
                for payload_field in &binding.payload_fields {
                    let field = self
                        .compiled_bindings
                        .property_name(payload_field.property)?;
                    let resolved =
                        payload_overrides
                            .remove(&payload_field.property)
                            .or_else(|| {
                                self.resolve_compiled_action_payload_value(
                                    source_node_id,
                                    &payload_field.value,
                                )
                            });
                    match binding.payload_missing_policy.resolve(resolved) {
                        UiBindingMissingValueResolution::Value(value) => {
                            payload.insert(field.to_string(), value);
                        }
                        UiBindingMissingValueResolution::Omitted => {}
                        UiBindingMissingValueResolution::RequiredMissing
                        | UiBindingMissingValueResolution::ExplicitError => return None,
                    }
                }
                return Some(UiTemplateActionInvocation::route(route, payload));
            }
            _ => {}
        }
        None
    }

    #[cfg(test)]
    pub(crate) fn dense_compiled_payload_overrides_for_benchmark(
        &self,
        handle: UiCompiledBindingHandle,
        overrides: BTreeMap<String, UiValue>,
    ) -> Option<BTreeMap<UiPropertyId, UiValue>> {
        let binding = self.compiled_bindings.binding(handle)?;
        overrides
            .into_iter()
            .map(|(name, value)| {
                let property = binding.payload_fields.iter().find_map(|field| {
                    (self.compiled_bindings.property_name(field.property) == Some(name.as_str()))
                        .then_some(field.property)
                })?;
                Some((property, value))
            })
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn template_action_for_compiled_binding_with_legacy_overrides_for_benchmark(
        &self,
        source_node_id: UiNodeId,
        handle: UiCompiledBindingHandle,
        payload_overrides: BTreeMap<String, UiValue>,
    ) -> Option<UiTemplateActionInvocation> {
        if !self.tree.node(source_node_id)?.state_flags.enabled {
            return None;
        }
        let binding = self.compiled_bindings.binding(handle)?;
        let route_id = binding.route_id?;
        if binding.action_id.is_some() {
            return None;
        }
        let route = self.compiled_bindings.route_name(route_id)?;
        let mut payload = BTreeMap::new();
        for payload_field in &binding.payload_fields {
            let field = self
                .compiled_bindings
                .property_name(payload_field.property)?;
            let resolved = payload_overrides.get(field).cloned().or_else(|| {
                self.resolve_compiled_action_payload_value(source_node_id, &payload_field.value)
            });
            match binding.payload_missing_policy.resolve(resolved) {
                UiBindingMissingValueResolution::Value(value) => {
                    payload.insert(field.to_string(), value);
                }
                UiBindingMissingValueResolution::Omitted => {}
                UiBindingMissingValueResolution::RequiredMissing
                | UiBindingMissingValueResolution::ExplicitError => return None,
            }
        }
        Some(UiTemplateActionInvocation::route(route, payload))
    }

    /// 为无直接编译句柄的来源按模板节点编号约定查找，并复核事件与绑定名；失败应交回原始路径/上层处理。
    /// 仅适用于保留树沿用编译节点编号加一的映射，不能把任意运行时 ID 当成模板索引。
    pub(crate) fn compiled_binding_handle_for_source(
        &self,
        node_id: UiNodeId,
        source_binding_index: usize,
        binding: &UiBindingRef,
        event_kind: UiEventKind,
    ) -> Option<UiCompiledBindingHandle> {
        node_id
            .0
            .checked_sub(1)
            .and_then(|node_index| u32::try_from(node_index).ok())
            .and_then(|node_index| {
                self.compiled_bindings.handle_for_node_binding(
                    UiCompiledNodeId::new(node_index),
                    source_binding_index,
                )
            })
            .filter(|handle| {
                self.compiled_bindings
                    .binding(*handle)
                    .is_some_and(|compiled| {
                        compiled.event == event_kind
                            && self.compiled_bindings.binding_name(*handle)
                                == Some(binding.id.as_str())
                    })
            })
    }

    /// 仅以表达式前缀区分字面量与绑定表达式；动作投影只允许本节点或唯一控件属性，外部状态需事先投影。
    pub(crate) fn template_action_payload_value(
        &self,
        source_node_id: UiNodeId,
        value: &toml::Value,
    ) -> Option<UiValue> {
        let toml::Value::String(expression_text) = value else {
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
                        |property| self.template_action_property_value(source_node_id, property),
                        |control_id, property| {
                            self.template_action_control_property_value(control_id, property)
                        },
                    )
                    .ok()
            })
    }

    pub(crate) fn template_action_control_property_value(
        &self,
        control_id: &str,
        property: &str,
    ) -> Option<UiValue> {
        self.control_index
            .unique_node_id_for_surface(&self.tree, control_id)
            .map(|node_id| self.template_action_property_value(node_id, property))
            .flatten()
    }

    pub(crate) fn template_action_property_value(
        &self,
        node_id: UiNodeId,
        property: &str,
    ) -> Option<UiValue> {
        self.component_states
            .get(node_id)
            .and_then(|state| state.value(property))
            .cloned()
            .or_else(|| {
                self.tree
                    .node(node_id)
                    .and_then(|node| node.template_metadata.as_ref())
                    .and_then(|metadata| metadata.attributes.get(property))
                    .map(UiValue::from_toml)
            })
    }
}
