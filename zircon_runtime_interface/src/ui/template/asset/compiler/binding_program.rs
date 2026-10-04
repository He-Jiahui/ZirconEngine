use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ui::{
    binding::UiEventKind,
    component::{UiComponentEventKind, UiValue},
    template::{
        UiBindingMissingValuePolicy, UiBindingMode, UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY,
        UI_BINDING_EXPRESSION_MAX_DEPTH, UI_BINDING_EXPRESSION_MAX_NODES,
    },
};

// 为不同编译表生成互不混用的轻量索引新类型；数值仍是各自稠密表中的位置。
macro_rules! dense_id {
    ($name:ident) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            Default,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
        )]
        pub struct $name(u32);

        impl $name {
            pub const fn new(value: u32) -> Self {
                Self(value)
            }

            pub const fn get(self) -> u32 {
                self.0
            }
        }
    };
}

dense_id!(UiBindingId);
dense_id!(UiPropertyId);
dense_id!(UiCompiledRouteId);
dense_id!(UiCompiledActionId);
dense_id!(UiCompiledControlId);
dense_id!(UiCompiledNodeId);
dense_id!(UiCompiledBindingTargetId);
dense_id!(UiCompiledAssetId);

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
/// 一份编译绑定程序的代际标识；零保留为 Default/空程序的无效哨兵。
pub struct UiCompiledBindingGeneration(u64);

