use serde::{Deserialize, Serialize};

use crate::core::math::{Real, Vec3};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiBehaviorNodeKind {
    #[default]
    Selector,
    Sequence,
    Parallel,
    Decorator,
    Service,
    Task,
    Subtree,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiBehaviorNodeParameterValue {
    Bool(bool),
    Integer(i64),
    Scalar(Real),
    String(String),
    Vec3(Vec3),
    Entity(u64),
}

impl AiBehaviorNodeParameterValue {
    pub fn is_finite(&self) -> bool {
        match self {
            Self::Scalar(value) => value.is_finite(),
            Self::Vec3(value) => value.is_finite(),
            _ => true,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub const fn value_type(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Integer(_) => "integer",
            Self::Scalar(_) => "scalar",
            Self::String(_) => "string",
            Self::Vec3(_) => "vec3",
            Self::Entity(_) => "entity",
        }
    }
}

impl From<bool> for AiBehaviorNodeParameterValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for AiBehaviorNodeParameterValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<Real> for AiBehaviorNodeParameterValue {
    fn from(value: Real) -> Self {
        Self::Scalar(value)
    }
}

impl From<Vec3> for AiBehaviorNodeParameterValue {
    fn from(value: Vec3) -> Self {
        Self::Vec3(value)
    }
}

impl From<u64> for AiBehaviorNodeParameterValue {
    fn from(value: u64) -> Self {
        Self::Entity(value)
    }
}

impl From<String> for AiBehaviorNodeParameterValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for AiBehaviorNodeParameterValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiBehaviorNodeParameter {
    pub key: String,
    pub value: AiBehaviorNodeParameterValue,
}

impl AiBehaviorNodeParameter {
    pub fn new(key: impl Into<String>, value: impl Into<AiBehaviorNodeParameterValue>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiBehaviorNodeDescriptor {
    pub id: String,
    pub kind: AiBehaviorNodeKind,
    pub display_name: String,
    #[serde(default)]
    pub children: Vec<String>,
    #[serde(default)]
    pub parameters: Vec<AiBehaviorNodeParameter>,
}

impl AiBehaviorNodeDescriptor {
    pub fn new(
        id: impl Into<String>,
        kind: AiBehaviorNodeKind,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            display_name: display_name.into(),
            children: Vec::new(),
            parameters: Vec::new(),
        }
    }

    pub fn with_child(mut self, child: impl Into<String>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn with_parameter(
        mut self,
        key: impl Into<String>,
        value: impl Into<AiBehaviorNodeParameterValue>,
    ) -> Self {
        self.parameters
            .push(AiBehaviorNodeParameter::new(key, value));
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AiBehaviorTreeDescriptor {
    pub id: String,
    pub display_name: String,
    pub root_node: String,
    pub nodes: Vec<AiBehaviorNodeDescriptor>,
}

impl AiBehaviorTreeDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        root_node: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            root_node: root_node.into(),
            nodes: Vec::new(),
        }
    }

    pub fn with_node(mut self, node: AiBehaviorNodeDescriptor) -> Self {
        self.nodes.push(node);
        self
    }
}
