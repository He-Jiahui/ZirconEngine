use crate::render_graph::{
    RenderGraphComputeDispatchExtent, RenderGraphComputeWorkload, RenderGraphPassResourceAccess,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphComputeDispatchRecord {
    pub pass_name: String,
    pub executor_id: String,
    pub pipeline_label: String,
    pub workgroup_size: [u32; 3],
    pub dispatch_groups: [u32; 3],
    /// False when the count is generated on GPU and consumed by an indirect
    /// dispatch that is intentionally not read back into the frame CPU path.
    pub dispatch_groups_known: bool,
    pub uploaded_bytes: u64,
    pub storage_write_resources: Vec<String>,
    pub resource_accesses: Vec<RenderGraphPassResourceAccess>,
}

impl RenderGraphComputeDispatchRecord {
    pub fn new(
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        pipeline_label: impl Into<String>,
        workgroup_size: [u32; 3],
        dispatch_groups: [u32; 3],
        storage_write_resources: Vec<String>,
    ) -> Self {
        Self {
            pass_name: pass_name.into(),
            executor_id: executor_id.into(),
            pipeline_label: pipeline_label.into(),
            workgroup_size,
            dispatch_groups,
            dispatch_groups_known: true,
            uploaded_bytes: 0,
            storage_write_resources,
            resource_accesses: Vec::new(),
        }
    }

    pub fn with_resource_accesses(
        mut self,
        resource_accesses: Vec<RenderGraphPassResourceAccess>,
    ) -> Self {
        self.resource_accesses = resource_accesses;
        self
    }

    pub fn with_uploaded_bytes(mut self, uploaded_bytes: u64) -> Self {
        self.uploaded_bytes = uploaded_bytes;
        self
    }

    pub fn with_gpu_indirect_dispatch_groups(mut self) -> Self {
        self.dispatch_groups_known = false;
        self
    }

    pub fn dispatch_group_volume(&self) -> usize {
        self.dispatch_groups.iter().fold(1_usize, |volume, groups| {
            volume.saturating_mul(*groups as usize)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderGraphComputeWorkloadDispatchContext {
    pub viewport_size: [u32; 2],
    pub cluster_grid_size: [u32; 2],
    pub froxel_grid_size: [u32; 3],
    pub hzb_furthest_size: [u32; 2],
    pub indirect_args_count: u32,
    pub indirect_args_dispatch_group_count: Option<u32>,
}

impl RenderGraphComputeWorkloadDispatchContext {
    pub fn new(
        viewport_size: [u32; 2],
        cluster_grid_size: [u32; 2],
        hzb_furthest_size: [u32; 2],
        indirect_args_count: u32,
    ) -> Self {
        Self {
            viewport_size: [viewport_size[0].max(1), viewport_size[1].max(1)],
            cluster_grid_size: [cluster_grid_size[0].max(1), cluster_grid_size[1].max(1)],
            froxel_grid_size: [1, 1, 1],
            hzb_furthest_size: [hzb_furthest_size[0].max(1), hzb_furthest_size[1].max(1)],
            indirect_args_count,
            indirect_args_dispatch_group_count: None,
        }
    }

    pub fn with_indirect_args_dispatch_group_count(mut self, dispatch_group_count: u32) -> Self {
        self.indirect_args_dispatch_group_count = Some(dispatch_group_count);
        self
    }

    pub fn with_froxel_grid_size(mut self, froxel_grid_size: [u32; 3]) -> Self {
        self.froxel_grid_size = froxel_grid_size.map(|extent| extent.max(1));
        self
    }

    fn expected_dispatch_groups(self, workload: &RenderGraphComputeWorkload) -> [u32; 3] {
        match &workload.dispatch_extent {
            RenderGraphComputeDispatchExtent::Viewport => {
                dispatch_groups_for_2d_extent(self.viewport_size, workload.workgroup_size)
            }
            RenderGraphComputeDispatchExtent::ClusterGrid => {
                dispatch_groups_for_2d_extent(self.cluster_grid_size, workload.workgroup_size)
            }
            RenderGraphComputeDispatchExtent::FroxelGrid => {
                dispatch_groups_for_3d_extent(self.froxel_grid_size, workload.workgroup_size)
            }
            RenderGraphComputeDispatchExtent::FroxelGridXy => dispatch_groups_for_2d_extent(
                [self.froxel_grid_size[0], self.froxel_grid_size[1]],
                workload.workgroup_size,
            ),
            RenderGraphComputeDispatchExtent::HzbFurthest => {
                dispatch_groups_for_2d_extent(self.hzb_furthest_size, workload.workgroup_size)
            }
            RenderGraphComputeDispatchExtent::IndirectArgs => {
                if let Some(dispatch_group_count) = self.indirect_args_dispatch_group_count {
                    return dispatch_groups_for_1d_group_count(
                        dispatch_group_count,
                        workload.workgroup_size,
                    );
                }
                dispatch_groups_for_1d_extent(self.indirect_args_count, workload.workgroup_size)
            }
            RenderGraphComputeDispatchExtent::Fixed(groups) => *groups,
        }
    }
}

fn dispatch_groups_for_1d_group_count(group_count: u32, workgroup_size: [u32; 3]) -> [u32; 3] {
    [
        group_count,
        dispatch_group_count(1, workgroup_size[1]),
        dispatch_group_count(1, workgroup_size[2]),
    ]
}

fn dispatch_groups_for_1d_extent(extent: u32, workgroup_size: [u32; 3]) -> [u32; 3] {
    [
        dispatch_group_count_allow_zero(extent, workgroup_size[0]),
        dispatch_group_count(1, workgroup_size[1]),
        dispatch_group_count(1, workgroup_size[2]),
    ]
}

fn dispatch_groups_for_2d_extent(extent: [u32; 2], workgroup_size: [u32; 3]) -> [u32; 3] {
    [
        dispatch_group_count(extent[0], workgroup_size[0]),
        dispatch_group_count(extent[1], workgroup_size[1]),
        dispatch_group_count(1, workgroup_size[2]),
    ]
}

fn dispatch_groups_for_3d_extent(extent: [u32; 3], workgroup_size: [u32; 3]) -> [u32; 3] {
    [
        dispatch_group_count(extent[0], workgroup_size[0]),
        dispatch_group_count(extent[1], workgroup_size[1]),
        dispatch_group_count(extent[2], workgroup_size[2]),
    ]
}

fn dispatch_group_count(extent: u32, workgroup_size: u32) -> u32 {
    extent.max(1).div_ceil(workgroup_size.max(1))
}

fn dispatch_group_count_allow_zero(extent: u32, workgroup_size: u32) -> u32 {
    if extent == 0 {
        0
    } else {
        extent.div_ceil(workgroup_size.max(1))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderGraphComputeWorkloadAuditStatus {
    Matched,
    MissingDispatch,
    UnexpectedDispatch,
    PipelineLabelMismatch,
    WorkgroupSizeMismatch,
    DispatchExtentMismatch,
}

impl RenderGraphComputeWorkloadAuditStatus {
    pub const fn is_mismatch(self) -> bool {
        matches!(
            self,
            Self::PipelineLabelMismatch
                | Self::WorkgroupSizeMismatch
                | Self::DispatchExtentMismatch
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphComputeWorkloadAuditRecord {
    pub pass_name: String,
    pub executor_id: String,
    pub planned_pipeline_label: Option<String>,
    pub actual_pipeline_label: Option<String>,
    pub planned_workgroup_size: Option<[u32; 3]>,
    pub actual_workgroup_size: Option<[u32; 3]>,
    pub planned_dispatch_groups: Option<[u32; 3]>,
    pub actual_dispatch_groups: Option<[u32; 3]>,
    pub status: RenderGraphComputeWorkloadAuditStatus,
}

impl RenderGraphComputeWorkloadAuditRecord {
    pub(super) fn matched_or_mismatched(
        pass_name: &str,
        executor_id: &str,
        planned: &RenderGraphComputeWorkload,
        dispatch_context: RenderGraphComputeWorkloadDispatchContext,
        actual: &RenderGraphComputeDispatchRecord,
    ) -> Self {
        let planned_dispatch_groups = dispatch_context.expected_dispatch_groups(planned);
        let status = if actual.pipeline_label != planned.pipeline_label {
            RenderGraphComputeWorkloadAuditStatus::PipelineLabelMismatch
        } else if actual.workgroup_size != planned.workgroup_size {
            RenderGraphComputeWorkloadAuditStatus::WorkgroupSizeMismatch
        } else if actual.dispatch_groups_known && actual.dispatch_groups != planned_dispatch_groups
        {
            RenderGraphComputeWorkloadAuditStatus::DispatchExtentMismatch
        } else {
            RenderGraphComputeWorkloadAuditStatus::Matched
        };
        Self {
            pass_name: pass_name.to_string(),
            executor_id: executor_id.to_string(),
            planned_pipeline_label: Some(planned.pipeline_label.clone()),
            actual_pipeline_label: Some(actual.pipeline_label.clone()),
            planned_workgroup_size: Some(planned.workgroup_size),
            actual_workgroup_size: Some(actual.workgroup_size),
            planned_dispatch_groups: Some(planned_dispatch_groups),
            actual_dispatch_groups: actual
                .dispatch_groups_known
                .then_some(actual.dispatch_groups),
            status,
        }
    }

    pub(super) fn missing_dispatch(
        pass_name: &str,
        executor_id: &str,
        planned: &RenderGraphComputeWorkload,
        dispatch_context: RenderGraphComputeWorkloadDispatchContext,
    ) -> Self {
        Self {
            pass_name: pass_name.to_string(),
            executor_id: executor_id.to_string(),
            planned_pipeline_label: Some(planned.pipeline_label.clone()),
            actual_pipeline_label: None,
            planned_workgroup_size: Some(planned.workgroup_size),
            actual_workgroup_size: None,
            planned_dispatch_groups: Some(dispatch_context.expected_dispatch_groups(planned)),
            actual_dispatch_groups: None,
            status: RenderGraphComputeWorkloadAuditStatus::MissingDispatch,
        }
    }

    pub(super) fn unexpected_dispatch(actual: &RenderGraphComputeDispatchRecord) -> Self {
        Self {
            pass_name: actual.pass_name.clone(),
            executor_id: actual.executor_id.clone(),
            planned_pipeline_label: None,
            actual_pipeline_label: Some(actual.pipeline_label.clone()),
            planned_workgroup_size: None,
            actual_workgroup_size: Some(actual.workgroup_size),
            planned_dispatch_groups: None,
            actual_dispatch_groups: actual
                .dispatch_groups_known
                .then_some(actual.dispatch_groups),
            status: RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch,
        }
    }
}

#[cfg(test)]
mod tests;
