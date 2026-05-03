use super::HybridGiRuntimeStats;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HybridGiRuntimeUpdate {
    stats: HybridGiRuntimeStats,
}

impl HybridGiRuntimeUpdate {
    pub fn new(stats: HybridGiRuntimeStats) -> Self {
        Self { stats }
    }

    pub fn stats(&self) -> HybridGiRuntimeStats {
        self.stats
    }
}
