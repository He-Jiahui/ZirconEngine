use super::VirtualGeometryRuntimeStats;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VirtualGeometryRuntimeUpdate {
    stats: VirtualGeometryRuntimeStats,
}

impl VirtualGeometryRuntimeUpdate {
    pub fn new(stats: VirtualGeometryRuntimeStats) -> Self {
        Self { stats }
    }

    pub fn stats(&self) -> &VirtualGeometryRuntimeStats {
        &self.stats
    }
}
