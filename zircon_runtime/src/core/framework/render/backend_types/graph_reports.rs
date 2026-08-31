use std::collections::BTreeSet;

use crate::core::framework::render::{RenderBudgetKey, RenderPassNativeResourceCreateMetrics};
use crate::core::math::UVec2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphTransientPoolReport {
    pub frame_index: u64,
    pub texture_created_count: usize,
    pub texture_reused_count: usize,
    /// Graph-owned resources held until a cross-frame extraction completes.
    pub persistent_texture_request_count: usize,
    pub persistent_texture_requested_bytes: u64,
    pub persistent_texture_created_count: usize,
    pub persistent_texture_reused_count: usize,
    pub buffer_created_count: usize,
    pub buffer_reused_count: usize,
    pub texture_pool_entry_count: usize,
    pub buffer_pool_entry_count: usize,
    pub texture_pool_retained_bytes: u64,
    pub buffer_pool_retained_bytes: u64,
    /// Submitted graph textures awaiting their exact GPU completion ticket.
    pub pending_retire_texture_count: usize,
    pub pending_retire_texture_bytes: u64,
    /// Submitted graph buffers awaiting their exact GPU completion ticket.
    pub pending_retire_buffer_count: usize,
    pub pending_retire_buffer_bytes: u64,
    pub completion_reclaimed_texture_count: usize,
    pub completion_reclaimed_buffer_count: usize,
    pub completion_discarded_texture_count: usize,
    pub completion_discarded_buffer_count: usize,
    /// Submission status lookups issued while collecting pending textures.
    pub completion_texture_status_query_count: usize,
    /// Submission status lookups issued while collecting pending buffers.
    pub completion_buffer_status_query_count: usize,
    /// Backings dropped because their WGPU device epoch was retired.
    pub device_epoch_discarded_texture_count: usize,
    /// Backings dropped because their WGPU device epoch was retired.
    pub device_epoch_discarded_buffer_count: usize,
    pub texture_pool_budget_bytes: u64,
    pub buffer_pool_budget_bytes: u64,
    pub evicted_texture_count: usize,
    pub evicted_buffer_count: usize,
    pub budget_evicted_texture_count: usize,
    pub budget_evicted_buffer_count: usize,
    /// Free allocations visited by frame-age eviction.
    pub stale_texture_scan_count: usize,
    pub stale_buffer_scan_count: usize,
    /// Free allocations visited while computing the retained-byte budget.
    pub budget_texture_accounted_count: usize,
    pub budget_buffer_accounted_count: usize,
    /// LRU candidates materialized and sorted only when a pool exceeds budget.
    pub budget_texture_sort_candidate_count: usize,
    pub budget_buffer_sort_candidate_count: usize,
}

impl RenderGraphTransientPoolReport {
    pub const fn new(
        frame_index: u64,
        texture_created_count: usize,
        texture_reused_count: usize,
        buffer_created_count: usize,
        buffer_reused_count: usize,
        texture_pool_entry_count: usize,
        buffer_pool_entry_count: usize,
        evicted_texture_count: usize,
        evicted_buffer_count: usize,
    ) -> Self {
        Self {
            frame_index,
            texture_created_count,
            texture_reused_count,
            persistent_texture_request_count: 0,
            persistent_texture_requested_bytes: 0,
            persistent_texture_created_count: 0,
            persistent_texture_reused_count: 0,
            buffer_created_count,
            buffer_reused_count,
            texture_pool_entry_count,
            buffer_pool_entry_count,
            texture_pool_retained_bytes: 0,
            buffer_pool_retained_bytes: 0,
            pending_retire_texture_count: 0,
            pending_retire_texture_bytes: 0,
            pending_retire_buffer_count: 0,
            pending_retire_buffer_bytes: 0,
            completion_reclaimed_texture_count: 0,
            completion_reclaimed_buffer_count: 0,
            completion_discarded_texture_count: 0,
            completion_discarded_buffer_count: 0,
            completion_texture_status_query_count: 0,
            completion_buffer_status_query_count: 0,
            device_epoch_discarded_texture_count: 0,
            device_epoch_discarded_buffer_count: 0,
            texture_pool_budget_bytes: 0,
            buffer_pool_budget_bytes: 0,
            evicted_texture_count,
            evicted_buffer_count,
            budget_evicted_texture_count: 0,
            budget_evicted_buffer_count: 0,
            stale_texture_scan_count: 0,
            stale_buffer_scan_count: 0,
            budget_texture_accounted_count: 0,
            budget_buffer_accounted_count: 0,
            budget_texture_sort_candidate_count: 0,
            budget_buffer_sort_candidate_count: 0,
        }
    }

