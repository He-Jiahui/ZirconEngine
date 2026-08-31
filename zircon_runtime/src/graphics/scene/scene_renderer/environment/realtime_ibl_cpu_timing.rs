use std::collections::VecDeque;

use super::realtime_ibl_wgpu_recorder::RealtimeIblWgpuRecordReport;

const REALTIME_IBL_CPU_TIMING_REPORT_CAPACITY: usize = 256;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RealtimeIblCpuTimingReport {
    pub profile_capture_epoch: u64,
    pub frame_number: u64,
    pub generation_start_frame_number: u64,
    pub generation_elapsed_frame_count: u64,
    pub coalesced_source_change_count: u64,
    pub queued_generation_pending: bool,
    pub generation: u64,
    pub recipe_fingerprint: String,
    pub logical_state: u8,
    pub work_slot: String,
    pub operation_label: String,
    pub pass_count: usize,
    pub dispatch_count: usize,
    pub binding_cache_hits: usize,
    pub binding_cache_misses: usize,
    pub params_buffer_creations: usize,
    pub bind_group_creations: usize,
    pub binding_cache_resets: usize,
    pub command_plan_creation_micros: u64,
    pub pipeline_ensure_micros: u64,
    pub binding_creation_micros: u64,
    pub capture_params_buffer_creations: usize,
    pub capture_bind_group_creations: usize,
    pub capture_binding_creation_micros: u64,
    pub source_mip_params_buffer_creations: usize,
    pub source_mip_bind_group_creations: usize,
    pub source_mip_binding_creation_micros: u64,
    pub execution_resource_binding_micros: u64,
    pub validation_micros: u64,
    pub execution_resource_cache_hits: u64,
    pub execution_resource_cache_misses: u64,
    pub execution_resource_cache_entry_count: usize,
    pub execution_resource_cache_topology_capacity: usize,
    pub texture_view_binding_count: usize,
    pub buffer_binding_count: usize,
    pub total_bound_resource_count: usize,
    pub scheduled_workgroups: u64,
    pub terminal_reason: String,
    /// Number of earlier reports overwritten in this capture before this sample.
    pub overwritten_report_count: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblGraphPreparationReport {
    pub execution_resource_binding_micros: u64,
    pub validation_micros: u64,
    pub execution_resource_cache_hits: u64,
    pub execution_resource_cache_misses: u64,
    pub execution_resource_cache_entry_count: usize,
    pub execution_resource_cache_topology_capacity: usize,
    pub texture_view_binding_count: usize,
    pub buffer_binding_count: usize,
    pub total_bound_resource_count: usize,
}

impl RealtimeIblGraphPreparationReport {
    pub(in crate::graphics) fn accumulate(&mut self, other: Self) {
        self.execution_resource_binding_micros = self
            .execution_resource_binding_micros
            .saturating_add(other.execution_resource_binding_micros);
        self.validation_micros = self
            .validation_micros
            .saturating_add(other.validation_micros);
        self.execution_resource_cache_hits = self
            .execution_resource_cache_hits
            .saturating_add(other.execution_resource_cache_hits);
        self.execution_resource_cache_misses = self
            .execution_resource_cache_misses
            .saturating_add(other.execution_resource_cache_misses);
        self.execution_resource_cache_entry_count = self
            .execution_resource_cache_entry_count
            .max(other.execution_resource_cache_entry_count);
        self.execution_resource_cache_topology_capacity = self
            .execution_resource_cache_topology_capacity
            .max(other.execution_resource_cache_topology_capacity);
        self.texture_view_binding_count = self
            .texture_view_binding_count
            .saturating_add(other.texture_view_binding_count);
        self.buffer_binding_count = self
            .buffer_binding_count
            .saturating_add(other.buffer_binding_count);
        self.total_bound_resource_count = self
            .total_bound_resource_count
            .saturating_add(other.total_bound_resource_count);
    }
}

impl RealtimeIblCpuTimingReport {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics) fn from_recording(
        profile_capture_epoch: u64,
        frame_number: u64,
        generation_start_frame_number: u64,
        generation_elapsed_frame_count: u64,
        coalesced_source_change_count: u64,
        queued_generation_pending: bool,
        generation: u64,
        recipe_fingerprint: String,
        logical_state: u8,
        work_slot: String,
        operation_label: String,
        scheduled_workgroups: u64,
        terminal_reason: String,
        record: &RealtimeIblWgpuRecordReport,
        graph_preparation: RealtimeIblGraphPreparationReport,
    ) -> Self {
        Self {
            profile_capture_epoch,
            frame_number,
            generation_start_frame_number,
            generation_elapsed_frame_count,
            coalesced_source_change_count,
            queued_generation_pending,
            generation,
            recipe_fingerprint,
            logical_state,
            work_slot,
            operation_label,
            pass_count: record.pass_count,
            dispatch_count: record.dispatch_count,
            binding_cache_hits: record.binding_cache_hits,
            binding_cache_misses: record.binding_cache_misses,
            params_buffer_creations: record.params_buffer_creations,
            bind_group_creations: record.bind_group_creations,
            binding_cache_resets: record.binding_cache_resets,
            command_plan_creation_micros: record.command_plan_creation_micros,
            pipeline_ensure_micros: record.pipeline_ensure_micros,
            binding_creation_micros: record.binding_creation_micros,
            capture_params_buffer_creations: record.capture_params_buffer_creations,
            capture_bind_group_creations: record.capture_bind_group_creations,
            capture_binding_creation_micros: record.capture_binding_creation_micros,
            source_mip_params_buffer_creations: record.source_mip_params_buffer_creations,
            source_mip_bind_group_creations: record.source_mip_bind_group_creations,
            source_mip_binding_creation_micros: record.source_mip_binding_creation_micros,
            execution_resource_binding_micros: graph_preparation.execution_resource_binding_micros,
            validation_micros: graph_preparation.validation_micros,
            execution_resource_cache_hits: graph_preparation.execution_resource_cache_hits,
            execution_resource_cache_misses: graph_preparation.execution_resource_cache_misses,
            execution_resource_cache_entry_count: graph_preparation
                .execution_resource_cache_entry_count,
            execution_resource_cache_topology_capacity: graph_preparation
                .execution_resource_cache_topology_capacity,
            texture_view_binding_count: graph_preparation.texture_view_binding_count,
            buffer_binding_count: graph_preparation.buffer_binding_count,
            total_bound_resource_count: graph_preparation.total_bound_resource_count,
            scheduled_workgroups,
            terminal_reason,
            overwritten_report_count: 0,
        }
    }
}

