use crate::core::framework::render::{
    IblBakeArtifactRequest, IblBakeKey, ProceduralSkyParams, RealtimeIblFailureKind,
    RealtimeIblStatusReport, SOURCE_CUBEMAP_PMREM_FACE_SIZE, SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};
use crate::core::runtime::diagnostics::profiling;
use crate::graphics::backend::RenderBackend;

use super::ibl_bake_wgpu_pipeline_cache::IblBakeWgpuPipelineCache;
use super::realtime_ibl_capture_wgpu::REALTIME_IBL_SOURCE_SHADER_CONTENT_IDENTITY;
use super::realtime_ibl_cpu_timing::{
    RealtimeIblCpuTimingCollector, RealtimeIblCpuTimingReport, RealtimeIblGraphPreparationReport,
};
use super::realtime_ibl_gpu_resources::RealtimeIblGpuResources;
use super::realtime_ibl_gpu_timestamps::{
    RealtimeIblGpuTimestampCollector, RealtimeIblGpuTimestampReadback,
    RealtimeIblGpuTimingMetadata, RealtimeIblGpuTimingReport,
};
use super::realtime_ibl_time_slice::{
    frame_sequence_age, IblRealtimeBufferSlot, RealtimeIblBatchToken, RealtimeIblCompletion,
    RealtimeIblFrameBatch, RealtimeIblOperation, RealtimeIblSliceAttempt,
    RealtimeIblTimeSliceConfig, RealtimeIblTimeSliceScheduler,
};
use super::realtime_ibl_wgpu_recorder::{RealtimeIblWgpuRecordReport, RealtimeIblWgpuRecorder};

mod compiled_graph_cache;

pub(in crate::graphics) use compiled_graph_cache::RealtimeIblCompiledGraphCacheStats;

use compiled_graph_cache::RealtimeIblCompiledGraphCache;

const REALTIME_IBL_SOURCE_FACE_SIZE: u32 = SOURCE_CUBEMAP_PMREM_FACE_SIZE;
const REALTIME_IBL_SOURCE_MIP_COUNT: u32 = SOURCE_CUBEMAP_PMREM_MIP_COUNT;
const REALTIME_IBL_CAPTURE_FACES_PER_FRAME: u8 = 2;

#[derive(Clone, Debug)]
pub(in crate::graphics) struct RealtimeIblPreparedFrame {
    batch: Option<RealtimeIblFrameBatch>,
    request: IblBakeArtifactRequest,
    sky: ProceduralSkyParams,
    sampling_slot: IblRealtimeBufferSlot,
    uses_realtime_resources: bool,
}

impl RealtimeIblPreparedFrame {
    pub(in crate::graphics) fn sampling_slot(&self) -> IblRealtimeBufferSlot {
        self.sampling_slot
    }

    pub(in crate::graphics) fn uses_realtime_resources(&self) -> bool {
        self.uses_realtime_resources
    }

    pub(in crate::graphics) fn source_face_size(&self) -> u32 {
        self.request.source_face_size()
    }

    pub(in crate::graphics) fn pmrem_face_size(&self) -> u32 {
        self.request.pmrem_face_size()
    }

    pub(in crate::graphics) fn pmrem_mip_count(&self) -> u32 {
        self.request.pmrem_mip_count()
    }
}

#[derive(Clone, Debug)]
pub(in crate::graphics) struct RealtimeIblPendingSubmission {
    token: RealtimeIblBatchToken,
    pub report: RealtimeIblWgpuRecordReport,
    pub graph_preparation: RealtimeIblGraphPreparationReport,
    cpu_timing_report: Option<RealtimeIblCpuTimingReport>,
    timestamp_readback: Option<RealtimeIblGpuTimestampReadback>,
    timestamp_metadata: Option<RealtimeIblGpuTimingMetadata>,
}

pub(in crate::graphics) struct RealtimeIblRuntime {
    scheduler: RealtimeIblTimeSliceScheduler,
    resources: Option<RealtimeIblGpuResources>,
    recorder: Option<RealtimeIblWgpuRecorder>,
    compiled_graph_cache: RealtimeIblCompiledGraphCache,
    cpu_timing_collector: RealtimeIblCpuTimingCollector,
    timestamp_collector: Option<RealtimeIblGpuTimestampCollector>,
    active_sky: Option<ProceduralSkyParams>,
    queued_sky: Option<ProceduralSkyParams>,
    published_generation_frame_number: Option<u64>,
    active_generation_start_frame_number: Option<u64>,
    active_generation_coalesced_source_change_count: u64,
    frame_number: u64,
}