    pub const fn with_retained_bytes(
        mut self,
        texture_pool_retained_bytes: u64,
        buffer_pool_retained_bytes: u64,
    ) -> Self {
        self.texture_pool_retained_bytes = texture_pool_retained_bytes;
        self.buffer_pool_retained_bytes = buffer_pool_retained_bytes;
        self
    }

    pub const fn with_budget_bytes(
        mut self,
        texture_pool_budget_bytes: u64,
        buffer_pool_budget_bytes: u64,
    ) -> Self {
        self.texture_pool_budget_bytes = texture_pool_budget_bytes;
        self.buffer_pool_budget_bytes = buffer_pool_budget_bytes;
        self
    }

    pub const fn with_budget_evictions(
        mut self,
        budget_evicted_texture_count: usize,
        budget_evicted_buffer_count: usize,
    ) -> Self {
        self.budget_evicted_texture_count = budget_evicted_texture_count;
        self.budget_evicted_buffer_count = budget_evicted_buffer_count;
        self
    }

    pub const fn with_persistent_extraction(
        mut self,
        persistent_texture_request_count: usize,
        persistent_texture_requested_bytes: u64,
        persistent_texture_created_count: usize,
        persistent_texture_reused_count: usize,
    ) -> Self {
        self.persistent_texture_request_count = persistent_texture_request_count;
        self.persistent_texture_requested_bytes = persistent_texture_requested_bytes;
        self.persistent_texture_created_count = persistent_texture_created_count;
        self.persistent_texture_reused_count = persistent_texture_reused_count;
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub const fn with_submission_retirement(
        mut self,
        pending_retire_texture_count: usize,
        pending_retire_texture_bytes: u64,
        pending_retire_buffer_count: usize,
        pending_retire_buffer_bytes: u64,
        completion_reclaimed_texture_count: usize,
        completion_reclaimed_buffer_count: usize,
        completion_discarded_texture_count: usize,
        completion_discarded_buffer_count: usize,
    ) -> Self {
        self.pending_retire_texture_count = pending_retire_texture_count;
        self.pending_retire_texture_bytes = pending_retire_texture_bytes;
        self.pending_retire_buffer_count = pending_retire_buffer_count;
        self.pending_retire_buffer_bytes = pending_retire_buffer_bytes;
        self.completion_reclaimed_texture_count = completion_reclaimed_texture_count;
        self.completion_reclaimed_buffer_count = completion_reclaimed_buffer_count;
        self.completion_discarded_texture_count = completion_discarded_texture_count;
        self.completion_discarded_buffer_count = completion_discarded_buffer_count;
        self
    }

    pub const fn with_device_epoch_discards(
        mut self,
        device_epoch_discarded_texture_count: usize,
        device_epoch_discarded_buffer_count: usize,
    ) -> Self {
        self.device_epoch_discarded_texture_count = device_epoch_discarded_texture_count;
        self.device_epoch_discarded_buffer_count = device_epoch_discarded_buffer_count;
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub const fn with_maintenance_work(
        mut self,
        completion_texture_status_query_count: usize,
        completion_buffer_status_query_count: usize,
        stale_texture_scan_count: usize,
        stale_buffer_scan_count: usize,
        budget_texture_accounted_count: usize,
        budget_buffer_accounted_count: usize,
        budget_texture_sort_candidate_count: usize,
        budget_buffer_sort_candidate_count: usize,
    ) -> Self {
        self.completion_texture_status_query_count = completion_texture_status_query_count;
        self.completion_buffer_status_query_count = completion_buffer_status_query_count;
        self.stale_texture_scan_count = stale_texture_scan_count;
        self.stale_buffer_scan_count = stale_buffer_scan_count;
        self.budget_texture_accounted_count = budget_texture_accounted_count;
        self.budget_buffer_accounted_count = budget_buffer_accounted_count;
        self.budget_texture_sort_candidate_count = budget_texture_sort_candidate_count;
        self.budget_buffer_sort_candidate_count = budget_buffer_sort_candidate_count;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphExecutionAccessBindingReport {
    pub transient_access_binding_count: usize,
    pub transient_texture_access_binding_count: usize,
    pub transient_buffer_access_binding_count: usize,
    pub unique_texture_view_count: usize,
    pub reused_texture_view_count: usize,
}

impl RenderGraphExecutionAccessBindingReport {
    pub const fn new(
        transient_access_binding_count: usize,
        transient_texture_access_binding_count: usize,
        transient_buffer_access_binding_count: usize,
        unique_texture_view_count: usize,
        reused_texture_view_count: usize,
    ) -> Self {
        Self {
            transient_access_binding_count,
            transient_texture_access_binding_count,
            transient_buffer_access_binding_count,
            unique_texture_view_count,
            reused_texture_view_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphExecutionResourceReport {
    pub texture_view_count: usize,
    pub external_texture_view_count: usize,
    pub owned_texture_count: usize,
    pub buffer_count: usize,
    pub total_bound_resource_count: usize,
    pub transient_pool_report: RenderGraphTransientPoolReport,
    pub access_binding_report: RenderGraphExecutionAccessBindingReport,
}

impl RenderGraphExecutionResourceReport {
    pub const fn new(
        texture_view_count: usize,
        external_texture_view_count: usize,
        owned_texture_count: usize,
        buffer_count: usize,
    ) -> Self {
        Self {
            texture_view_count,
            external_texture_view_count,
            owned_texture_count,
            buffer_count,
            total_bound_resource_count: texture_view_count + buffer_count,
            transient_pool_report: RenderGraphTransientPoolReport::new(0, 0, 0, 0, 0, 0, 0, 0, 0),
            access_binding_report: RenderGraphExecutionAccessBindingReport::new(0, 0, 0, 0, 0),
        }
    }

    pub const fn with_transient_pool_report(
        mut self,
        transient_pool_report: RenderGraphTransientPoolReport,
    ) -> Self {
        self.transient_pool_report = transient_pool_report;
        self
    }

    pub const fn with_access_binding_report(
        mut self,
        access_binding_report: RenderGraphExecutionAccessBindingReport,
    ) -> Self {
        self.access_binding_report = access_binding_report;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphMaterializationReport {
    pub required_texture_count: usize,
    pub bound_texture_count: usize,
    pub missing_texture_count: usize,
    pub required_buffer_count: usize,
    pub bound_buffer_count: usize,
    pub missing_buffer_count: usize,
    pub required_external_count: usize,
    pub bound_required_external_count: usize,
    pub missing_required_external_count: usize,
    pub report_only_external_count: usize,
    pub bound_report_only_external_count: usize,
    pub missing_report_only_external_count: usize,
    pub stale_texture_binding_count: usize,
    pub stale_buffer_binding_count: usize,
    pub sparse_texture_reservation_count: usize,
}

impl RenderGraphMaterializationReport {
    pub const fn required_resource_count(self) -> usize {
        self.required_texture_count + self.required_buffer_count + self.required_external_count
    }

    pub const fn bound_resource_count(self) -> usize {
        self.bound_texture_count + self.bound_buffer_count + self.bound_external_count()
    }

    pub const fn missing_resource_count(self) -> usize {
        self.missing_texture_count + self.missing_buffer_count + self.missing_external_count()
    }

    pub const fn missing_materialized_resource_count(self) -> usize {
        self.missing_texture_count + self.missing_buffer_count
    }

    pub const fn external_count(self) -> usize {
        self.required_external_count + self.report_only_external_count
    }

    pub const fn bound_external_count(self) -> usize {
        self.bound_required_external_count + self.bound_report_only_external_count
    }

    pub const fn missing_external_count(self) -> usize {
        self.missing_required_external_count + self.missing_report_only_external_count
    }

    pub const fn stale_binding_count(self) -> usize {
        self.stale_texture_binding_count + self.stale_buffer_binding_count
    }

    pub const fn materialized_resources_complete(self) -> bool {
        self.missing_materialized_resource_count() == 0 && self.stale_binding_count() == 0
    }

    pub const fn is_complete(self) -> bool {
        self.missing_resource_count() == 0 && self.stale_binding_count() == 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphExecutionAliasRecord {
    pub logical_name: String,
    pub backing_name: String,
}

impl RenderGraphExecutionAliasRecord {
    pub fn new(logical_name: impl Into<String>, backing_name: impl Into<String>) -> Self {
        Self {
            logical_name: logical_name.into(),
            backing_name: backing_name.into(),
        }
    }

    pub fn is_alias(&self) -> bool {
        self.logical_name != self.backing_name
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphExecutionAliasReport {
    pub texture_aliases: Vec<RenderGraphExecutionAliasRecord>,
    pub buffer_aliases: Vec<RenderGraphExecutionAliasRecord>,
}

impl RenderGraphExecutionAliasReport {
    pub fn new(
        texture_aliases: Vec<RenderGraphExecutionAliasRecord>,
        buffer_aliases: Vec<RenderGraphExecutionAliasRecord>,
    ) -> Self {
        Self {
            texture_aliases,
            buffer_aliases,
        }
    }

    pub fn texture_logical_count(&self) -> usize {
        self.texture_aliases.len()
    }

    pub fn texture_alias_count(&self) -> usize {
        self.texture_aliases
            .iter()
            .filter(|record| record.is_alias())
            .count()
    }

    pub fn texture_backing_count(&self) -> usize {
        backing_count(&self.texture_aliases)
    }

    pub fn buffer_logical_count(&self) -> usize {
        self.buffer_aliases.len()
    }

    pub fn buffer_alias_count(&self) -> usize {
        self.buffer_aliases
            .iter()
            .filter(|record| record.is_alias())
            .count()
    }

    pub fn buffer_backing_count(&self) -> usize {
        backing_count(&self.buffer_aliases)
    }
}

fn backing_count(records: &[RenderGraphExecutionAliasRecord]) -> usize {
    records
        .iter()
        .map(|record| record.backing_name.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphPassProfileMetrics {
    pub draw_count: u32,
    pub instance_count: u32,
    pub state_change_count: u32,
}

impl RenderGraphPassProfileMetrics {
    pub const fn new(draw_count: u32, instance_count: u32, state_change_count: u32) -> Self {
        Self {
            draw_count,
            instance_count,
            state_change_count,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphPassProfileRecord {
    pub pass_name: String,
    pub executor_id: String,
    pub budget_key: RenderBudgetKey,
    pub cpu_elapsed_micros: u64,
    pub draw_count: u32,
    pub instance_count: u32,
    pub state_change_count: u32,
    pub dispatch_count: u32,
    pub upload_bytes: u64,
    pub native_resource_creates: RenderPassNativeResourceCreateMetrics,
}

impl RenderGraphPassProfileRecord {
    pub fn new(
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        cpu_elapsed_micros: u64,
    ) -> Self {
        Self {
            pass_name: pass_name.into(),
            executor_id: executor_id.into(),
            budget_key: RenderBudgetKey::Other,
            cpu_elapsed_micros,
            draw_count: 0,
            instance_count: 0,
            state_change_count: 0,
            dispatch_count: 0,
            upload_bytes: 0,
            native_resource_creates: RenderPassNativeResourceCreateMetrics::default(),
        }
    }

    pub fn with_budget_key(mut self, budget_key: RenderBudgetKey) -> Self {
        self.budget_key = budget_key;
        self
    }

    pub fn with_compute_metrics(mut self, dispatch_count: u32, upload_bytes: u64) -> Self {
        self.dispatch_count = dispatch_count;
        self.upload_bytes = upload_bytes;
        self
    }

    pub fn with_render_metrics(mut self, metrics: RenderGraphPassProfileMetrics) -> Self {
        self.draw_count = metrics.draw_count;
        self.instance_count = metrics.instance_count;
        self.state_change_count = metrics.state_change_count;
        self
    }

    pub fn with_native_resource_creates(
        mut self,
        metrics: RenderPassNativeResourceCreateMetrics,
    ) -> Self {
        self.native_resource_creates = metrics;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphExecutionProfileReport {
    pub pass_profiles: Vec<RenderGraphPassProfileRecord>,
}

impl RenderGraphExecutionProfileReport {
    pub fn new(pass_profiles: Vec<RenderGraphPassProfileRecord>) -> Self {
        Self { pass_profiles }
    }

    pub fn pass_count(&self) -> usize {
        self.pass_profiles.len()
    }

    pub fn total_cpu_elapsed_micros(&self) -> u64 {
        self.pass_profiles.iter().fold(0_u64, |total, record| {
            total.saturating_add(record.cpu_elapsed_micros)
        })
    }

    pub fn max_cpu_elapsed_micros(&self) -> u64 {
        self.pass_profiles
            .iter()
            .map(|record| record.cpu_elapsed_micros)
            .max()
            .unwrap_or(0)
    }
}

/// Distinguishes graph stages that could use the parallel encoder path from those that did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphParallelRecordingReport {
    pub eligible_stage_count: usize,
    pub eligible_bucket_count: usize,
    pub executed_stage_count: usize,
    pub executed_bucket_count: usize,
}

/// Immutable structural counts lowered from the compiled render-graph packet.
///
/// These values describe planned batches only. They do not claim that a batch
/// executed or provide CPU/GPU timing evidence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphExecutionBatchReport {
    pub planned_batch_count: usize,
    pub planned_live_pass_count: usize,
    pub graphics_batch_count: usize,
    pub async_compute_batch_count: usize,
    pub async_copy_batch_count: usize,
    pub max_passes_per_batch: usize,
    pub queue_transition_count: usize,
}

impl RenderGraphExecutionBatchReport {
    pub const fn new(
        planned_batch_count: usize,
        planned_live_pass_count: usize,
        graphics_batch_count: usize,
        async_compute_batch_count: usize,
        async_copy_batch_count: usize,
        max_passes_per_batch: usize,
        queue_transition_count: usize,
    ) -> Self {
        Self {
            planned_batch_count,
            planned_live_pass_count,
            graphics_batch_count,
            async_compute_batch_count,
            async_copy_batch_count,
            max_passes_per_batch,
            queue_transition_count,
        }
    }
}

impl RenderGraphParallelRecordingReport {
    pub const fn new(
        eligible_stage_count: usize,
        eligible_bucket_count: usize,
        executed_stage_count: usize,
        executed_bucket_count: usize,
    ) -> Self {
        Self {
            eligible_stage_count,
            eligible_bucket_count,
            executed_stage_count,
            executed_bucket_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphExecutionCoverageReport {
    pub planned_live_pass_count: usize,
    pub executed_pass_count: usize,
    pub matched_planned_pass_count: usize,
    pub missing_planned_pass_count: usize,
    pub unexpected_executed_pass_count: usize,
    pub duplicate_executed_pass_count: usize,
}

impl RenderGraphExecutionCoverageReport {
    pub const fn new(
        planned_live_pass_count: usize,
        executed_pass_count: usize,
        matched_planned_pass_count: usize,
        missing_planned_pass_count: usize,
        unexpected_executed_pass_count: usize,
        duplicate_executed_pass_count: usize,
    ) -> Self {
        Self {
            planned_live_pass_count,
            executed_pass_count,
            matched_planned_pass_count,
            missing_planned_pass_count,
            unexpected_executed_pass_count,
            duplicate_executed_pass_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphStageExecutionReport {
    pub staged_pass_count: usize,
    pub unstaged_pass_count: usize,
    pub unique_stage_count: usize,
    pub stage_transition_count: usize,
    pub stage_order_violation_count: usize,
}

impl RenderGraphStageExecutionReport {
    pub const fn new(
        staged_pass_count: usize,
        unstaged_pass_count: usize,
        unique_stage_count: usize,
        stage_transition_count: usize,
        stage_order_violation_count: usize,
    ) -> Self {
        Self {
            staged_pass_count,
            unstaged_pass_count,
            unique_stage_count,
            stage_transition_count,
            stage_order_violation_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderSceneVelocityReadbackReport {
    pub available: bool,
    pub size: UVec2,
    pub byte_len: usize,
    pub nonzero_pixel_count: usize,
}

impl RenderSceneVelocityReadbackReport {
    pub fn from_raw_rg16_float_bytes(size: UVec2, bytes: &[u8]) -> Self {
        let nonzero_pixel_count = bytes
            .chunks_exact(4)
            .filter(|pixel| rg16_float_pixel_has_nonzero_value(pixel))
            .count();
        Self {
            available: true,
            size,
            byte_len: bytes.len(),
            nonzero_pixel_count,
        }
    }
}

fn rg16_float_pixel_has_nonzero_value(pixel: &[u8]) -> bool {
    let x = u16::from_le_bytes([pixel[0], pixel[1]]) & 0x7fff;
    let y = u16::from_le_bytes([pixel[2], pixel[3]]) & 0x7fff;
    x != 0 || y != 0
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MotionVectorCameraStatus {
    #[default]
    NotRequested,
    MissingPreviousCamera,
    CameraCutOrInvalid,
    Ready,
}

impl MotionVectorCameraStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::MissingPreviousCamera => "missing_previous_camera",
            Self::CameraCutOrInvalid => "camera_cut_or_invalid",
            Self::Ready => "ready",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::RenderGraphPassProfileMetrics;

    #[test]
    fn pass_profile_metrics_are_available_from_the_framework_render_root() {
        let metrics = RenderGraphPassProfileMetrics::new(3, 5, 7);

        assert_eq!(metrics.draw_count, 3);
        assert_eq!(metrics.instance_count, 5);
        assert_eq!(metrics.state_change_count, 7);
    }
}
