use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum ReflectedValue {
    Null,
    Bool(bool),
    Integer(i64),
    Unsigned(u64),
    Scalar(f32),
    String(String),
    Enum(String),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Quaternion([f32; 4]),
    Entity(Option<u64>),
    Resource(String),
    List(Vec<ReflectedValue>),
    Map(BTreeMap<String, ReflectedValue>),
    Json(serde_json::Value),
}

impl ReflectedValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "Null",
            Self::Bool(_) => "Bool",
            Self::Integer(_) => "Integer",
            Self::Unsigned(_) => "Unsigned",
            Self::Scalar(_) => "Scalar",
            Self::String(_) => "String",
            Self::Enum(_) => "Enum",
            Self::Vec2(_) => "Vec2",
            Self::Vec3(_) => "Vec3",
            Self::Vec4(_) => "Vec4",
            Self::Quaternion(_) => "Quaternion",
            Self::Entity(_) => "Entity",
            Self::Resource(_) => "Resource",
            Self::List(_) => "List",
            Self::Map(_) => "Map",
            Self::Json(_) => "Json",
        }
    }
}
