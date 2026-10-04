use std::collections::BTreeMap;

use super::{ResolutionContext, ShellRegionId};

#[derive(Clone, Copy)]
/// 物理偏好映射的借用查询视图；本轮按需转为logical，避免复制整张映射并保留缺值回退。
pub(crate) struct LogicalRegionPreferredExtents<'a> {
    physical: Option<&'a BTreeMap<ShellRegionId, f32>>,
    resolution: ResolutionContext,
}

impl<'a> LogicalRegionPreferredExtents<'a> {
    // EDITOR77_REGION_PREFERRED_ZERO_ALLOCATION_LOOKUP_BENCH_V1
    pub(crate) const fn new(
        physical: Option<&'a BTreeMap<ShellRegionId, f32>>,
        resolution: ResolutionContext,
    ) -> Self {
        Self {
            physical,
            resolution,
        }
    }

    /// 与本轮壳缩放上下文一致的单region查询；缺key保持空值以允许下一层偏好。
    pub(crate) fn get(self, region: ShellRegionId) -> Option<f32> {
        self.physical
            .and_then(|extents| extents.get(&region).copied())
            .map(|extent| self.resolution.to_logical(extent))
    }
}

#[cfg(test)]
#[path = "region_preferred_extents/tests/allocation_tests.rs"]
mod allocation_tests;
