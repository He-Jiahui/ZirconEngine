use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "format", content = "values", rename_all = "snake_case")]
pub enum MeshAttributeValues {
    Float32x2(Vec<[f32; 2]>),
    Float32x3(Vec<[f32; 3]>),
    Float32x4(Vec<[f32; 4]>),
    Uint16x4(Vec<[u16; 4]>),
    Uint32x4(Vec<[u32; 4]>),
}

impl MeshAttributeValues {
    pub fn len(&self) -> usize {
        match self {
            Self::Float32x2(values) => values.len(),
            Self::Float32x3(values) => values.len(),
            Self::Float32x4(values) => values.len(),
            Self::Uint16x4(values) => values.len(),
            Self::Uint32x4(values) => values.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_float32x2(&self) -> Option<&[[f32; 2]]> {
        match self {
            Self::Float32x2(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_float32x3(&self) -> Option<&[[f32; 3]]> {
        match self {
            Self::Float32x3(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_float32x4(&self) -> Option<&[[f32; 4]]> {
        match self {
            Self::Float32x4(values) => Some(values),
            _ => None,
        }
    }

    pub fn as_uint16x4(&self) -> Option<&[[u16; 4]]> {
        match self {
            Self::Uint16x4(values) => Some(values),
            _ => None,
        }
    }
}
