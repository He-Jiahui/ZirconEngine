use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderHybridGiQuality {
    Low,
    Medium,
    High,
}

impl Default for RenderHybridGiQuality {
    fn default() -> Self {
        Self::Medium
    }
}

impl RenderHybridGiQuality {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}
