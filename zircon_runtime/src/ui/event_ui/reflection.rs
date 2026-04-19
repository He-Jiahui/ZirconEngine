use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ui::binding::UiEventKind;

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
