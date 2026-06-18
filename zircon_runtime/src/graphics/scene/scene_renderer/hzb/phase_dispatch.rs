use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshIndirectDrawExecution;

use super::HZB_OCCLUSION_CULL_WORKGROUP_SIZE;

#[derive(Clone, Copy)]
pub(crate) struct HzbOcclusionPhaseDispatch<'a> {
    execution: &'a MeshIndirectDrawExecution,
    args_count: u32,
    dispatch_group_count: u32,
}

impl<'a> HzbOcclusionPhaseDispatch<'a> {
    pub(crate) fn new(execution: &'a MeshIndirectDrawExecution) -> Option<Self> {
        let args_count = execution.args_count();
        let dispatch_group_count = dispatch_group_count(args_count);
        (dispatch_group_count > 0).then_some(Self {
            execution,
            args_count,
            dispatch_group_count,
        })
    }

    pub(crate) const fn execution(&self) -> &'a MeshIndirectDrawExecution {
        self.execution
    }

    pub(crate) const fn args_count(&self) -> u32 {
        self.args_count
    }

    pub(crate) const fn dispatch_group_count(&self) -> u32 {
        self.dispatch_group_count
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HzbOcclusionPhaseDispatchSummary {
    dispatched_phase_count: u32,
    dispatch_group_count: u32,
}

impl HzbOcclusionPhaseDispatchSummary {
    pub(crate) fn record_phase(&mut self, phase: &HzbOcclusionPhaseDispatch<'_>) {
        self.record_dispatch_group_count(phase.dispatch_group_count());
    }

    fn record_dispatch_group_count(&mut self, dispatch_group_count: u32) {
        self.dispatched_phase_count = self.dispatched_phase_count.saturating_add(1);
        self.dispatch_group_count = self
            .dispatch_group_count
            .saturating_add(dispatch_group_count);
    }

    pub(crate) const fn dispatched_phase_count(&self) -> u32 {
        self.dispatched_phase_count
    }

    pub(crate) const fn dispatch_group_count(&self) -> u32 {
        self.dispatch_group_count
    }
}

pub(crate) fn dispatch_group_count(args_count: u32) -> u32 {
    if args_count == 0 {
        0
    } else {
        args_count.div_ceil(HZB_OCCLUSION_CULL_WORKGROUP_SIZE[0])
    }
}

#[cfg(test)]
pub(crate) fn dispatch_group_count_for_phase_arg_counts(
    arg_counts: impl IntoIterator<Item = u32>,
) -> u32 {
    arg_counts.into_iter().fold(0u32, |groups, args_count| {
        groups.saturating_add(dispatch_group_count(args_count))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hzb_occlusion_dispatch_groups_cover_indirect_args() {
        assert_eq!(dispatch_group_count(0), 0);
        assert_eq!(dispatch_group_count(1), 1);
        assert_eq!(dispatch_group_count(64), 1);
        assert_eq!(dispatch_group_count(65), 2);
    }

    #[test]
    fn hzb_occlusion_dispatch_groups_sum_phase_local_workloads() {
        assert_eq!(dispatch_group_count(3), 1);
        assert_eq!(dispatch_group_count_for_phase_arg_counts([1, 1, 1]), 3);
        assert_eq!(dispatch_group_count_for_phase_arg_counts([64, 65, 0]), 3);
        assert_eq!(
            dispatch_group_count_for_phase_arg_counts([u32::MAX]),
            u32::MAX.div_ceil(64)
        );
    }

    #[test]
    fn hzb_occlusion_dispatch_summary_saturates_phase_and_group_counts() {
        let mut summary = HzbOcclusionPhaseDispatchSummary::default();

        summary.record_dispatch_group_count(u32::MAX);
        summary.record_dispatch_group_count(1);

        assert_eq!(summary.dispatched_phase_count(), 2);
        assert_eq!(summary.dispatch_group_count(), u32::MAX);
    }
}
