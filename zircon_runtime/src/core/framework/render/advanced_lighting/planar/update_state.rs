use std::collections::BTreeSet;

use super::{PlanarReflectionProbeData, PlanarUpdateMode};

/// Tracks successful captures without coupling the framework contract to GPU resources.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlanarReflectionUpdateState {
    captured: BTreeSet<u64>,
    dirty: BTreeSet<u64>,
}

impl PlanarReflectionUpdateState {
    pub fn should_capture(&self, probe: &PlanarReflectionProbeData) -> bool {
        probe.update == PlanarUpdateMode::EveryFrame
            || !self.captured.contains(&probe.probe_id)
            || self.dirty.contains(&probe.probe_id)
    }

    pub fn mark_dirty(&mut self, probe_id: u64) {
        self.dirty.insert(probe_id);
    }

    /// Call only after the capture and filter chain completed successfully.
    pub fn mark_captured(&mut self, probe_id: u64) {
        self.captured.insert(probe_id);
        self.dirty.remove(&probe_id);
    }

    pub fn forget(&mut self, probe_id: u64) {
        self.captured.remove(&probe_id);
        self.dirty.remove(&probe_id);
    }
}