impl RealtimeIblRuntime {
    pub(in crate::graphics) fn new() -> Self {
        Self {
            scheduler: RealtimeIblTimeSliceScheduler::new(
                RealtimeIblTimeSliceConfig::try_new(
                    SOURCE_CUBEMAP_PMREM_MIP_COUNT as u8,
                    REALTIME_IBL_CAPTURE_FACES_PER_FRAME,
                )
                .expect("realtime IBL constants must form a valid scheduler config"),
            ),
            resources: None,
            recorder: None,
            compiled_graph_cache: RealtimeIblCompiledGraphCache::new(),
            cpu_timing_collector: RealtimeIblCpuTimingCollector::default(),
            timestamp_collector: None,
            active_sky: None,
            queued_sky: None,
            published_generation_frame_number: None,
            active_generation_start_frame_number: None,
            active_generation_coalesced_source_change_count: 0,
            frame_number: 0,
        }
    }

    pub(in crate::graphics) fn prepare_frame(
        &mut self,
        device: &wgpu::Device,
        sky: ProceduralSkyParams,
    ) -> RealtimeIblPreparedFrame {
        self.ensure_gpu_resources(device);
        let (bake_key, bake_sky) = self.resolve_bake_snapshot(sky);
        self.frame_number = self.frame_number.wrapping_add(1);
        let batch = self.scheduler.begin_frame(self.frame_number);
        let request = request_for_key(bake_key);
        let sampling_slot = self.scheduler.ready_slot();
        RealtimeIblPreparedFrame {
            batch,
            request,
            sky: bake_sky,
            sampling_slot,
            uses_realtime_resources: self.scheduler.has_published_environment(),
        }
    }

    fn resolve_bake_snapshot(
        &mut self,
        requested_sky: ProceduralSkyParams,
    ) -> (IblBakeKey, ProceduralSkyParams) {
        let requested_key = runtime_bake_key(&requested_sky);
        if let Some(active_key) = self.scheduler.pending_key() {
            if requested_key == active_key {
                self.queued_sky = None;
            } else {
                let latest_slot_changed = self
                    .queued_sky
                    .is_none_or(|queued| runtime_bake_key(&queued) != requested_key);
                if latest_slot_changed {
                    self.active_generation_coalesced_source_change_count = self
                        .active_generation_coalesced_source_change_count
                        .saturating_add(1);
                }
                self.queued_sky = Some(requested_sky);
            }
            let active_sky = self
                .active_sky
                .expect("a pending realtime IBL generation must own its sky snapshot");
            debug_assert_eq!(runtime_bake_key(&active_sky), active_key);
            return (active_key, active_sky);
        }

        self.clear_active_generation();
        self.queued_sky = None;
        let needs_reconciliation = self.scheduler.published_key() != Some(requested_key)
            || self.scheduler.failure_report().is_some();
        if needs_reconciliation && self.scheduler.request_rebake(requested_key) {
            self.begin_active_generation(requested_sky);
        }
        (requested_key, requested_sky)
    }

    fn begin_active_generation(&mut self, sky: ProceduralSkyParams) {
        self.active_sky = Some(sky);
        self.active_generation_start_frame_number = Some(self.frame_number.wrapping_add(1));
        self.active_generation_coalesced_source_change_count = 0;
    }

    fn clear_active_generation(&mut self) {
        self.active_sky = None;
        self.active_generation_start_frame_number = None;
        self.active_generation_coalesced_source_change_count = 0;
    }

    fn complete_scheduler_frame(
        &mut self,
        token: RealtimeIblBatchToken,
        attempt: RealtimeIblSliceAttempt,
    ) -> RealtimeIblCompletion {
        let completion = self.scheduler.complete_attempt(token, attempt);
        if completion == RealtimeIblCompletion::Failed {
            self.clear_active_generation();
            self.queued_sky = None;
            return completion;
        }
        if completion != RealtimeIblCompletion::Published {
            return completion;
        }

        self.published_generation_frame_number = Some(self.frame_number);
        self.clear_active_generation();
        let Some(queued_sky) = self.queued_sky.take() else {
            return completion;
        };
        let queued_key = runtime_bake_key(&queued_sky);
        if self.scheduler.published_key() != Some(queued_key) {
            let started = self.scheduler.request_rebake(queued_key);
            debug_assert!(started);
            self.begin_active_generation(queued_sky);
        }
        completion
    }

