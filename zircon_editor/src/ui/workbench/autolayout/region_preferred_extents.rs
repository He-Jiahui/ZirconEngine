use std::collections::BTreeMap;

use super::{ResolutionContext, ShellRegionId};

#[derive(Clone, Copy)]
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

    pub(crate) fn get(self, region: ShellRegionId) -> Option<f32> {
        self.physical
            .and_then(|extents| extents.get(&region).copied())
            .map(|extent| self.resolution.to_logical(extent))
    }
}

#[cfg(test)]
#[path = "region_preferred_extents/allocation_tests.rs"]
mod allocation_tests;
