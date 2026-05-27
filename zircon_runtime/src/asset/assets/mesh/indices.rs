use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "format", content = "values", rename_all = "snake_case")]
pub enum MeshIndices {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl MeshIndices {
    pub fn len(&self) -> usize {
        match self {
            Self::U16(values) => values.len(),
            Self::U32(values) => values.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn max_index(&self) -> Option<u32> {
        match self {
            Self::U16(values) => values.iter().map(|index| u32::from(*index)).max(),
            Self::U32(values) => values.iter().copied().max(),
        }
    }

    pub fn to_u32_vec(&self) -> Vec<u32> {
        match self {
            Self::U16(values) => values.iter().map(|index| u32::from(*index)).collect(),
            Self::U32(values) => values.clone(),
        }
    }
}