    fn ensure_gpu_resources(&mut self, device: &wgpu::Device) {
        if self.resources.is_some() {
            return;
        }

        let request = request_for_key(runtime_bake_key(&ProceduralSkyParams::default_gradient()));
        self.resources = Some(RealtimeIblGpuResources::new(device, &request));
        self.recorder = Some(RealtimeIblWgpuRecorder::new(device));
        self.timestamp_collector = Some(RealtimeIblGpuTimestampCollector::new(device));
    }

    pub(in crate::graphics) fn record_prepared_frame(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        gpu_timing_enabled: bool,
        prepared: &RealtimeIblPreparedFrame,
        pipeline_cache: &mut IblBakeWgpuPipelineCache,
    ) -> Result<Option<RealtimeIblPendingSubmission>, String> {
        let Some(batch) = prepared.batch.as_ref() else {
            return Ok(None);
        };
        let result = self.record_prepared_frame_inner(
            device,
            encoder,
            gpu_timing_enabled,
            prepared,
            pipeline_cache,
        );
        if result.is_err() {
            self.complete_scheduler_frame(
                batch.token(),
                RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Recording),
            );
        }
        result.map(Some)
    }

    fn record_prepared_frame_inner(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        gpu_timing_enabled: bool,
        prepared: &RealtimeIblPreparedFrame,
        pipeline_cache: &mut IblBakeWgpuPipelineCache,
    ) -> Result<RealtimeIblPendingSubmission, String> {
        let batch = prepared
            .batch
            .as_ref()
            .ok_or_else(|| "realtime IBL prepared frame has no pending batch".to_string())?;
        let frame_number = self.frame_number;
        let generation_start_frame_number = self
            .active_generation_start_frame_number
            .expect("a recorded realtime IBL batch must own its generation start frame");
        let generation_elapsed_frame_count =
            frame_sequence_age(frame_number, generation_start_frame_number)
                .expect("a recorded realtime IBL batch cannot precede its generation start frame")
                .saturating_add(1);
        let coalesced_source_change_count = self.active_generation_coalesced_source_change_count;
        let queued_generation_pending = self.queued_sky.is_some();
        let artifact = self
            .compiled_graph_cache
            .resolve(&prepared.request, batch)
            .map_err(|error| error.to_string())?;
        let cpu_capture_epoch = profiling::capture_epoch();
        let cpu_timing_enabled = cpu_capture_epoch.is_some();
        // This path only runs while a rebake batch records. Keep graph-resource
        // preparation separate from recorder binding creation for CPU profiling.
        let resources = self
            .resources
            .as_mut()
            .expect("realtime IBL resources must be initialized by prepare_frame");
        let graph_preparation = {
            let execution_resources = resources.execution_resources_for(
                &prepared.request,
                batch,
                artifact.plan(),
                artifact.graph(),
                artifact.required_resource_names(),
                cpu_timing_enabled,
            )?;
            RealtimeIblGraphPreparationReport {
                execution_resource_binding_micros: execution_resources
                    .execution_resource_binding_micros(),
                validation_micros: execution_resources.validation_micros(),
                execution_resource_cache_hits: execution_resources.execution_resource_cache_hits(),
                execution_resource_cache_misses: execution_resources
                    .execution_resource_cache_misses(),
                execution_resource_cache_entry_count: execution_resources
                    .execution_resource_cache_entry_count(),
                execution_resource_cache_topology_capacity: execution_resources
                    .execution_resource_cache_topology_capacity(),
                texture_view_binding_count: execution_resources.texture_view_binding_count(),
                buffer_binding_count: execution_resources.buffer_binding_count(),
                total_bound_resource_count: execution_resources
                    .texture_view_binding_count()
                    .saturating_add(execution_resources.buffer_binding_count()),
            }
        };
        let recorder = self
            .recorder
            .as_mut()
            .expect("realtime IBL recorder must be initialized by prepare_frame");
        let result = recorder.record_graph_plan(
            device,
            encoder,
            gpu_timing_enabled,
            cpu_timing_enabled,
            &prepared.request,
            &prepared.sky,
            artifact.plan(),
            artifact.recording_passes(),
            resources,
            pipeline_cache,
        )?;
        let scheduled_workgroups = dispatch_workgroup_count(&result.report.dispatch_groups);
        let terminal_reason = if batch.completes_generation() {
            "published_after_sh9"
        } else {
            "advanced"
        };
        let cpu_timing_report = cpu_capture_epoch.map(|profile_capture_epoch| {
            RealtimeIblCpuTimingReport::from_recording(
                profile_capture_epoch,
                frame_number,
                generation_start_frame_number,
                generation_elapsed_frame_count,
                coalesced_source_change_count,
                queued_generation_pending,
                batch.token().generation(),
                recipe_fingerprint(&prepared.request),
                batch.logical_state(),
                slot_label(batch.work_slot()).to_string(),
                operation_label(batch.operations()),
                scheduled_workgroups,
                terminal_reason.to_string(),
                &result.report,
                graph_preparation,
            )
        });
        // The public timing drain is timestamp-readback backed. On adapters without
        // timestamp queries, CPU profiling uses a separate accepted-submission drain.
        let timestamp_metadata =
            result
                .timestamp_readback
                .as_ref()
                .map(|_| RealtimeIblGpuTimingMetadata {
                    frame_number,
                    generation: batch.token().generation(),
                    recipe_fingerprint: recipe_fingerprint(&prepared.request),
                    logical_state: batch.logical_state(),
                    work_slot: slot_label(batch.work_slot()).to_string(),
                    operation_label: operation_label(batch.operations()),
                    pass_count: result.report.pass_count,
                    dispatch_count: result.report.dispatch_count,
                    binding_cache_hits: result.report.binding_cache_hits,
                    binding_cache_misses: result.report.binding_cache_misses,
                    params_buffer_creations: result.report.params_buffer_creations,
                    bind_group_creations: result.report.bind_group_creations,
                    binding_cache_resets: result.report.binding_cache_resets,
                    capture_params_buffer_creations: result.report.capture_params_buffer_creations,
                    capture_bind_group_creations: result.report.capture_bind_group_creations,
                    source_mip_params_buffer_creations: result
                        .report
                        .source_mip_params_buffer_creations,
                    source_mip_bind_group_creations: result.report.source_mip_bind_group_creations,
                    scheduled_workgroups,
                    completed_workgroups: scheduled_workgroups,
                    terminal_reason: terminal_reason.to_string(),
                });
        Ok(RealtimeIblPendingSubmission {
            token: batch.token(),
            report: result.report,
            graph_preparation,
            cpu_timing_report,
            timestamp_readback: result.timestamp_readback,
            timestamp_metadata,
        })
    }

    pub(in crate::graphics) fn complete_submission(
        &mut self,
        submission: RealtimeIblPendingSubmission,
        gpu_succeeded: bool,
    ) {
        self.complete_scheduler_frame(
            submission.token,
            if gpu_succeeded {
                RealtimeIblSliceAttempt::Succeeded
            } else {
                RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Submission)
            },
        );
        if let Some(report) = submission.cpu_timing_report.filter(|report| {
            gpu_succeeded
                && profiling::capture_epoch_for_completion() == Some(report.profile_capture_epoch)
        }) {
            self.cpu_timing_collector.record_completed(report);
        }
    }

    #[cfg(test)]
    fn complete_recording_without_submission(&mut self, submission: RealtimeIblPendingSubmission) {
        self.complete_scheduler_frame(submission.token, RealtimeIblSliceAttempt::Succeeded);
    }

    pub(in crate::graphics) fn request_product_gpu_timestamp_readback(
        &mut self,
        submission: &RealtimeIblPendingSubmission,
        timestamp_period_nanoseconds: f32,
        backend: &RenderBackend,
    ) -> bool {
        let (Some(readback), Some(metadata), Some(timestamp_collector)) = (
            submission.timestamp_readback.as_ref(),
            submission.timestamp_metadata.as_ref(),
            self.timestamp_collector.as_mut(),
        ) else {
            return false;
        };
        timestamp_collector.request_product_readback(
            readback,
            metadata.clone(),
            timestamp_period_nanoseconds,
            backend,
        )
    }

    pub(in crate::graphics) fn gpu_timestamps_supported(&self) -> bool {
        self.timestamp_collector
            .as_ref()
            .is_some_and(RealtimeIblGpuTimestampCollector::is_supported)
    }

    pub(in crate::graphics) fn compiled_graph_cache_stats(
        &self,
    ) -> RealtimeIblCompiledGraphCacheStats {
        self.compiled_graph_cache.stats()
    }

    pub(in crate::graphics) fn status_report(&self) -> RealtimeIblStatusReport {
        let published_key = self.scheduler.published_key();
        let pending_key = self.scheduler.pending_key();
        let failure = self.scheduler.failure_report();
        let last_good_age_frame_count = self
            .published_generation_frame_number
            .and_then(|published_frame| frame_sequence_age(self.frame_number, published_frame));
        let active_generation_elapsed_frame_count =
            self.active_generation_start_frame_number
                .map(|start_frame| {
                    frame_sequence_age(self.frame_number, start_frame)
                        .map(|age| age.saturating_add(1))
                        .unwrap_or(0)
                });
        RealtimeIblStatusReport {
            readiness: self.scheduler.readiness(),
            current_frame_number: self.frame_number,
            published_key,
            pending_key,
            queued_key: self.queued_sky.as_ref().map(runtime_bake_key),
            published_generation_frame_number: self.published_generation_frame_number,
            last_good_age_frame_count,
            active_generation_start_frame_number: self.active_generation_start_frame_number,
            active_generation_elapsed_frame_count,
            active_generation_coalesced_source_change_count: self
                .active_generation_coalesced_source_change_count,
            failure,
        }
    }

    pub(in crate::graphics) fn take_gpu_timing_reports(
        &mut self,
    ) -> Vec<RealtimeIblGpuTimingReport> {
        let Some(timestamp_collector) = self.timestamp_collector.as_mut() else {
            return Vec::new();
        };
        timestamp_collector.take_completed()
    }

    pub(in crate::graphics) fn take_cpu_timing_reports(
        &mut self,
    ) -> Vec<RealtimeIblCpuTimingReport> {
        self.cpu_timing_collector
            .synchronize_capture_epoch(profiling::capture_epoch_for_completion());
        self.cpu_timing_collector.take_completed()
    }

    pub(in crate::graphics) fn source_view(
        &self,
        slot: IblRealtimeBufferSlot,
    ) -> &wgpu::TextureView {
        self.resources
            .as_ref()
            .expect("realtime IBL resources must be initialized before sampling")
            .source_sampled(slot)
    }

    pub(in crate::graphics) fn pmrem_view(
        &self,
        slot: IblRealtimeBufferSlot,
    ) -> &wgpu::TextureView {
        self.resources
            .as_ref()
            .expect("realtime IBL resources must be initialized before sampling")
            .pmrem_sampled(slot)
    }

    pub(in crate::graphics) fn sh9_buffer(&self, slot: IblRealtimeBufferSlot) -> &wgpu::Buffer {
        self.resources
            .as_ref()
            .expect("realtime IBL resources must be initialized before sampling")
            .sh9(slot)
    }

    #[cfg(test)]
    fn is_gpu_initialized(&self) -> bool {
        self.resources.is_some()
    }
}