impl UiCompiledBindingGeneration {
    pub const INVALID: Self = Self(0);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn is_invalid(self) -> bool {
        self.0 == Self::INVALID.0
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
/// 绑定表句柄把稠密绑定索引和程序代际组合，热替换后旧句柄不会误命中新表中的同号行。
pub struct UiCompiledBindingHandle {
    pub generation: UiCompiledBindingGeneration,
    pub binding_id: UiBindingId,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// 目标端点用程序代际、节点、绑定和目标序号定位一次写入；`is_well_formed` 校验四项均对应所属绑定目标行。
pub struct UiCompiledBindingTargetEndpoint {
    pub generation: UiCompiledBindingGeneration,
    pub node_id: UiCompiledNodeId,
    pub binding_id: UiBindingId,
    pub target_index: UiCompiledBindingTargetId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiCompiledBindingTargetKind {
    Property,
    Class,
    Visibility,
    Enabled,
    ActionPayload,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 名称解析完成后的表达式；属性和控件引用已转成同一程序字符串表的类型化索引。
pub enum UiCompiledBindingExpression {
    Literal(UiValue),
    Property(UiPropertyId),
    ControlProperty {
        control_id: UiCompiledControlId,
        property_id: UiPropertyId,
    },
    Equals(
        Box<UiCompiledBindingExpression>,
        Box<UiCompiledBindingExpression>,
    ),
    NotEquals(
        Box<UiCompiledBindingExpression>,
        Box<UiCompiledBindingExpression>,
    ),
    And(
        Box<UiCompiledBindingExpression>,
        Box<UiCompiledBindingExpression>,
    ),
    Or(
        Box<UiCompiledBindingExpression>,
        Box<UiCompiledBindingExpression>,
    ),
    Not(Box<UiCompiledBindingExpression>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Payload 字段区分立即值、编译后的动态表达式和不可用值，避免把无法编译的源文本伪装成字面量。
pub enum UiCompiledActionPayloadValue {
    Literal(UiValue),
    Expression(UiCompiledBindingExpression),
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiCompiledActionPayloadField {
    pub property: UiPropertyId,
    pub value: UiCompiledActionPayloadValue,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 一项已编译状态目标；仅 Property/Class/ActionPayload 需要 property 索引，Visibility/Enabled 使用无属性端点。
pub struct UiCompiledBindingTarget {
    pub endpoint: UiCompiledBindingTargetEndpoint,
    pub kind: UiCompiledBindingTargetKind,
    pub property: Option<UiPropertyId>,
    #[serde(default)]
    pub missing_policy: UiBindingMissingValuePolicy,
    pub expression: UiCompiledBindingExpression,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 一个按节点源顺序排列的事件绑定，串接触发类型、动作路由、payload 及其目标写入。
pub struct UiCompiledBinding {
    pub handle: UiCompiledBindingHandle,
    #[serde(default)]
    pub owner_asset_id: UiCompiledAssetId,
    pub node_id: UiCompiledNodeId,
    pub source_binding_index: u32,
    pub event: UiEventKind,
    pub mode: UiBindingMode,
    #[serde(default)]
    pub component_event: Option<UiComponentEventKind>,
    pub route_id: Option<UiCompiledRouteId>,
    pub action_id: Option<UiCompiledActionId>,
    #[serde(default)]
    pub payload_missing_policy: UiBindingMissingValuePolicy,
    pub payload_fields: Vec<UiCompiledActionPayloadField>,
    pub targets: Vec<UiCompiledBindingTarget>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 节点按源顺序列出绑定表中的稠密 ID，并记录所属资产，合并包后仍可追溯来源。
pub struct UiCompiledNodeBindings {
    #[serde(default)]
    pub owner_asset_id: UiCompiledAssetId,
    pub binding_ids: Vec<UiBindingId>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 可序列化的绑定运行程序和其字符串查找表；代际及稠密索引共同构成跨编辑器/运行时的数据契约。
pub struct UiCompiledBindingProgram {
    generation: UiCompiledBindingGeneration,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    asset_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    asset_ids: Vec<String>,
    binding_names: Vec<String>,
    properties: Vec<String>,
    routes: Vec<String>,
    actions: Vec<String>,
    controls: Vec<String>,
    nodes: Vec<UiCompiledNodeBindings>,
    bindings: Vec<UiCompiledBinding>,
}

impl UiCompiledBindingProgram {
    /// 组合代际、名称表、节点索引和绑定行；调用方在发布或恢复序列化数据前应检查 `is_well_formed`。
    pub fn new(
        generation: UiCompiledBindingGeneration,
        binding_names: Vec<String>,
        properties: Vec<String>,
        routes: Vec<String>,
        actions: Vec<String>,
        controls: Vec<String>,
        nodes: Vec<UiCompiledNodeBindings>,
        bindings: Vec<UiCompiledBinding>,
    ) -> Self {
        Self {
            generation,
            asset_id: String::new(),
            asset_ids: Vec::new(),
            binding_names,
            properties,
            routes,
            actions,
            controls,
            nodes,
            bindings,
        }
    }

    pub const fn generation(&self) -> UiCompiledBindingGeneration {
        self.generation
    }

    /// 设置单资产名称回退；未提供 asset_ids 表时，节点和绑定所有权查询使用该值。
    pub fn with_asset_id(mut self, asset_id: impl Into<String>) -> Self {
        self.asset_id = asset_id.into();
        self
    }

    /// 安装合并资产包的名称表，使 node/binding owner_asset_id 可解析为实际资产身份。
    pub fn with_asset_ownership(mut self, asset_ids: Vec<String>) -> Self {
        self.asset_ids = asset_ids;
        self
    }

    pub fn asset_id(&self) -> Option<&str> {
        (!self.asset_id.is_empty()).then_some(self.asset_id.as_str())
    }

    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    pub fn asset_count(&self) -> usize {
        self.asset_ids
            .len()
            .max(if self.asset_id.is_empty() { 0 } else { 1 })
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn control_count(&self) -> usize {
        self.controls.len()
    }

    pub fn iter_bindings(&self) -> impl ExactSizeIterator<Item = &UiCompiledBinding> {
        self.bindings.iter()
    }

    pub fn iter_nodes(
        &self,
    ) -> impl ExactSizeIterator<Item = (UiCompiledNodeId, &UiCompiledNodeBindings)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (UiCompiledNodeId::new(index as u32), node))
    }

    pub fn node_asset_id(&self, node_id: UiCompiledNodeId) -> Option<&str> {
        let node = self.nodes.get(node_id.get() as usize)?;
        self.owner_asset_name(node.owner_asset_id)
    }

    pub fn binding_asset_id(&self, handle: UiCompiledBindingHandle) -> Option<&str> {
        let binding = self.binding(handle)?;
        self.owner_asset_name(binding.owner_asset_id)
    }

    pub fn iter_control_names(&self) -> impl ExactSizeIterator<Item = &str> {
        self.controls.iter().map(String::as_str)
    }

    /// 校验程序代际、表索引、所有权、payload 唯一有序字段、目标端点和表达式预算；编译、入包和热替换入口据此拒收坏数据。
    pub fn is_well_formed(&self) -> bool {
        if self.generation.is_invalid() {
            return self.asset_id.is_empty()
                && self.asset_ids.is_empty()
                && self.binding_names.is_empty()
                && self.properties.is_empty()
                && self.routes.is_empty()
                && self.actions.is_empty()
                && self.controls.is_empty()
                && self.nodes.is_empty()
                && self.bindings.is_empty();
        }
        if self.binding_names.len() != self.bindings.len() {
            return false;
        }
        if !self.asset_ids.is_empty() {
            let mut unique_assets = BTreeSet::new();
            if self
                .asset_ids
                .iter()
                .any(|asset| asset.is_empty() || !unique_assets.insert(asset.as_str()))
            {
                return false;
            }
        }

        if !self.node_bindings_are_well_formed() {
            return false;
        }

        let mut seen_payload_fields = vec![false; self.properties.len()];
        let mut expression_pending =
            Vec::with_capacity(UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY);
        self.bindings.iter().all(|binding| {
            self.payload_fields_are_well_formed(
                binding,
                &mut seen_payload_fields,
                &mut expression_pending,
            ) && binding.payload_missing_policy.is_well_formed()
                && binding
                    .route_id
                    .is_none_or(|id| (id.get() as usize) < self.routes.len())
                && binding
                    .action_id
                    .is_none_or(|id| (id.get() as usize) < self.actions.len())
                && binding.targets.iter().enumerate().all(|(index, target)| {
                    target.endpoint.generation == self.generation
                        && target.endpoint.node_id == binding.node_id
                        && target.endpoint.binding_id == binding.handle.binding_id
                        && target.endpoint.target_index.get() as usize == index
                        && self.target_property_is_valid(target)
                        && target.missing_policy.is_well_formed()
                        && self
                            .expression_is_well_formed(&target.expression, &mut expression_pending)
                })
        })
    }

    /// 将节点内源绑定序号转换为带当前程序代际的句柄；节点或源序号越界时返回 `None`。
    pub fn handle_for_node_binding(
        &self,
        node_id: UiCompiledNodeId,
        source_binding_index: usize,
    ) -> Option<UiCompiledBindingHandle> {
        let binding_id = *self
            .nodes
            .get(node_id.get() as usize)?
            .binding_ids
            .get(source_binding_index)?;
        Some(UiCompiledBindingHandle {
            generation: self.generation,
            binding_id,
        })
    }

    /// 只解析本程序代际且行内句柄完全相等的绑定；旧代际、默认句柄或被错位的行均不返回。
    pub fn binding(&self, handle: UiCompiledBindingHandle) -> Option<&UiCompiledBinding> {
        if handle.generation != self.generation || self.generation.is_invalid() {
            return None;
        }
        let binding = self.bindings.get(handle.binding_id.get() as usize)?;
        (binding.handle == handle).then_some(binding)
    }

    pub fn binding_name(&self, handle: UiCompiledBindingHandle) -> Option<&str> {
        self.binding(handle)?;
        self.binding_names
            .get(handle.binding_id.get() as usize)
            .map(String::as_str)
    }

    pub fn property_name(&self, id: UiPropertyId) -> Option<&str> {
        self.properties.get(id.get() as usize).map(String::as_str)
    }

    pub fn route_name(&self, id: UiCompiledRouteId) -> Option<&str> {
        self.routes.get(id.get() as usize).map(String::as_str)
    }

    pub fn action_name(&self, id: UiCompiledActionId) -> Option<&str> {
        self.actions.get(id.get() as usize).map(String::as_str)
    }

    pub fn control_name(&self, id: UiCompiledControlId) -> Option<&str> {
        self.controls.get(id.get() as usize).map(String::as_str)
    }

    fn owner_asset_name(&self, id: UiCompiledAssetId) -> Option<&str> {
        if self.asset_ids.is_empty() {
            return self.asset_id();
        }
        self.asset_ids.get(id.get() as usize).map(String::as_str)
    }

    fn owner_asset_id_is_valid(&self, id: UiCompiledAssetId) -> bool {
        self.asset_ids.is_empty() || (id.get() as usize) < self.asset_ids.len()
    }

    // 编译器按节点和源顺序生成连续 binding id；此处核对节点映射、行内句柄、源序号及 owner asset 索引范围。
    fn node_bindings_are_well_formed(&self) -> bool {
        let mut expected_binding_index = 0usize;
        for (node_index, node) in self.nodes.iter().enumerate() {
            if !self.owner_asset_id_is_valid(node.owner_asset_id) {
                return false;
            }
            for (source_binding_index, binding_id) in node.binding_ids.iter().enumerate() {
                let Some(binding) = self.bindings.get(binding_id.get() as usize) else {
                    return false;
                };
                if binding_id.get() as usize != expected_binding_index
                    || binding.handle.generation != self.generation
                    || binding.handle.binding_id != *binding_id
                    || binding.node_id.get() as usize != node_index
                    || binding.source_binding_index as usize != source_binding_index
                    || !self.owner_asset_id_is_valid(binding.owner_asset_id)
                {
                    return false;
                }
                expected_binding_index += 1;
            }
        }
        if expected_binding_index != self.bindings.len() {
            return false;
        }
        true
    }

    #[cfg(test)]
    fn node_bindings_are_well_formed_bitmap(&self) -> bool {
        let mut referenced_bindings = vec![false; self.bindings.len()];
        let mut expected_binding_index = 0usize;
        for (node_index, node) in self.nodes.iter().enumerate() {
            if !self.owner_asset_id_is_valid(node.owner_asset_id) {
                return false;
            }
            for (source_binding_index, binding_id) in node.binding_ids.iter().enumerate() {
                let Some(binding) = self.bindings.get(binding_id.get() as usize) else {
                    return false;
                };
                if binding_id.get() as usize != expected_binding_index
                    || referenced_bindings[binding_id.get() as usize]
                    || binding.handle.generation != self.generation
                    || binding.handle.binding_id != *binding_id
                    || binding.node_id.get() as usize != node_index
                    || binding.source_binding_index as usize != source_binding_index
                    || !self.owner_asset_id_is_valid(binding.owner_asset_id)
                {
                    return false;
                }
                referenced_bindings[binding_id.get() as usize] = true;
                expected_binding_index += 1;
            }
        }
        !referenced_bindings.iter().any(|referenced| !referenced)
    }

    fn target_property_is_valid(&self, target: &UiCompiledBindingTarget) -> bool {
        match target.kind {
            UiCompiledBindingTargetKind::Property
            | UiCompiledBindingTargetKind::Class
            | UiCompiledBindingTargetKind::ActionPayload => target
                .property
                .is_some_and(|id| (id.get() as usize) < self.properties.len()),
            UiCompiledBindingTargetKind::Visibility | UiCompiledBindingTargetKind::Enabled => {
                target.property.is_none()
            }
        }
    }

    // 同一属性位图跨绑定复用：先按属性名检查严格递增并阻止单个 binding 重复，再清掉本行触碰的位。
    fn payload_fields_are_well_formed<'expression>(
        &self,
        binding: &'expression UiCompiledBinding,
        seen_payload_fields: &mut [bool],
        expression_pending: &mut Vec<(&'expression UiCompiledBindingExpression, usize)>,
    ) -> bool {
        let mut previous_payload_name: Option<&str> = None;
        for field in &binding.payload_fields {
            let property_index = field.property.get() as usize;
            if property_index >= self.properties.len() || seen_payload_fields[property_index] {
                return false;
            }
            let property_name = self.properties[property_index].as_str();
            if previous_payload_name.is_some_and(|previous| previous >= property_name) {
                return false;
            }
            previous_payload_name = Some(property_name);
            seen_payload_fields[property_index] = true;
            let value_is_well_formed = match &field.value {
                UiCompiledActionPayloadValue::Literal(value) => value.is_finite(),
                UiCompiledActionPayloadValue::Unavailable => true,
                UiCompiledActionPayloadValue::Expression(expression) => {
                    self.expression_is_well_formed(expression, expression_pending)
                }
            };
            if !value_is_well_formed {
                return false;
            }
        }
        // 清除仅为当前 binding 标记的属性位，下一绑定可合法复用同名字段。
        for field in &binding.payload_fields {
            let property_index = field.property.get() as usize;
            seen_payload_fields[property_index] = false;
        }
        true
    }

    #[cfg(test)]
    fn payload_fields_are_well_formed_reused(&self) -> bool {
        let mut seen_payload_fields = vec![false; self.properties.len()];
        let mut expression_pending =
            Vec::with_capacity(UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY);
        self.bindings.iter().all(|binding| {
            self.payload_fields_are_well_formed(
                binding,
                &mut seen_payload_fields,
                &mut expression_pending,
            )
        })
    }

    #[cfg(test)]
    fn payload_fields_are_well_formed_allocating(&self) -> bool {
        let mut expression_pending =
            Vec::with_capacity(UI_BINDING_EXPRESSION_INLINE_STACK_CAPACITY);
        self.bindings.iter().all(|binding| {
            let mut seen_payload_fields = vec![false; self.properties.len()];
            self.payload_fields_are_well_formed(
                binding,
                &mut seen_payload_fields,
                &mut expression_pending,
            )
        })
    }

    // 用可复用的显式栈检查深度、节点数、有限字面量和索引边界，避免验证反序列化树时递归下探。
    fn expression_is_well_formed<'expression>(
        &self,
        root: &'expression UiCompiledBindingExpression,
        pending: &mut Vec<(&'expression UiCompiledBindingExpression, usize)>,
    ) -> bool {
        // 每次从空栈开始，防止上次失败留下的遍历状态污染后续表达式。
        pending.clear();
        pending.push((root, 1));
        let mut visited = 0usize;
        let is_well_formed = loop {
            let Some((expression, depth)) = pending.pop() else {
                break true;
            };
            visited += 1;
            if visited > UI_BINDING_EXPRESSION_MAX_NODES || depth > UI_BINDING_EXPRESSION_MAX_DEPTH
            {
                break false;
            }
            match expression {
                UiCompiledBindingExpression::Literal(value) => {
                    if !value.is_finite() {
                        break false;
                    }
                }
                UiCompiledBindingExpression::Property(id) => {
                    if id.get() as usize >= self.properties.len() {
                        break false;
                    }
                }
                UiCompiledBindingExpression::ControlProperty {
                    control_id,
                    property_id,
                } => {
                    if control_id.get() as usize >= self.controls.len()
                        || property_id.get() as usize >= self.properties.len()
                    {
                        break false;
                    }
                }
                UiCompiledBindingExpression::Equals(lhs, rhs)
                | UiCompiledBindingExpression::NotEquals(lhs, rhs)
                | UiCompiledBindingExpression::And(lhs, rhs)
                | UiCompiledBindingExpression::Or(lhs, rhs) => {
                    pending.push((lhs, depth + 1));
                    pending.push((rhs, depth + 1));
                }
                UiCompiledBindingExpression::Not(value) => pending.push((value, depth + 1)),
            }
        };
        pending.clear();
        is_well_formed
    }

    #[cfg(test)]
    fn expression_is_well_formed_allocating(&self, root: &UiCompiledBindingExpression) -> bool {
        let mut pending = vec![(root, 1usize)];
        self.expression_is_well_formed(root, &mut pending)
    }
}

#[cfg(test)]
#[path = "binding_program/tests/expression_stack_performance_tests.rs"]
mod expression_stack_performance_tests;

#[cfg(test)]
#[path = "binding_program/tests/payload_bitmap_performance_tests.rs"]
mod payload_bitmap_performance_tests;

#[cfg(test)]
#[path = "binding_program/tests/reference_bitmap_performance_tests.rs"]
mod reference_bitmap_performance_tests;
