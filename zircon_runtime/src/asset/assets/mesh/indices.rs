use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshIndexFormat {
    U16,
    U32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "format", content = "values", rename_all = "snake_case")]
pub enum MeshIndices {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl MeshIndices {
    pub fn format(&self) -> MeshIndexFormat {
        match self {
            Self::U16(_) => MeshIndexFormat::U16,
            Self::U32(_) => MeshIndexFormat::U32,
        }
    }

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

    pub(super) fn for_each_triangle(&self, mut visit: impl FnMut([usize; 3])) {
        match self {
            Self::U16(values) => {
                for triangle in values.chunks_exact(3) {
                    visit([
                        usize::from(triangle[0]),
                        usize::from(triangle[1]),
                        usize::from(triangle[2]),
                    ]);
                }
            }
            Self::U32(values) => {
                for triangle in values.chunks_exact(3) {
                    visit([
                        triangle[0] as usize,
                        triangle[1] as usize,
                        triangle[2] as usize,
                    ]);
                }
            }
        }
    }
}