fn request_for_key(bake_key: IblBakeKey) -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        bake_key,
        REALTIME_IBL_SOURCE_FACE_SIZE,
        REALTIME_IBL_SOURCE_MIP_COUNT,
    )
}

fn runtime_bake_key(sky: &ProceduralSkyParams) -> IblBakeKey {
    let mut key = sky.ibl_bake_key();
    for (word, source_identity) in key
        .source_hash
        .iter_mut()
        .zip(REALTIME_IBL_SOURCE_SHADER_CONTENT_IDENTITY)
    {
        *word ^= source_identity;
    }
    key
}

fn operation_label(operations: &[RealtimeIblOperation]) -> String {
    let mut label = String::with_capacity(operations.len() * 16);
    for operation in operations {
        if !label.is_empty() {
            label.push('+');
        }
        label.push_str(match operation {
            RealtimeIblOperation::CaptureSky(_) => "capture_sky",
            RealtimeIblOperation::GenerateSourceMip { .. } => "source_mip",
            RealtimeIblOperation::Prefilter { .. } => "ggx_pmrem",
            RealtimeIblOperation::ProjectDiffuseSh9 => "diffuse_sh9",
        });
    }
    label
}

fn slot_label(slot: IblRealtimeBufferSlot) -> &'static str {
    match slot {
        IblRealtimeBufferSlot::A => "a",
        IblRealtimeBufferSlot::B => "b",
    }
}

fn dispatch_workgroup_count(dispatch_groups: &[[u32; 3]]) -> u64 {
    dispatch_groups.iter().fold(0_u64, |total, groups| {
        total.saturating_add(
            u64::from(groups[0])
                .saturating_mul(u64::from(groups[1]))
                .saturating_mul(u64::from(groups[2])),
        )
    })
}

fn recipe_fingerprint(request: &IblBakeArtifactRequest) -> String {
    let key = request.bake_key();
    format!(
        "{key:?}:{}x{}-{}x{}",
        request.source_face_size(),
        request.source_mip_count(),
        request.pmrem_face_size(),
        request.pmrem_mip_count(),
    )
}

#[cfg(test)]
mod tests;
