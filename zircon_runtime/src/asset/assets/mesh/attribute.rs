use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshAttributeFormat {
    Float32x2,
    Float32x3,
    Float32x4,
    Uint16x4,
    Uint32x4,
}

impl MeshAttributeFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Float32x2 => "float32x2",
            Self::Float32x3 => "float32x3",
            Self::Float32x4 => "float32x4",
            Self::Uint16x4 => "uint16x4",
            Self::Uint32x4 => "uint32x4",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshAttributeSummary {
    pub name: String,
    pub format: MeshAttributeFormat,
    pub len: usize,
    pub is_builtin: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshMorphTargetAttributeSummary {
    pub target_index: usize,
    pub target_name: Option<String>,
    pub attribute: MeshAttributeSummary,
}

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
    pub fn format(&self) -> MeshAttributeFormat {
        match self {
            Self::Float32x2(_) => MeshAttributeFormat::Float32x2,
            Self::Float32x3(_) => MeshAttributeFormat::Float32x3,
            Self::Float32x4(_) => MeshAttributeFormat::Float32x4,
            Self::Uint16x4(_) => MeshAttributeFormat::Uint16x4,
            Self::Uint32x4(_) => MeshAttributeFormat::Uint32x4,
        }
    }

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
