use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshIndirectDrawExecution;

use super::HZB_OCCLUSION_CULL_WORKGROUP_SIZE;

#[derive(Clone, Copy)]
/// 将每个间接绘制阶段的有效参数数目映射为 HZB cull dispatch；阶段为空时跳过 GPU 工作。
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
#[path = "tests/phase_dispatch.rs"]
mod tests;