#[derive(Default)]
pub(in crate::graphics) struct RealtimeIblCpuTimingCollector {
    capture_epoch: Option<u64>,
    completed: VecDeque<RealtimeIblCpuTimingReport>,
    overwritten_report_count: u64,
}

impl RealtimeIblCpuTimingCollector {
    pub(in crate::graphics) fn synchronize_capture_epoch(&mut self, capture_epoch: Option<u64>) {
        let Some(capture_epoch) = capture_epoch else {
            return;
        };
        if self.capture_epoch != Some(capture_epoch) {
            self.capture_epoch = Some(capture_epoch);
            self.completed.clear();
            self.overwritten_report_count = 0;
        }
    }

    pub(in crate::graphics) fn record_completed(&mut self, mut report: RealtimeIblCpuTimingReport) {
        self.synchronize_capture_epoch(Some(report.profile_capture_epoch));
        if self.completed.len() >= REALTIME_IBL_CPU_TIMING_REPORT_CAPACITY {
            self.completed.pop_front();
            self.overwritten_report_count = self.overwritten_report_count.saturating_add(1);
        }
        report.overwritten_report_count = self.overwritten_report_count;
        self.completed.push_back(report);
    }

    pub(in crate::graphics) fn take_completed(&mut self) -> Vec<RealtimeIblCpuTimingReport> {
        self.completed.drain(..).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RealtimeIblCpuTimingCollector, RealtimeIblCpuTimingReport,
        REALTIME_IBL_CPU_TIMING_REPORT_CAPACITY,
    };

    fn report(capture_epoch: u64, frame_number: u64) -> RealtimeIblCpuTimingReport {
        RealtimeIblCpuTimingReport {
            profile_capture_epoch: capture_epoch,
            frame_number,
            ..RealtimeIblCpuTimingReport::default()
        }
    }

    #[test]
    fn cpu_timing_collector_evicts_oldest_reports_and_exposes_overwrite_count() {
        let mut collector = RealtimeIblCpuTimingCollector::default();
        for frame_number in 0..=REALTIME_IBL_CPU_TIMING_REPORT_CAPACITY as u64 {
            collector.record_completed(report(7, frame_number));
        }

        let reports = collector.take_completed();
        assert_eq!(reports.len(), REALTIME_IBL_CPU_TIMING_REPORT_CAPACITY);
        assert_eq!(reports.first().map(|report| report.frame_number), Some(1));
        assert_eq!(reports.last().map(|report| report.frame_number), Some(256));
        assert_eq!(
            reports.last().map(|report| report.overwritten_report_count),
            Some(1)
        );
    }

    #[test]
    fn cpu_timing_collector_never_mixes_profile_capture_epochs() {
        let mut collector = RealtimeIblCpuTimingCollector::default();
        collector.record_completed(report(7, 1));
        collector.record_completed(report(8, 2));

        let reports = collector.take_completed();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].profile_capture_epoch, 8);
        assert_eq!(reports[0].frame_number, 2);
        assert_eq!(reports[0].overwritten_report_count, 0);
    }

    #[test]
    fn cpu_timing_collector_discards_prior_samples_when_a_new_capture_starts() {
        let mut collector = RealtimeIblCpuTimingCollector::default();
        collector.record_completed(report(7, 1));
        collector.synchronize_capture_epoch(Some(8));

        assert!(collector.take_completed().is_empty());
    }
}
