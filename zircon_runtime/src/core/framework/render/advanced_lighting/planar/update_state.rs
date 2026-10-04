use std::collections::HashMap;

use super::{PlanarReflectionProbeData, PlanarUpdateMode};

/// Tracks successful captures without coupling the framework contract to GPU resources.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlanarReflectionUpdateState {
    // RUNTIME133_PLANAR_PROBE_SINGLE_HASH_STATE_BENCH_V1
    states: HashMap<u64, PlanarProbeCaptureState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlanarProbeCaptureState {
    Captured,
    Dirty,
}

impl PlanarReflectionUpdateState {
    /// 提交前查询按需捕获资格；只有整条相机捕获链成功后才能调用 mark_captured。
    /// 内容或目标变化时由拥有者 mark_dirty，卸载探针时 forget。
    pub fn should_capture(&self, probe: &PlanarReflectionProbeData) -> bool {
        probe.update == PlanarUpdateMode::EveryFrame
            || !matches!(
                self.states.get(&probe.probe_id),
                Some(PlanarProbeCaptureState::Captured)
            )
    }

    pub fn mark_dirty(&mut self, probe_id: u64) {
        self.states.insert(probe_id, PlanarProbeCaptureState::Dirty);
    }

    /// Call only after the capture and filter chain completed successfully.
    pub fn mark_captured(&mut self, probe_id: u64) {
        self.states
            .insert(probe_id, PlanarProbeCaptureState::Captured);
    }

    pub fn forget(&mut self, probe_id: u64) {
        self.states.remove(&probe_id);
    }
}

#[cfg(test)]
#[path = "update_state/tests/hash_state_tests.rs"]
mod hash_state_tests;
