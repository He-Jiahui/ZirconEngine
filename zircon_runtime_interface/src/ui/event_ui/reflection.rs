use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ui::binding::UiEventKind;
use crate::ui::component::{UiValue, UiValueKind};
use crate::ui::layout::{UiFrame, UiPoint};
use crate::ui::tree::UiVisibility;
use crate::ui::tree::{UiDirtyFlags, UiInputPolicy};

use super::UiRouteId;

macro_rules! define_id {
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
        pub struct $name(pub u64);

        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }
        }
    };
}

define_id!(UiNodeId);

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiTreeId(pub String);

impl UiTreeId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiNodePath(pub String);

impl UiNodePath {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UiNodePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiValueType {
    Any,
    String,
    Unsigned,
    Signed,
    Float,
    Bool,
    Null,
    Array,
    Object,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStateFlags {
    pub visible: bool,
    pub enabled: bool,
    pub clickable: bool,
    pub hoverable: bool,
    pub focusable: bool,
    pub pressed: bool,
    pub checked: bool,
    pub dirty: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiWidgetLifecycleState {
    #[default]
    Declared,
    Constructed,
    PropertiesSynchronized,
    Arranged,
    Visible,
    Interactive,
    Detached,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiReflectedPropertySource {
    #[default]
    Authored,
    DescriptorDefault,
    InferredDefault,
    RuntimeState,
    Binding,
    SystemState,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPropertyInvalidationReason {
    pub dirty: UiDirtyFlags,
    pub reflection: bool,
}

impl UiPropertyInvalidationReason {
    pub const fn none() -> Self {
        Self {
            dirty: UiDirtyFlags {
                layout: false,
                hit_test: false,
                render: false,
                style: false,
                text: false,
                input: false,
                visible_range: false,
            },
            reflection: false,
        }
    }

    pub const fn reflection_only() -> Self {
        Self {
            dirty: UiDirtyFlags {
                layout: false,
                hit_test: false,
                render: false,
                style: false,
                text: false,
                input: false,
                visible_range: false,
            },
            reflection: true,
        }
    }

    pub const fn with_dirty(dirty: UiDirtyFlags) -> Self {
        Self {
            dirty,
            reflection: true,
        }
    }

    pub const fn any(self) -> bool {
        self.reflection || self.dirty.any()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiReflectedProperty {
    pub name: String,
    pub value_kind: UiValueKind,
    pub resolved_value: UiValue,
    pub authored_value: Option<UiValue>,
    pub descriptor_default_value: Option<UiValue>,
    pub source: UiReflectedPropertySource,
    pub readable: bool,
    pub writable: bool,
    pub invalidation: UiPropertyInvalidationReason,
    pub visibility_affecting: bool,
    pub validation_message: Option<String>,
}

impl Default for UiReflectedProperty {
    fn default() -> Self {
        Self {
            name: String::new(),
            value_kind: UiValueKind::Any,
            resolved_value: UiValue::Null,
            authored_value: None,
            descriptor_default_value: None,
            source: UiReflectedPropertySource::default(),
            readable: true,
            writable: false,
            invalidation: UiPropertyInvalidationReason::none(),
            visibility_affecting: false,
            validation_message: None,
        }
    }
}

impl UiReflectedProperty {
    pub fn new(
        name: impl Into<String>,
        source: UiReflectedPropertySource,
        resolved_value: UiValue,
    ) -> Self {
        Self {
            name: name.into(),
            value_kind: resolved_value.kind(),
            resolved_value,
            source,
            ..Self::default()
        }
    }

    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    pub fn authored_value(mut self, value: UiValue) -> Self {
        self.authored_value = Some(value);
        self
    }

    pub fn descriptor_default_value(mut self, value: UiValue) -> Self {
        self.descriptor_default_value = Some(value);
        self
    }

    pub fn invalidation(mut self, invalidation: UiPropertyInvalidationReason) -> Self {
        self.invalidation = invalidation;
        self
    }

    pub fn visibility_affecting(mut self, visibility_affecting: bool) -> Self {
        self.visibility_affecting = visibility_affecting;
        self
    }

    pub fn validation_message(mut self, message: impl Into<String>) -> Self {
        self.validation_message = Some(message.into());
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiReflectorNode {
    pub node_id: UiNodeId,
    pub node_path: UiNodePath,
    pub parent: Option<UiNodeId>,
    pub children: Vec<UiNodeId>,
    pub class_name: String,
    pub display_name: String,
    pub lifecycle: UiWidgetLifecycleState,
    pub visibility: UiVisibility,
    pub effective_visibility: UiVisibility,
    pub state_flags: UiStateFlags,
    pub input_policy: UiInputPolicy,
    pub z_index: i32,
    pub paint_order: u64,
    pub clip_to_bounds: bool,
    pub frame: UiFrame,
    pub clip_frame: UiFrame,
    pub dirty: UiDirtyFlags,
    pub properties: BTreeMap<String, UiReflectedProperty>,
    pub actions: BTreeMap<String, UiActionDescriptor>,
    pub focused: bool,
    pub hovered: bool,
    pub captured: bool,
    pub pressed: bool,
    pub source_asset: Option<String>,
    pub source_template_path: Option<String>,
}

impl UiReflectorNode {
    pub fn new(
        node_id: UiNodeId,
        node_path: UiNodePath,
        class_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            node_id,
            node_path,
            class_name: class_name.into(),
            display_name: display_name.into(),
            ..Self::default()
        }
    }

    pub fn with_property(mut self, property: UiReflectedProperty) -> Self {
        self.properties.insert(property.name.clone(), property);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiReflectorHitContext {
    pub query_point: UiPoint,
    pub hit_target: Option<UiNodeId>,
    pub hit_stack: Vec<UiNodeId>,
    pub rejected: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiReflectorSnapshot {
    pub tree_id: UiTreeId,
    pub roots: Vec<UiNodeId>,
    pub nodes: BTreeMap<UiNodeId, UiReflectorNode>,
    pub focused: Option<UiNodeId>,
    pub captured: Option<UiNodeId>,
    pub hovered: Vec<UiNodeId>,
    pub hit_context: Option<UiReflectorHitContext>,
}

impl UiReflectorSnapshot {
    pub fn new(tree_id: UiTreeId, roots: Vec<UiNodeId>, nodes: Vec<UiReflectorNode>) -> Self {
        Self {
            tree_id,
            roots,
            nodes: nodes.into_iter().map(|node| (node.node_id, node)).collect(),
            focused: None,
            captured: None,
            hovered: Vec::new(),
            hit_context: None,
        }
    }

    pub fn node(&self, node_id: UiNodeId) -> Option<&UiReflectorNode> {
        self.nodes.get(&node_id)
    }
}

impl UiStateFlags {
    pub const fn visible_enabled(&self) -> bool {
        self.visible && self.enabled
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiParameterDescriptor {
    pub name: String,
    pub value_type: UiValueType,
    pub optional: bool,
}

impl UiParameterDescriptor {
    pub fn new(name: impl Into<String>, value_type: UiValueType) -> Self {
        Self {
            name: name.into(),
            value_type,
            optional: false,
        }
    }

    pub fn optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPropertyDescriptor {
    pub name: String,
    pub value_type: UiValueType,
    pub readable: bool,
    pub writable: bool,
    pub reflected_value: Value,
}

impl UiPropertyDescriptor {
    pub fn new(name: impl Into<String>, value_type: UiValueType, reflected_value: Value) -> Self {
        Self {
            name: name.into(),
            value_type,
            readable: true,
            writable: false,
            reflected_value,
        }
    }

    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiActionDescriptor {
    pub action_id: String,
    pub event_kind: UiEventKind,
    pub binding_symbol: String,
    pub parameter_schema: Vec<UiParameterDescriptor>,
    pub callable_from_remote: bool,
    pub route_id: Option<UiRouteId>,
}

impl UiActionDescriptor {
    pub fn new(
        action_id: impl Into<String>,
        event_kind: UiEventKind,
        binding_symbol: impl Into<String>,
    ) -> Self {
        Self {
            action_id: action_id.into(),
            event_kind,
            binding_symbol: binding_symbol.into(),
            parameter_schema: Vec::new(),
            callable_from_remote: false,
            route_id: None,
        }
    }

    pub fn with_parameter(mut self, parameter: UiParameterDescriptor) -> Self {
        self.parameter_schema.push(parameter);
        self
    }

    pub fn with_callable_from_remote(mut self, callable: bool) -> Self {
        self.callable_from_remote = callable;
        self
    }

    pub fn with_route_id(mut self, route_id: UiRouteId) -> Self {
        self.route_id = Some(route_id);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiNodeDescriptor {
    pub node_id: UiNodeId,
    pub node_path: UiNodePath,
    pub class_name: String,
    pub display_name: String,
    pub children: Vec<UiNodeId>,
    pub state_flags: UiStateFlags,
    #[serde(default)]
    pub visibility: UiVisibility,
    pub properties: BTreeMap<String, UiPropertyDescriptor>,
    pub actions: BTreeMap<String, UiActionDescriptor>,
}

impl UiNodeDescriptor {
    pub fn new(
        node_id: UiNodeId,
        node_path: UiNodePath,
        class_name: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            node_id,
            node_path,
            class_name: class_name.into(),
            display_name: display_name.into(),
            children: Vec::new(),
            state_flags: UiStateFlags::default(),
            visibility: UiVisibility::Visible,
            properties: BTreeMap::new(),
            actions: BTreeMap::new(),
        }
    }

    pub fn with_child(mut self, child: UiNodeId) -> Self {
        self.children.push(child);
        self
    }

    pub fn with_state_flags(mut self, state_flags: UiStateFlags) -> Self {
        self.state_flags = state_flags;
        self
    }

    pub fn with_visibility(mut self, visibility: UiVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn with_property(mut self, property: UiPropertyDescriptor) -> Self {
        self.properties.insert(property.name.clone(), property);
        self
    }

    pub fn with_action(mut self, action: UiActionDescriptor) -> Self {
        let key = action.action_id.clone();
        self.actions.insert(key, action);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiReflectionSnapshot {
    pub tree_id: UiTreeId,
    pub roots: Vec<UiNodeId>,
    pub nodes: BTreeMap<UiNodeId, UiNodeDescriptor>,
}

impl UiReflectionSnapshot {
    pub fn new(tree_id: UiTreeId, roots: Vec<UiNodeId>, nodes: Vec<UiNodeDescriptor>) -> Self {
        Self {
            tree_id,
            roots,
            nodes: nodes.into_iter().map(|node| (node.node_id, node)).collect(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiReflectionDiff {
    pub tree_id: UiTreeId,
    pub changed_nodes: Vec<UiNodeId>,
    pub removed_nodes: Vec<UiNodeId>,
}

impl UiReflectionDiff {
    pub fn is_empty(&self) -> bool {
        self.changed_nodes.is_empty() && self.removed_nodes.is_empty()
    }
}
