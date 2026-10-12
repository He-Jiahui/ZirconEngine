use std::fmt;
use std::num::NonZeroU64;

/// 代号从 NonZeroU64 的 INITIAL=1 开始递增；checked_next 溢出时返回 None，候选不得回绕复用旧代。
/// Monotonic published revision owned by one runtime plugin catalog authority.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginCatalogGeneration(NonZeroU64);

impl PluginCatalogGeneration {
    pub const INITIAL: Self = Self(NonZeroU64::MIN);

    pub const fn get(self) -> u64 {
        self.0.get()
    }

    pub(super) fn checked_next(self) -> Option<Self> {
        self.get()
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .map(Self)
    }

    #[cfg(test)]
    pub(super) fn from_raw_for_test(raw: u64) -> Self {
        Self(NonZeroU64::new(raw).expect("test catalog generation must be non-zero"))
    }
}

impl Default for PluginCatalogGeneration {
    fn default() -> Self {
        Self::INITIAL
    }
}

impl fmt::Display for PluginCatalogGeneration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(formatter)
    }
}

#[cfg(test)]
#[path = "tests/generation.rs"]
mod tests;
