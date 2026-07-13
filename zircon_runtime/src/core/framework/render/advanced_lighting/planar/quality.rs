use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanarReflectionQuality {
    Low,
    #[default]
    Medium,
    High,
}

impl PlanarReflectionQuality {
    pub const fn resolution(self) -> u32 {
        match self {
            Self::Low => 256,
            Self::Medium => 512,
            Self::High => 1024,
        }
    }
}
