use crate::core::framework::render::PostProcessPassGraph;
use crate::core::framework::render::{
    MotionVectorCameraStatus, RenderBudgetKey, RenderColorLutReadbackReport,
    RenderExposureReadbackReport, RenderGraphExecutionAliasReport,
    RenderGraphExecutionProfileReport, RenderGraphExecutionResourceReport,
    RenderGraphMaterializationReport, RenderGraphParallelRecordingReport,
    RenderGraphPassProfileMetrics, RenderGraphPassProfileRecord, RenderGraphStageExecutionReport,
    RenderHistoryCopyReport, RenderSceneVelocityReadbackReport,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::scene_renderer::lighting::light_grid_builder::LightGridStats;
use crate::graphics::visibility::HzbOcclusionCullReport;
use crate::render_graph::{
    QueueLane, RenderGraphComputeWorkload, RenderGraphPassResourceAccess,
    RenderGraphResourceAccessKind, RenderPassId,
};

mod compute_workload;
#[cfg(test)]
mod tests;

pub use self::compute_workload::{
    RenderGraphComputeDispatchRecord, RenderGraphComputeWorkloadAuditRecord,
    RenderGraphComputeWorkloadAuditStatus, RenderGraphComputeWorkloadDispatchContext,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphLightGridReport {
    pub light_count: usize,
    pub tile_count: usize,
    pub zbin_count: usize,
    pub non_empty_tile_count: usize,
    pub non_empty_zbin_count: usize,
    pub non_empty_cluster_count: usize,
    pub peak_lights_per_cluster: usize,
    pub average_lights_per_cluster_milli: usize,
}

impl RenderGraphLightGridReport {
    pub(crate) fn from_stats(stats: &LightGridStats) -> Self {
        Self {
            light_count: stats.light_count as usize,
            tile_count: stats.tile_count as usize,
            zbin_count: stats.zbin_count as usize,
            non_empty_tile_count: stats.non_empty_tile_count as usize,
            non_empty_zbin_count: stats.non_empty_zbin_count as usize,
            non_empty_cluster_count: stats.non_empty_cluster_count as usize,
            peak_lights_per_cluster: stats.peak_lights_per_cluster as usize,
            average_lights_per_cluster_milli: (stats.average_lights_per_cluster * 1000.0)
                .round()
                .max(0.0) as usize,
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
    materialization_report: RenderGraphMaterializationReport,
    resource_alias_report: RenderGraphExecutionAliasReport,
    pass_profile_records: Vec<RenderGraphPassProfileRecord>,
    parallel_recording_report: RenderGraphParallelRecordingReport,
    history_copy_report: RenderHistoryCopyReport,
    scene_velocity_readback_report: RenderSceneVelocityReadbackReport,
    #[cfg(test)]
    scene_velocity_readback_rg16_float_bytes: Option<Vec<u8>>,
    exposure_readback_report: RenderExposureReadbackReport,
    color_lut_readback_report: RenderColorLutReadbackReport,
    hzb_occlusion_cull_report: Option<HzbOcclusionCullReport>,
    light_grid_report: Option<RenderGraphLightGridReport>,
    taa_reactive_mask_encoded_pass_count: usize,
    taa_reactive_mask_encoded_write_bytes: u64,
    taa_resolve_bind_group_create_count: usize,
}

impl RenderGraphExecutionRecord {
    #[cfg(test)]
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

    #[cfg(test)]
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

    #[cfg(test)]
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

    pub fn set_materialization_report(&mut self, report: RenderGraphMaterializationReport) {
        self.materialization_report = report;
    }

    pub fn set_resource_alias_report(&mut self, report: RenderGraphExecutionAliasReport) {
        self.resource_alias_report = report;
    }

    pub fn push_pass_profile(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        cpu_elapsed_micros: u64,
    ) {
        self.push_pass_profile_with_budget_key(
            pass_name,
            executor_id,
            RenderBudgetKey::Other,
            cpu_elapsed_micros,
        );
    }

    pub fn push_pass_profile_with_budget_key(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        budget_key: RenderBudgetKey,
        cpu_elapsed_micros: u64,
    ) {
        self.push_pass_profile_with_budget_key_and_compute_dispatches(
            pass_name,
            executor_id,
            budget_key,
            cpu_elapsed_micros,
            RenderGraphPassProfileMetrics::default(),
            &[],
        );
    }

    pub fn push_pass_profile_with_budget_key_and_compute_dispatches(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        budget_key: RenderBudgetKey,
        cpu_elapsed_micros: u64,
        render_metrics: RenderGraphPassProfileMetrics,
        compute_dispatches: &[RenderGraphComputeDispatchRecord],
    ) {
        let (dispatch_count, upload_bytes) = compute_dispatches.iter().fold(
            (0_u32, 0_u64),
            |(dispatch_count, upload_bytes), dispatch| {
                (
                    dispatch_count.saturating_add(1),
                    upload_bytes.saturating_add(dispatch.uploaded_bytes),
                )
            },
        );
        self.pass_profile_records.push(
            RenderGraphPassProfileRecord::new(pass_name, executor_id, cpu_elapsed_micros)
                .with_budget_key(budget_key)
                .with_render_metrics(render_metrics)
                .with_compute_metrics(dispatch_count, upload_bytes),
        );
    }

    pub fn record_parallel_recording_eligibility(&mut self, bucket_count: usize) {
        self.parallel_recording_report.eligible_stage_count = self
            .parallel_recording_report
            .eligible_stage_count
            .saturating_add(1);
        self.parallel_recording_report.eligible_bucket_count = self
            .parallel_recording_report
            .eligible_bucket_count
            .saturating_add(bucket_count);
    }

    pub fn record_parallel_recording_execution(&mut self, bucket_count: usize) {
        self.parallel_recording_report.executed_stage_count = self
            .parallel_recording_report
            .executed_stage_count
            .saturating_add(1);
        self.parallel_recording_report.executed_bucket_count = self
            .parallel_recording_report
            .executed_bucket_count
            .saturating_add(bucket_count);
    }

    pub fn set_history_copy_report(&mut self, report: RenderHistoryCopyReport) {
        self.history_copy_report = report;
    }

    #[cfg(test)]
    pub fn set_scene_velocity_readback_report(
        &mut self,
        report: RenderSceneVelocityReadbackReport,
    ) {
        self.scene_velocity_readback_report = report;
    }

    #[cfg(test)]
    pub fn set_scene_velocity_readback_rg16_float_bytes(&mut self, bytes: Vec<u8>) {
        self.scene_velocity_readback_rg16_float_bytes = Some(bytes);
    }

    #[cfg(test)]
    pub fn set_exposure_readback_report(&mut self, report: RenderExposureReadbackReport) {
        self.exposure_readback_report = report;
    }

    #[cfg(test)]
    pub fn set_color_lut_readback_report(&mut self, report: RenderColorLutReadbackReport) {
        self.color_lut_readback_report = report;
    }

    pub fn set_hzb_occlusion_cull_report(&mut self, report: HzbOcclusionCullReport) {
        self.hzb_occlusion_cull_report = Some(report);
    }

    pub fn set_light_grid_report(&mut self, report: RenderGraphLightGridReport) {
        self.light_grid_report = Some(report);
    }

    pub fn add_taa_reactive_mask_encoding(&mut self, pass_count: usize, write_bytes: u64) {
        self.taa_reactive_mask_encoded_pass_count = self
            .taa_reactive_mask_encoded_pass_count
            .saturating_add(pass_count);
        self.taa_reactive_mask_encoded_write_bytes = self
            .taa_reactive_mask_encoded_write_bytes
            .saturating_add(write_bytes);
    }

    pub fn add_taa_resolve_bind_group_create_count(&mut self, count: usize) {
        self.taa_resolve_bind_group_create_count = self
            .taa_resolve_bind_group_create_count
            .saturating_add(count);
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
        if let Some(planned) = planned_workload {
            let first_matching_dispatch_index = dispatches.iter().position(|dispatch| {
                dispatch.pass_name == pass_name && dispatch.executor_id == executor_id
            });
            if let Some(index) = first_matching_dispatch_index {
                self.compute_workload_audit.push(
                    RenderGraphComputeWorkloadAuditRecord::matched_or_mismatched(
                        pass_name,
                        executor_id,
                        planned,
                        dispatch_context,
                        &dispatches[index],
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
            for unexpected in dispatches
                .iter()
                .enumerate()
                .filter_map(|(index, dispatch)| {
                    (Some(index) != first_matching_dispatch_index
                        && dispatch.pass_name == pass_name
                        && dispatch.executor_id == executor_id)
                        .then_some(dispatch)
                })
            {
                self.compute_workload_audit.push(
                    RenderGraphComputeWorkloadAuditRecord::unexpected_dispatch(unexpected),
                );
            }
            for unexpected in dispatches.iter().filter(|dispatch| {
                dispatch.pass_name != pass_name || dispatch.executor_id != executor_id
            }) {
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

    pub fn materialization_report(&self) -> RenderGraphMaterializationReport {
        self.materialization_report
    }

    pub fn resource_alias_report(&self) -> &RenderGraphExecutionAliasReport {
        &self.resource_alias_report
    }

    pub fn profile_report(&self) -> RenderGraphExecutionProfileReport {
        RenderGraphExecutionProfileReport::new(self.pass_profile_records.clone())
    }

    pub fn parallel_recording_report(&self) -> RenderGraphParallelRecordingReport {
        self.parallel_recording_report
    }

    pub fn stage_execution_report(&self) -> RenderGraphStageExecutionReport {
        let mut seen_stages = [false; RenderPassStage::ALL.len()];
        let mut staged_pass_count = 0;
        let mut unstaged_pass_count = 0;
        let mut unique_stage_count = 0;
        let mut stage_transition_count = 0;
        let mut stage_order_violation_count = 0;
        let mut previous_stage = None;

        for executed_stage in &self.executed_pass_stages {
            if let Some(stage) = executed_stage {
                staged_pass_count += 1;
                let stage_index = stage.index();
                if !seen_stages[stage_index] {
                    seen_stages[stage_index] = true;
                    unique_stage_count += 1;
                }
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
            unique_stage_count,
            stage_transition_count,
            stage_order_violation_count,
        )
    }

    pub fn history_copy_report(&self) -> RenderHistoryCopyReport {
        self.history_copy_report
    }

    pub fn scene_velocity_readback_report(&self) -> RenderSceneVelocityReadbackReport {
        self.scene_velocity_readback_report
    }

    #[cfg(test)]
    pub fn scene_velocity_readback_rg16_float_bytes(&self) -> Option<&[u8]> {
        self.scene_velocity_readback_rg16_float_bytes.as_deref()
    }

    pub fn exposure_readback_report(&self) -> RenderExposureReadbackReport {
        self.exposure_readback_report
    }

    pub fn color_lut_readback_report(&self) -> RenderColorLutReadbackReport {
        self.color_lut_readback_report
    }

    pub fn hzb_occlusion_cull_report(&self) -> Option<HzbOcclusionCullReport> {
        self.hzb_occlusion_cull_report
    }

    pub fn light_grid_report(&self) -> Option<RenderGraphLightGridReport> {
        self.light_grid_report
    }

    pub fn taa_reactive_mask_encoding(&self) -> (usize, u64) {
        (
            self.taa_reactive_mask_encoded_pass_count,
            self.taa_reactive_mask_encoded_write_bytes,
        )
    }

    pub fn taa_resolve_bind_group_create_count(&self) -> usize {
        self.taa_resolve_bind_group_create_count
    }

    #[cfg(test)]
    pub fn executed_pass_stages(&self) -> &[Option<RenderPassStage>] {
        &self.executed_pass_stages
    }

    #[cfg(test)]
    pub fn executed_pass_resources(&self) -> &[Vec<RenderGraphPassResourceAccess>] {
        &self.executed_pass_resources
    }

    #[cfg(test)]
    pub fn compute_dispatches(&self) -> &[RenderGraphComputeDispatchRecord] {
        &self.compute_dispatches
    }

    #[cfg(test)]
    pub fn compute_workload_audit(&self) -> &[RenderGraphComputeWorkloadAuditRecord] {
        &self.compute_workload_audit
    }

    #[cfg(test)]
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

    pub fn compute_dispatch_count_for_executor_prefix(&self, executor_prefix: &str) -> usize {
        self.compute_dispatches_for_executor_prefix(executor_prefix)
            .count()
    }

    pub fn compute_dispatch_group_volume_total_for_executor_prefix(
        &self,
        executor_prefix: &str,
    ) -> usize {
        self.compute_dispatches_for_executor_prefix(executor_prefix)
            .map(RenderGraphComputeDispatchRecord::dispatch_group_volume)
            .fold(0, usize::saturating_add)
    }

    pub fn compute_uploaded_bytes_total_for_executor_prefix(&self, executor_prefix: &str) -> u64 {
        self.compute_dispatches_for_executor_prefix(executor_prefix)
            .map(|dispatch| dispatch.uploaded_bytes)
            .fold(0, u64::saturating_add)
    }

    pub fn compute_storage_write_resource_count(&self) -> usize {
        self.compute_dispatches
            .iter()
            .map(|dispatch| dispatch.storage_write_resources.len())
            .sum()
    }

    fn compute_dispatches_for_executor_prefix<'a>(
        &'a self,
        executor_prefix: &'a str,
    ) -> impl Iterator<Item = &'a RenderGraphComputeDispatchRecord> {
        self.compute_dispatches
            .iter()
            .filter(move |dispatch| dispatch.executor_id.starts_with(executor_prefix))
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

    pub fn executed_stage_count(&self, stage: RenderPassStage) -> usize {
        self.executed_pass_stages
            .iter()
            .filter(|executed_stage| **executed_stage == Some(stage))
            .count()
    }
}
