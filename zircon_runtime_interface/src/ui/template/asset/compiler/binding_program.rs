use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::ui::{
    binding::UiEventKind,
    component::{UiComponentEventKind, UiValue},
    template::{
        UiBindingMissingValuePolicy, UiBindingMode, UI_BINDING_EXPRESSION_MAX_DEPTH,
        UI_BINDING_EXPRESSION_MAX_NODES,
    },
};

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
pub struct UiCompiledBindingHandle {
    pub generation: UiCompiledBindingGeneration,
    pub binding_id: UiBindingId,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
pub struct UiCompiledBindingTarget {
    pub endpoint: UiCompiledBindingTargetEndpoint,
    pub kind: UiCompiledBindingTargetKind,
    pub property: Option<UiPropertyId>,
    #[serde(default)]
    pub missing_policy: UiBindingMissingValuePolicy,
    pub expression: UiCompiledBindingExpression,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
pub struct UiCompiledNodeBindings {
    #[serde(default)]
    pub owner_asset_id: UiCompiledAssetId,
    pub binding_ids: Vec<UiBindingId>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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

    pub fn with_asset_id(mut self, asset_id: impl Into<String>) -> Self {
        self.asset_id = asset_id.into();
        self
    }

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
        if referenced_bindings.iter().any(|referenced| !referenced) {
            return false;
        }

        self.bindings.iter().all(|binding| {
            let mut seen_payload_fields = vec![false; self.properties.len()];
            let mut previous_payload_name: Option<&str> = None;
            let payload_is_well_formed = binding.payload_fields.iter().all(|field| {
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
                match &field.value {
                    UiCompiledActionPayloadValue::Literal(value) => value.is_finite(),
                    UiCompiledActionPayloadValue::Unavailable => true,
                    UiCompiledActionPayloadValue::Expression(expression) => {
                        self.expression_is_well_formed(expression)
                    }
                }
            });
            payload_is_well_formed
                && binding.payload_missing_policy.is_well_formed()
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
                        && self.expression_is_well_formed(&target.expression)
                })
        })
    }

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

    fn expression_is_well_formed(&self, root: &UiCompiledBindingExpression) -> bool {
        let mut pending = vec![(root, 1usize)];
        let mut visited = 0usize;
        while let Some((expression, depth)) = pending.pop() {
            visited += 1;
            if visited > UI_BINDING_EXPRESSION_MAX_NODES || depth > UI_BINDING_EXPRESSION_MAX_DEPTH
            {
                return false;
            }
            match expression {
                UiCompiledBindingExpression::Literal(value) => {
                    if !value.is_finite() {
                        return false;
                    }
                }
                UiCompiledBindingExpression::Property(id) => {
                    if id.get() as usize >= self.properties.len() {
                        return false;
                    }
                }
                UiCompiledBindingExpression::ControlProperty {
                    control_id,
                    property_id,
                } => {
                    if control_id.get() as usize >= self.controls.len()
                        || property_id.get() as usize >= self.properties.len()
                    {
                        return false;
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
        }
        true
    }
}
