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
    /// 为要求真实提供者的配置标识不可用状态；当前可用性投影另行处理各分类。
    pub const fn is_unavailable_for_required_profile(self) -> bool {
        matches!(self, Self::Externalized | Self::Stub | Self::Deprecated)
    }

    /// 用于配置候选的成熟度门槛；投影调用前还需应用特殊不可用状态规则。
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
