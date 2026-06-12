use std::collections::BTreeSet;

use crate::core::framework::render::PostProcessPassGraph;
use crate::core::framework::render::{
    MotionVectorCameraStatus, RenderGraphExecutionResourceReport, RenderGraphStageExecutionReport,
    RenderHistoryCopyReport,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{
    QueueLane, RenderGraphComputeDispatchExtent, RenderGraphComputeWorkload,
    RenderGraphPassResourceAccess, RenderGraphResourceAccessKind, RenderPassId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphComputeDispatchRecord {
    pub pass_name: String,
    pub executor_id: String,
    pub pipeline_label: String,
    pub workgroup_size: [u32; 3],
    pub dispatch_groups: [u32; 3],
    pub storage_write_resources: Vec<String>,
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
        }
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
}

impl RenderGraphComputeWorkloadDispatchContext {
    pub fn new(
        viewport_size: [u32; 2],
        cluster_grid_size: [u32; 2],
        hzb_furthest_size: [u32; 2],
    ) -> Self {
        Self {
            viewport_size: [viewport_size[0].max(1), viewport_size[1].max(1)],
            cluster_grid_size: [cluster_grid_size[0].max(1), cluster_grid_size[1].max(1)],
            hzb_furthest_size: [hzb_furthest_size[0].max(1), hzb_furthest_size[1].max(1)],
        }
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
            RenderGraphComputeDispatchExtent::Fixed(groups) => *groups,
        }
    }
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
    fn matched_or_mismatched(
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

    fn missing_dispatch(
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

    fn unexpected_dispatch(actual: &RenderGraphComputeDispatchRecord) -> Self {
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphExecutionRecord {
    executed_passes: Vec<String>,
    executed_executor_ids: Vec<String>,
    executed_debug_markers: Vec<String>,
    executed_queue_lanes: Vec<QueueLane>,
    executed_declared_queue_lanes: Vec<QueueLane>,
    executed_pass_stages: Vec<Option<RenderPassStage>>,
    executed_pass_dependencies: Vec<Vec<RenderPassId>>,
    executed_pass_resources: Vec<Vec<RenderGraphPassResourceAccess>>,
    compute_dispatches: Vec<RenderGraphComputeDispatchRecord>,
    compute_workload_audit: Vec<RenderGraphComputeWorkloadAuditRecord>,
    post_process_graph: Option<PostProcessPassGraph>,
    executed_post_process_nodes: Vec<String>,
    motion_vector_camera_status: MotionVectorCameraStatus,
    resource_report: RenderGraphExecutionResourceReport,
    history_copy_report: RenderHistoryCopyReport,
}

impl RenderGraphExecutionRecord {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn push_executed_pass(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        queue: QueueLane,
    ) {
        self.push_executed_pass_with_resources(pass_name, executor_id, queue, Vec::new());
    }

    pub fn push_executed_pass_with_resources(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        queue: QueueLane,
        resources: Vec<RenderGraphPassResourceAccess>,
    ) {
        self.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
            None,
            pass_name,
            executor_id,
            queue,
            queue,
            Vec::new(),
            resources,
        );
    }

    #[allow(dead_code)]
    pub fn push_executed_pass_with_declared_queue_and_resources(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        queue: QueueLane,
        declared_queue: QueueLane,
        resources: Vec<RenderGraphPassResourceAccess>,
    ) {
        self.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
            None,
            pass_name,
            executor_id,
            queue,
            declared_queue,
            Vec::new(),
            resources,
        );
    }

    #[allow(dead_code)]
    pub fn push_executed_pass_with_declared_queue_dependencies_and_resources(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        queue: QueueLane,
        declared_queue: QueueLane,
        dependencies: Vec<RenderPassId>,
        resources: Vec<RenderGraphPassResourceAccess>,
    ) {
        self.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
            None,
            pass_name,
            executor_id,
            queue,
            declared_queue,
            dependencies,
            resources,
        );
    }

    pub fn push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
        &mut self,
        stage: Option<RenderPassStage>,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        queue: QueueLane,
        declared_queue: QueueLane,
        dependencies: Vec<RenderPassId>,
        resources: Vec<RenderGraphPassResourceAccess>,
    ) {
        self.push_executed_pass_with_stage_declared_queue_dependencies_resources_and_debug_marker(
            stage,
            pass_name,
            executor_id,
            queue,
            declared_queue,
            dependencies,
            resources,
            None,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn push_executed_pass_with_stage_declared_queue_dependencies_resources_and_debug_marker(
        &mut self,
        stage: Option<RenderPassStage>,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        queue: QueueLane,
        declared_queue: QueueLane,
        dependencies: Vec<RenderPassId>,
        resources: Vec<RenderGraphPassResourceAccess>,
        debug_marker: Option<String>,
    ) {
        self.executed_passes.push(pass_name.into());
        self.executed_executor_ids.push(executor_id.into());
        self.executed_debug_markers
            .push(debug_marker.unwrap_or_default());
        self.executed_queue_lanes.push(queue);
        self.executed_declared_queue_lanes.push(declared_queue);
        self.executed_pass_stages.push(stage);
        self.executed_pass_dependencies.push(dependencies);
        self.executed_pass_resources.push(resources);
    }

    pub fn push_executed_post_process_node(&mut self, node_name: impl Into<String>) {
        self.executed_post_process_nodes.push(node_name.into());
    }

    pub fn set_motion_vector_camera_status(&mut self, status: MotionVectorCameraStatus) {
        self.motion_vector_camera_status = status;
    }

    pub fn set_resource_report(&mut self, report: RenderGraphExecutionResourceReport) {
        self.resource_report = report;
    }

    pub fn set_history_copy_report(&mut self, report: RenderHistoryCopyReport) {
        self.history_copy_report = report;
    }

    pub fn push_compute_dispatch(&mut self, dispatch: RenderGraphComputeDispatchRecord) {
        self.compute_dispatches.push(dispatch);
    }

    pub fn audit_compute_workload(
        &mut self,
        pass_name: &str,
        executor_id: &str,
        planned_workload: Option<&RenderGraphComputeWorkload>,
        dispatch_context: RenderGraphComputeWorkloadDispatchContext,
        dispatches: &[RenderGraphComputeDispatchRecord],
    ) {
        let (matching_dispatches, unexpected_dispatches): (Vec<_>, Vec<_>) =
            dispatches.iter().partition(|dispatch| {
                dispatch.pass_name == pass_name && dispatch.executor_id == executor_id
            });

        if let Some(planned) = planned_workload {
            if let Some(actual) = matching_dispatches.first() {
                self.compute_workload_audit.push(
                    RenderGraphComputeWorkloadAuditRecord::matched_or_mismatched(
                        pass_name,
                        executor_id,
                        planned,
                        dispatch_context,
                        actual,
                    ),
                );
            } else {
                self.compute_workload_audit.push(
                    RenderGraphComputeWorkloadAuditRecord::missing_dispatch(
                        pass_name,
                        executor_id,
                        planned,
                        dispatch_context,
                    ),
                );
            }
            for unexpected in matching_dispatches
                .into_iter()
                .skip(1)
                .chain(unexpected_dispatches)
            {
                self.compute_workload_audit.push(
                    RenderGraphComputeWorkloadAuditRecord::unexpected_dispatch(unexpected),
                );
            }
            return;
        }

        for unexpected in dispatches {
            self.compute_workload_audit.push(
                RenderGraphComputeWorkloadAuditRecord::unexpected_dispatch(unexpected),
            );
        }
    }

    pub fn set_post_process_graph(&mut self, graph: PostProcessPassGraph) {
        self.post_process_graph = Some(graph);
    }

    pub fn post_process_graph(&self) -> Option<&PostProcessPassGraph> {
        self.post_process_graph.as_ref()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn executed_post_process_nodes(&self) -> &[String] {
        &self.executed_post_process_nodes
    }

    pub fn executed_passes(&self) -> &[String] {
        &self.executed_passes
    }

    pub fn executed_executor_ids(&self) -> &[String] {
        &self.executed_executor_ids
    }

    pub fn executed_debug_markers(&self) -> &[String] {
        &self.executed_debug_markers
    }

    pub fn motion_vector_camera_status(&self) -> MotionVectorCameraStatus {
        self.motion_vector_camera_status
    }

    pub fn resource_report(&self) -> RenderGraphExecutionResourceReport {
        self.resource_report
    }

    pub fn stage_execution_report(&self) -> RenderGraphStageExecutionReport {
        let mut unique_stages = BTreeSet::new();
        let mut staged_pass_count = 0;
        let mut unstaged_pass_count = 0;
        let mut stage_transition_count = 0;
        let mut stage_order_violation_count = 0;
        let mut previous_stage = None;

        for executed_stage in &self.executed_pass_stages {
            if let Some(stage) = executed_stage {
                staged_pass_count += 1;
                unique_stages.insert(*stage);
                if let Some(previous) = previous_stage {
                    if previous != *stage {
                        stage_transition_count += 1;
                    }
                    if previous > *stage {
                        stage_order_violation_count += 1;
                    }
                }
                previous_stage = Some(*stage);
            } else {
                unstaged_pass_count += 1;
                previous_stage = None;
            }
        }

        RenderGraphStageExecutionReport::new(
            staged_pass_count,
            unstaged_pass_count,
            unique_stages.len(),
            stage_transition_count,
            stage_order_violation_count,
        )
    }

    pub fn history_copy_report(&self) -> RenderHistoryCopyReport {
        self.history_copy_report
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn executed_pass_stages(&self) -> &[Option<RenderPassStage>] {
        &self.executed_pass_stages
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn executed_pass_resources(&self) -> &[Vec<RenderGraphPassResourceAccess>] {
        &self.executed_pass_resources
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn compute_dispatches(&self) -> &[RenderGraphComputeDispatchRecord] {
        &self.compute_dispatches
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn compute_workload_audit(&self) -> &[RenderGraphComputeWorkloadAuditRecord] {
        &self.compute_workload_audit
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn executed_pass_dependencies(&self) -> &[Vec<RenderPassId>] {
        &self.executed_pass_dependencies
    }

    pub fn executed_resource_access_count(&self) -> usize {
        self.executed_pass_resources.iter().map(Vec::len).sum()
    }

    pub fn executed_resource_access_count_for(
        &self,
        resource_name: &str,
        access: RenderGraphResourceAccessKind,
    ) -> usize {
        self.executed_pass_resources
            .iter()
            .flatten()
            .filter(|resource| resource.name == resource_name && resource.access == access)
            .count()
    }

    pub fn executed_dependency_count(&self) -> usize {
        self.executed_pass_dependencies.iter().map(Vec::len).sum()
    }

    pub fn compute_dispatch_count(&self) -> usize {
        self.compute_dispatches.len()
    }

    pub fn compute_dispatch_group_volume_total(&self) -> usize {
        self.compute_dispatches
            .iter()
            .map(RenderGraphComputeDispatchRecord::dispatch_group_volume)
            .sum()
    }

    pub fn compute_storage_write_resource_count(&self) -> usize {
        self.compute_dispatches
            .iter()
            .map(|dispatch| dispatch.storage_write_resources.len())
            .sum()
    }

    pub fn compute_workload_planned_count(&self) -> usize {
        self.compute_workload_audit
            .iter()
            .filter(|record| record.planned_pipeline_label.is_some())
            .count()
    }

    pub fn compute_workload_matched_count(&self) -> usize {
        self.compute_workload_audit
            .iter()
            .filter(|record| record.status == RenderGraphComputeWorkloadAuditStatus::Matched)
            .count()
    }

    pub fn compute_workload_missing_dispatch_count(&self) -> usize {
        self.compute_workload_audit
            .iter()
            .filter(|record| {
                record.status == RenderGraphComputeWorkloadAuditStatus::MissingDispatch
            })
            .count()
    }

    pub fn compute_workload_mismatch_count(&self) -> usize {
        self.compute_workload_audit
            .iter()
            .filter(|record| record.status.is_mismatch())
            .count()
    }

    pub fn compute_workload_unexpected_dispatch_count(&self) -> usize {
        self.compute_workload_audit
            .iter()
            .filter(|record| {
                record.status == RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch
            })
            .count()
    }

    pub fn executed_queue_fallback_count(&self) -> usize {
        self.executed_queue_lanes
            .iter()
            .zip(&self.executed_declared_queue_lanes)
            .filter(|(queue, declared_queue)| queue != declared_queue)
            .count()
    }

    pub fn executed_queue_lane_count(&self, queue: QueueLane) -> usize {
        self.executed_queue_lanes
            .iter()
            .filter(|executed_queue| **executed_queue == queue)
            .count()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn executed_stage_count(&self, stage: RenderPassStage) -> usize {
        self.executed_pass_stages
            .iter()
            .filter(|executed_stage| **executed_stage == Some(stage))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderGraphExecutionResourceReport, RenderGraphStageExecutionReport,
        RenderHistoryCopyReport,
    };
    use crate::core::math::UVec2;
    use crate::graphics::pipeline::RenderPassStage;
    use crate::render_graph::RenderPassId;
    use crate::render_graph::{
        QueueLane, RenderGraphComputeDispatchExtent, RenderGraphComputeWorkload,
        RenderGraphPassResourceAccess, RenderGraphResourceAccessKind, RenderGraphResourceKind,
    };

    use super::{
        RenderGraphComputeDispatchRecord, RenderGraphComputeWorkloadAuditStatus,
        RenderGraphComputeWorkloadDispatchContext, RenderGraphExecutionRecord,
    };

    fn dispatch_context() -> RenderGraphComputeWorkloadDispatchContext {
        RenderGraphComputeWorkloadDispatchContext::new([320, 240], [40, 30], [1024, 1024])
    }

    #[test]
    fn execution_record_preserves_resource_binding_report() {
        let mut record = RenderGraphExecutionRecord::default();
        let report = RenderGraphExecutionResourceReport::new(6, 4, 2, 3);

        record.set_resource_report(report);

        assert_eq!(record.resource_report(), report);
    }

    #[test]
    fn execution_record_preserves_history_copy_report() {
        let mut record = RenderGraphExecutionRecord::default();
        let report = RenderHistoryCopyReport::new(
            true,
            UVec2::new(640, 360),
            4,
            true,
            true,
            true,
            false,
            false,
        );

        record.set_history_copy_report(report);

        assert_eq!(record.history_copy_report(), report);
        assert!(record.history_copy_report().debug_marker_emitted);
    }

    #[test]
    fn execution_record_counts_queue_lanes_from_executed_passes() {
        let mut record = RenderGraphExecutionRecord::default();

        record.push_executed_pass_with_declared_queue_and_resources(
            "cull",
            "virtual-geometry.node-cluster-cull",
            QueueLane::Graphics,
            QueueLane::AsyncCompute,
            Vec::new(),
        );
        record.push_executed_pass("main", "mesh.opaque", QueueLane::Graphics);

        assert_eq!(record.executed_queue_lane_count(QueueLane::AsyncCompute), 0);
        assert_eq!(record.executed_queue_lane_count(QueueLane::Graphics), 2);
        assert_eq!(record.executed_queue_lane_count(QueueLane::AsyncCopy), 0);
        assert_eq!(record.executed_queue_fallback_count(), 1);
    }

    #[test]
    fn execution_record_preserves_executed_pass_resource_accesses() {
        let mut record = RenderGraphExecutionRecord::default();
        let resources = vec![
            RenderGraphPassResourceAccess {
                name: "scene-depth".to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Read,
                attachment_ops: None,
            },
            RenderGraphPassResourceAccess {
                name: "scene-color".to_string(),
                kind: RenderGraphResourceKind::TransientTexture,
                access: RenderGraphResourceAccessKind::Write,
                attachment_ops: None,
            },
        ];

        record.push_executed_pass_with_resources(
            "opaque",
            "mesh.opaque",
            QueueLane::Graphics,
            resources.clone(),
        );

        assert_eq!(record.executed_pass_resources(), &[resources]);
        assert_eq!(record.executed_resource_access_count(), 2);
    }

    #[test]
    fn execution_record_preserves_executed_pass_dependencies() {
        let mut record = RenderGraphExecutionRecord::default();
        let dependencies = vec![RenderPassId(2), RenderPassId(5)];

        record.push_executed_pass_with_declared_queue_dependencies_and_resources(
            "lighting",
            "lighting.clustered-cull",
            QueueLane::Graphics,
            QueueLane::Graphics,
            dependencies.clone(),
            Vec::new(),
        );

        assert_eq!(record.executed_pass_dependencies(), &[dependencies]);
        assert_eq!(record.executed_dependency_count(), 2);
    }

    #[test]
    fn execution_record_keeps_post_process_nodes_out_of_render_graph_passes() {
        let mut record = RenderGraphExecutionRecord::default();

        record.push_executed_pass("overlay-gizmo", "overlay.gizmo", QueueLane::Graphics);
        record.push_executed_post_process_node("final-composite");

        assert_eq!(record.executed_passes(), &["overlay-gizmo".to_string()]);
        assert_eq!(
            record.executed_post_process_nodes(),
            &["final-composite".to_string()]
        );
        assert_eq!(record.executed_queue_lane_count(QueueLane::Graphics), 1);
    }

    #[test]
    fn execution_record_preserves_renderer_stage_metadata() {
        let mut record = RenderGraphExecutionRecord::default();

        record.push_executed_pass("legacy-overlay", "overlay.legacy", QueueLane::Graphics);
        record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
            Some(RenderPassStage::Transparent3d),
            "particle-render",
            "particle.transparent",
            QueueLane::Graphics,
            QueueLane::Graphics,
            Vec::new(),
            Vec::new(),
        );
        record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
            Some(RenderPassStage::Transparent3d),
            "transparent-mesh",
            "mesh.transparent",
            QueueLane::Graphics,
            QueueLane::Graphics,
            Vec::new(),
            Vec::new(),
        );
        record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
            Some(RenderPassStage::PostProcess),
            "post-stack",
            "post.stack",
            QueueLane::Graphics,
            QueueLane::Graphics,
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(
            record.executed_pass_stages(),
            &[
                None,
                Some(RenderPassStage::Transparent3d),
                Some(RenderPassStage::Transparent3d),
                Some(RenderPassStage::PostProcess),
            ]
        );
        assert_eq!(
            record.executed_stage_count(RenderPassStage::Transparent3d),
            2
        );
        assert_eq!(record.executed_stage_count(RenderPassStage::PostProcess), 1);
        assert_eq!(
            record.stage_execution_report(),
            RenderGraphStageExecutionReport::new(3, 1, 2, 1, 0)
        );
    }

    #[test]
    fn execution_record_counts_named_resource_accesses() {
        let mut record = RenderGraphExecutionRecord::default();
        let shadow_write = RenderGraphPassResourceAccess {
            name: "shadow-map".to_string(),
            kind: crate::render_graph::RenderGraphResourceKind::TransientTexture,
            access: RenderGraphResourceAccessKind::Write,
            attachment_ops: None,
        };
        let shadow_read = RenderGraphPassResourceAccess {
            name: "shadow-map".to_string(),
            kind: crate::render_graph::RenderGraphResourceKind::TransientTexture,
            access: RenderGraphResourceAccessKind::Read,
            attachment_ops: None,
        };
        let scene_color_read = RenderGraphPassResourceAccess {
            name: "scene-color".to_string(),
            kind: crate::render_graph::RenderGraphResourceKind::External,
            access: RenderGraphResourceAccessKind::Read,
            attachment_ops: None,
        };

        record.push_executed_pass_with_resources(
            "shadow-map",
            "shadow.map",
            QueueLane::Graphics,
            vec![shadow_write],
        );
        record.push_executed_pass_with_resources(
            "opaque-mesh",
            "mesh.opaque",
            QueueLane::Graphics,
            vec![shadow_read, scene_color_read],
        );

        assert_eq!(
            record.executed_resource_access_count_for(
                "shadow-map",
                RenderGraphResourceAccessKind::Write,
            ),
            1
        );
        assert_eq!(
            record.executed_resource_access_count_for(
                "shadow-map",
                RenderGraphResourceAccessKind::Read
            ),
            1
        );
        assert_eq!(
            record.executed_resource_access_count_for(
                "scene-color",
                RenderGraphResourceAccessKind::Write,
            ),
            0
        );
    }

    #[test]
    fn execution_record_counts_renderer_stage_order_violations() {
        let mut record = RenderGraphExecutionRecord::default();

        record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
            Some(RenderPassStage::PostProcess),
            "post-stack",
            "post.stack",
            QueueLane::Graphics,
            QueueLane::Graphics,
            Vec::new(),
            Vec::new(),
        );
        record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
            Some(RenderPassStage::Opaque3d),
            "late-opaque",
            "mesh.opaque",
            QueueLane::Graphics,
            QueueLane::Graphics,
            Vec::new(),
            Vec::new(),
        );
        record.push_executed_pass("legacy-gap", "legacy.gap", QueueLane::Graphics);
        record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
            Some(RenderPassStage::Shadow),
            "shadow-map",
            "shadow.map",
            QueueLane::Graphics,
            QueueLane::Graphics,
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(
            record.stage_execution_report(),
            RenderGraphStageExecutionReport::new(3, 1, 3, 1, 1)
        );
    }

    #[test]
    fn execution_record_preserves_pass_debug_markers() {
        let mut record = RenderGraphExecutionRecord::default();

        record
            .push_executed_pass_with_stage_declared_queue_dependencies_resources_and_debug_marker(
                Some(RenderPassStage::PostProcess),
                "clustered-lighting",
                "lighting.clustered-cull",
                QueueLane::Graphics,
                QueueLane::AsyncCompute,
                Vec::new(),
                Vec::new(),
                Some("zircon::RenderGraphPass::clustered-lighting".to_string()),
            );

        assert_eq!(
            record.executed_debug_markers(),
            &["zircon::RenderGraphPass::clustered-lighting".to_string()]
        );
        assert_eq!(record.executed_queue_fallback_count(), 1);
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
        record.push_compute_dispatch(RenderGraphComputeDispatchRecord::new(
            "clustered-light-culling",
            "lighting.clustered-cull",
            "zircon-cluster-pipeline",
            [8, 8, 1],
            [5, 4, 1],
            vec!["light-list".to_string()],
        ));

        assert_eq!(record.compute_dispatch_count(), 2);
        assert_eq!(record.compute_dispatch_group_volume_total(), 1220);
        assert_eq!(record.compute_storage_write_resource_count(), 2);
        assert_eq!(
            record.compute_dispatches()[0].storage_write_resources,
            ["ambient-occlusion".to_string()]
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
            "clustered-light-culling",
            "lighting.clustered-cull",
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

        assert_eq!(record.compute_workload_planned_count(), 4);
        assert_eq!(record.compute_workload_matched_count(), 3);
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
            RenderGraphComputeWorkloadAuditStatus::MissingDispatch
        );
        assert_eq!(
            record.compute_workload_audit()[4].status,
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
            Some([5, 4, 1])
        );
        assert_eq!(
            record.compute_workload_audit()[4].actual_dispatch_groups,
            Some([1, 1, 1])
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
