use serde::{Deserialize, Serialize};

use crate::core::math::{Real, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiBlackboardValueType {
    Bool,
    Integer,
    Scalar,
    String,
    Vec3,
    Entity,
}

impl AiBlackboardValueType {
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        if value.eq_ignore_ascii_case("bool") || value.eq_ignore_ascii_case("boolean") {
            Some(Self::Bool)
        } else if value.eq_ignore_ascii_case("integer")
            || value.eq_ignore_ascii_case("int")
            || value.eq_ignore_ascii_case("i64")
        {
            Some(Self::Integer)
        } else if value.eq_ignore_ascii_case("scalar")
            || value.eq_ignore_ascii_case("float")
            || value.eq_ignore_ascii_case("real")
            || value.eq_ignore_ascii_case("f32")
        {
            Some(Self::Scalar)
        } else if value.eq_ignore_ascii_case("string") || value.eq_ignore_ascii_case("str") {
            Some(Self::String)
        } else if value.eq_ignore_ascii_case("vec3") || value.eq_ignore_ascii_case("vector3") {
            Some(Self::Vec3)
        } else if value.eq_ignore_ascii_case("entity") || value.eq_ignore_ascii_case("entity_id") {
            Some(Self::Entity)
        } else {
            None
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Integer => "integer",
            Self::Scalar => "scalar",
            Self::String => "string",
            Self::Vec3 => "vec3",
            Self::Entity => "entity",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiBlackboardValue {
    Bool(bool),
    Integer(i64),
    Scalar(Real),
    String(String),
    Vec3(Vec3),
    Entity(u64),
}

impl AiBlackboardValue {
    pub const fn value_type(&self) -> AiBlackboardValueType {
        match self {
            Self::Bool(_) => AiBlackboardValueType::Bool,
            Self::Integer(_) => AiBlackboardValueType::Integer,
            Self::Scalar(_) => AiBlackboardValueType::Scalar,
            Self::String(_) => AiBlackboardValueType::String,
            Self::Vec3(_) => AiBlackboardValueType::Vec3,
            Self::Entity(_) => AiBlackboardValueType::Entity,
        }
    }

    pub fn is_finite(&self) -> bool {
        match self {
            Self::Scalar(value) => value.is_finite(),
            Self::Vec3(value) => value.is_finite(),
            _ => true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiBlackboardEntry {
    pub key: String,
    pub value: AiBlackboardValue,
}

impl AiBlackboardEntry {
    pub fn new(key: impl Into<String>, value: AiBlackboardValue) -> Self {
        Self {
            key: key.into(),
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiBlackboardKeyDescriptor {
    pub key: String,
    pub value_type: String,
    pub required: bool,
}

impl AiBlackboardKeyDescriptor {
    pub fn expected_value_type(&self) -> Option<AiBlackboardValueType> {
        AiBlackboardValueType::parse(&self.value_type)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiBlackboardSchemaDescriptor {
    pub id: String,
    pub display_name: String,
    pub keys: Vec<AiBlackboardKeyDescriptor>,
}

impl AiBlackboardSchemaDescriptor {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            keys: Vec::new(),
        }
    }

    pub fn with_key(
        mut self,
        key: impl Into<String>,
        value_type: impl Into<String>,
        required: bool,
    ) -> Self {
        self.keys.push(AiBlackboardKeyDescriptor {
            key: key.into(),
            value_type: value_type.into(),
            required,
        });
        self
    }
}
