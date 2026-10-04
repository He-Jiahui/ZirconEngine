use serde::{Deserialize, Serialize};

/// 固定平面反射捕获目标的边长档位；Medium 为默认值，档位变化会改变纹理预算。
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
