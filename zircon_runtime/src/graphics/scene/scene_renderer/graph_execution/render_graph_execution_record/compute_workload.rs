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
            hzb_furthest_size: [hzb_furthest_size[0].max(1), hzb_furthest_size[1].max(1)],
            indirect_args_count,
            indirect_args_dispatch_group_count: None,
        }
    }

    pub fn with_indirect_args_dispatch_group_count(mut self, dispatch_group_count: u32) -> Self {
        self.indirect_args_dispatch_group_count = Some(dispatch_group_count);
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
        } else if actual.dispatch_groups != planned_dispatch_groups {
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
            actual_dispatch_groups: Some(actual.dispatch_groups),
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
            actual_dispatch_groups: Some(actual.dispatch_groups),
            status: RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::render_graph::{
        RenderGraphComputeDispatchExtent, RenderGraphComputeWorkload,
        RenderGraphPassResourceAccess, RenderGraphResourceAccessKind, RenderGraphResourceKind,
    };

    use super::super::RenderGraphExecutionRecord;
    use super::{
        RenderGraphComputeDispatchRecord, RenderGraphComputeWorkloadAuditStatus,
        RenderGraphComputeWorkloadDispatchContext,
    };

    fn dispatch_context() -> RenderGraphComputeWorkloadDispatchContext {
        RenderGraphComputeWorkloadDispatchContext::new([320, 240], [40, 30], [1024, 1024], 130)
    }

    #[test]
    fn execution_record_tracks_compute_dispatch_metadata() {
        let mut record = RenderGraphExecutionRecord::default();

        record.push_compute_dispatch(RenderGraphComputeDispatchRecord::new(
            "ssao-evaluate",
            "ao.ssao-evaluate",
            "zircon-ssao-pipeline",
            [8, 8, 1],
            [40, 30, 1],
            vec!["ambient-occlusion".to_string()],
        ));
        record.push_compute_dispatch(
            RenderGraphComputeDispatchRecord::new(
                "light-grid-build",
                "lighting.light-grid",
                "zircon-cluster-pipeline",
                [8, 8, 1],
                [5, 4, 1],
                vec!["light-list".to_string()],
            )
            .with_resource_accesses(vec![RenderGraphPassResourceAccess {
                name: "light-list".to_string(),
                kind: RenderGraphResourceKind::TransientBuffer,
                access: RenderGraphResourceAccessKind::Write,
                attachment_ops: None,
            }]),
        );

        assert_eq!(record.compute_dispatch_count(), 2);
        assert_eq!(record.compute_dispatch_group_volume_total(), 1220);
        assert_eq!(record.compute_storage_write_resource_count(), 2);
        assert_eq!(
            record.compute_dispatches()[0].storage_write_resources,
            ["ambient-occlusion".to_string()]
        );
        assert_eq!(record.compute_dispatches()[1].resource_accesses.len(), 1);
        assert_eq!(
            record.compute_dispatches()[1].resource_accesses[0].access,
            RenderGraphResourceAccessKind::Write
        );
    }

    #[test]
    fn execution_record_audits_planned_compute_workloads_against_dispatches() {
        let mut record = RenderGraphExecutionRecord::default();
        let planned = RenderGraphComputeWorkload::new(
            "zircon-ssao-pipeline",
            [8, 8, 1],
            RenderGraphComputeDispatchExtent::Viewport,
        );
        let matched = RenderGraphComputeDispatchRecord::new(
            "ssao-evaluate",
            "ao.ssao-evaluate",
            "zircon-ssao-pipeline",
            [8, 8, 1],
            [40, 30, 1],
            vec!["ambient-occlusion".to_string()],
        );
        let unexpected = RenderGraphComputeDispatchRecord::new(
            "legacy-compute",
            "legacy.executor",
            "legacy-pipeline",
            [4, 4, 1],
            [1, 1, 1],
            Vec::new(),
        );

        record.audit_compute_workload(
            "ssao-evaluate",
            "ao.ssao-evaluate",
            Some(&planned),
            dispatch_context(),
            std::slice::from_ref(&matched),
        );
        record.audit_compute_workload(
            "compute-fixed",
            "compute.fixed",
            Some(&RenderGraphComputeWorkload::fixed(
                "fixed-pipeline",
                [4, 4, 1],
                [2, 3, 1],
            )),
            dispatch_context(),
            &[RenderGraphComputeDispatchRecord::new(
                "compute-fixed",
                "compute.fixed",
                "fixed-pipeline",
                [4, 4, 1],
                [2, 3, 1],
                Vec::new(),
            )],
        );
        record.audit_compute_workload(
            "hzb-build",
            "visibility.hzb-build",
            Some(&RenderGraphComputeWorkload::hzb_furthest(
                "zircon-hzb-build-pipeline",
                [8, 8, 1],
            )),
            dispatch_context(),
            &[RenderGraphComputeDispatchRecord::new(
                "hzb-build",
                "visibility.hzb-build",
                "zircon-hzb-build-pipeline",
                [8, 8, 1],
                [128, 128, 1],
                vec!["hzb-furthest".to_string()],
            )],
        );
        record.audit_compute_workload(
            "hzb-occlusion-cull",
            "visibility.hzb-occlusion-cull",
            Some(&RenderGraphComputeWorkload::indirect_args(
                "zircon-hzb-occlusion-cull-pipeline",
                [64, 1, 1],
            )),
            dispatch_context(),
            &[RenderGraphComputeDispatchRecord::new(
                "hzb-occlusion-cull",
                "visibility.hzb-occlusion-cull",
                "zircon-hzb-occlusion-cull-pipeline",
                [64, 1, 1],
                [3, 1, 1],
                vec!["mesh.indirect-args".to_string()],
            )],
        );
        record.audit_compute_workload(
            "light-grid-build",
            "lighting.light-grid",
            Some(&RenderGraphComputeWorkload::new(
                "zircon-cluster-pipeline",
                [8, 8, 1],
                RenderGraphComputeDispatchExtent::ClusterGrid,
            )),
            dispatch_context(),
            &[],
        );
        record.audit_compute_workload(
            "legacy-compute",
            "legacy.executor",
            None,
            dispatch_context(),
            &[unexpected],
        );

        assert_eq!(record.compute_workload_planned_count(), 5);
        assert_eq!(record.compute_workload_matched_count(), 4);
        assert_eq!(record.compute_workload_missing_dispatch_count(), 1);
        assert_eq!(record.compute_workload_unexpected_dispatch_count(), 1);
        assert_eq!(record.compute_workload_mismatch_count(), 0);
        assert_eq!(
            record.compute_workload_audit()[0].status,
            RenderGraphComputeWorkloadAuditStatus::Matched
        );
        assert_eq!(
            record.compute_workload_audit()[1].status,
            RenderGraphComputeWorkloadAuditStatus::Matched
        );
        assert_eq!(
            record.compute_workload_audit()[2].status,
            RenderGraphComputeWorkloadAuditStatus::Matched
        );
        assert_eq!(
            record.compute_workload_audit()[3].status,
            RenderGraphComputeWorkloadAuditStatus::Matched
        );
        assert_eq!(
            record.compute_workload_audit()[4].status,
            RenderGraphComputeWorkloadAuditStatus::MissingDispatch
        );
        assert_eq!(
            record.compute_workload_audit()[5].status,
            RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch
        );
        assert_eq!(
            record.compute_workload_audit()[0].planned_dispatch_groups,
            Some([40, 30, 1])
        );
        assert_eq!(
            record.compute_workload_audit()[1].planned_dispatch_groups,
            Some([2, 3, 1])
        );
        assert_eq!(
            record.compute_workload_audit()[2].planned_dispatch_groups,
            Some([128, 128, 1])
        );
        assert_eq!(
            record.compute_workload_audit()[3].planned_dispatch_groups,
            Some([3, 1, 1])
        );
        assert_eq!(
            record.compute_workload_audit()[4].planned_dispatch_groups,
            Some([5, 4, 1])
        );
        assert_eq!(
            record.compute_workload_audit()[5].actual_dispatch_groups,
            Some([1, 1, 1])
        );
    }

    #[test]
    fn execution_record_audits_zero_indirect_arg_workload_as_zero_groups() {
        let mut record = RenderGraphExecutionRecord::default();
        let context =
            RenderGraphComputeWorkloadDispatchContext::new([320, 240], [40, 30], [1024, 1024], 0);

        record.audit_compute_workload(
            "hzb-occlusion-cull",
            "visibility.hzb-occlusion-cull",
            Some(&RenderGraphComputeWorkload::indirect_args(
                "zircon-hzb-occlusion-cull-pipeline",
                [64, 1, 1],
            )),
            context,
            &[RenderGraphComputeDispatchRecord::new(
                "hzb-occlusion-cull",
                "visibility.hzb-occlusion-cull",
                "zircon-hzb-occlusion-cull-pipeline",
                [64, 1, 1],
                [0, 1, 1],
                Vec::new(),
            )],
        );

        assert_eq!(
            record.compute_workload_audit()[0].planned_dispatch_groups,
            Some([0, 1, 1])
        );
        assert_eq!(
            record.compute_workload_audit()[0].status,
            RenderGraphComputeWorkloadAuditStatus::Matched
        );
    }

    #[test]
    fn execution_record_audits_phase_local_indirect_arg_workload_groups() {
        let mut record = RenderGraphExecutionRecord::default();
        let context =
            RenderGraphComputeWorkloadDispatchContext::new([320, 240], [40, 30], [1024, 1024], 3)
                .with_indirect_args_dispatch_group_count(3);

        record.audit_compute_workload(
            "hzb-occlusion-cull",
            "visibility.hzb-occlusion-cull",
            Some(&RenderGraphComputeWorkload::indirect_args(
                "zircon-hzb-occlusion-cull-pipeline",
                [64, 1, 1],
            )),
            context,
            &[RenderGraphComputeDispatchRecord::new(
                "hzb-occlusion-cull",
                "visibility.hzb-occlusion-cull",
                "zircon-hzb-occlusion-cull-pipeline",
                [64, 1, 1],
                [3, 1, 1],
                Vec::new(),
            )],
        );

        assert_eq!(
            record.compute_workload_audit()[0].planned_dispatch_groups,
            Some([3, 1, 1])
        );
        assert_eq!(
            record.compute_workload_audit()[0].status,
            RenderGraphComputeWorkloadAuditStatus::Matched
        );
    }

    #[test]
    fn execution_record_flags_compute_workload_label_workgroup_and_extent_mismatches() {
        let mut record = RenderGraphExecutionRecord::default();
        let planned = RenderGraphComputeWorkload::new(
            "zircon-ssao-pipeline",
            [8, 8, 1],
            RenderGraphComputeDispatchExtent::Viewport,
        );
        let wrong_label = RenderGraphComputeDispatchRecord::new(
            "ssao-evaluate",
            "ao.ssao-evaluate",
            "other-pipeline",
            [8, 8, 1],
            [40, 30, 1],
            Vec::new(),
        );
        let wrong_workgroup = RenderGraphComputeDispatchRecord::new(
            "ssao-evaluate-2",
            "ao.ssao-evaluate",
            "zircon-ssao-pipeline",
            [16, 8, 1],
            [40, 30, 1],
            Vec::new(),
        );
        let wrong_extent = RenderGraphComputeDispatchRecord::new(
            "ssao-evaluate-3",
            "ao.ssao-evaluate",
            "zircon-ssao-pipeline",
            [8, 8, 1],
            [39, 30, 1],
            Vec::new(),
        );

        record.audit_compute_workload(
            "ssao-evaluate",
            "ao.ssao-evaluate",
            Some(&planned),
            dispatch_context(),
            &[wrong_label],
        );
        record.audit_compute_workload(
            "ssao-evaluate-2",
            "ao.ssao-evaluate",
            Some(&planned),
            dispatch_context(),
            &[wrong_workgroup],
        );
        record.audit_compute_workload(
            "ssao-evaluate-3",
            "ao.ssao-evaluate",
            Some(&planned),
            dispatch_context(),
            &[wrong_extent],
        );

        assert_eq!(record.compute_workload_mismatch_count(), 3);
        assert_eq!(
            record.compute_workload_audit()[0].status,
            RenderGraphComputeWorkloadAuditStatus::PipelineLabelMismatch
        );
        assert_eq!(
            record.compute_workload_audit()[1].status,
            RenderGraphComputeWorkloadAuditStatus::WorkgroupSizeMismatch
        );
        assert_eq!(
            record.compute_workload_audit()[2].status,
            RenderGraphComputeWorkloadAuditStatus::DispatchExtentMismatch
        );
        assert_eq!(
            record.compute_workload_audit()[2].planned_dispatch_groups,
            Some([40, 30, 1])
        );
        assert_eq!(
            record.compute_workload_audit()[2].actual_dispatch_groups,
            Some([39, 30, 1])
        );
    }
}
