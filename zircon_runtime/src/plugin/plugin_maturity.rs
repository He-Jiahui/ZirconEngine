use serde::{Deserialize, Serialize};

/// Product maturity of a runtime plugin package or catalog descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginMaturity {
    Core,
    Stable,
    Beta,
    Experimental,
    Externalized,
    Stub,
    Deprecated,
}

impl Default for PluginMaturity {
    fn default() -> Self {
        Self::Experimental
    }
}

impl PluginMaturity {
    pub const fn is_unavailable_for_required_profile(self) -> bool {
        matches!(self, Self::Externalized | Self::Stub | Self::Deprecated)
    }

    pub fn meets_minimum(self, minimum: Self) -> bool {
        maturity_rank(self) >= maturity_rank(minimum)
    }
}

fn maturity_rank(maturity: PluginMaturity) -> u8 {
    match maturity {
        PluginMaturity::Stub => 0,
        PluginMaturity::Externalized => 1,
        PluginMaturity::Experimental => 2,
        PluginMaturity::Beta => 3,
        PluginMaturity::Stable => 4,
        PluginMaturity::Core => 5,
        PluginMaturity::Deprecated => 0,
    }
}
