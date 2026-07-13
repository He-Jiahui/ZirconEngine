use crate::core::framework::render::{
    IblBakeArtifactRequest, ProceduralSkyParams, SOURCE_CUBEMAP_PMREM_FACE_SIZE,
    SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::render_graph::RenderGraphBuilder;

use super::ibl_bake_wgpu_pipeline_cache::IblBakeWgpuPipelineCache;
use super::realtime_ibl_gpu_resources::RealtimeIblGpuResources;
use super::realtime_ibl_gpu_timestamps::{
    RealtimeIblGpuTimestampCollector, RealtimeIblGpuTimestampReadback,
    RealtimeIblGpuTimingMetadata, RealtimeIblGpuTimingReport,
};
use super::realtime_ibl_graph_plan::append_realtime_ibl_graph_plan;
use super::realtime_ibl_time_slice::{
    IblRealtimeBufferSlot, RealtimeIblBatchToken, RealtimeIblFrameBatch, RealtimeIblOperation,
    RealtimeIblTimeSliceConfig, RealtimeIblTimeSliceScheduler,
};
use super::realtime_ibl_wgpu_recorder::{RealtimeIblWgpuRecordReport, RealtimeIblWgpuRecorder};

const REALTIME_IBL_SOURCE_FACE_SIZE: u32 = SOURCE_CUBEMAP_PMREM_FACE_SIZE;
const REALTIME_IBL_SOURCE_MIP_COUNT: u32 = SOURCE_CUBEMAP_PMREM_MIP_COUNT;
const REALTIME_IBL_CAPTURE_FACES_PER_FRAME: u8 = 2;

#[derive(Clone, Debug)]
pub(in crate::graphics) struct RealtimeIblPreparedFrame {
    batch: Option<RealtimeIblFrameBatch>,
    request: IblBakeArtifactRequest,
    sky: ProceduralSkyParams,
    sampling_slot: IblRealtimeBufferSlot,
}

impl RealtimeIblPreparedFrame {
    pub(in crate::graphics) fn sampling_slot(&self) -> IblRealtimeBufferSlot {
        self.sampling_slot
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
    timestamp_readback: Option<RealtimeIblGpuTimestampReadback>,
    timestamp_metadata: RealtimeIblGpuTimingMetadata,
}

pub(in crate::graphics) struct RealtimeIblRuntime {
    scheduler: RealtimeIblTimeSliceScheduler,
    resources: RealtimeIblGpuResources,
    recorder: RealtimeIblWgpuRecorder,
    timestamp_collector: RealtimeIblGpuTimestampCollector,
    frame_number: u64,
}

impl RealtimeIblRuntime {
    pub(in crate::graphics) fn new(device: &wgpu::Device) -> Self {
        let initial_sky = ProceduralSkyParams::default_gradient();
        let request = request_for_sky(&initial_sky);
        Self {
            scheduler: RealtimeIblTimeSliceScheduler::new(
                RealtimeIblTimeSliceConfig::try_new(
                    SOURCE_CUBEMAP_PMREM_MIP_COUNT as u8,
                    REALTIME_IBL_CAPTURE_FACES_PER_FRAME,
                )
                .expect("realtime IBL constants must form a valid scheduler config"),
            ),
            resources: RealtimeIblGpuResources::new(device, &request),
            recorder: RealtimeIblWgpuRecorder::new(device),
            timestamp_collector: RealtimeIblGpuTimestampCollector::new(device),
            frame_number: 0,
        }
    }

    pub(in crate::graphics) fn prepare_frame(
        &mut self,
        sky: ProceduralSkyParams,
    ) -> RealtimeIblPreparedFrame {
        self.scheduler.request_rebake(runtime_bake_key(&sky));
        self.frame_number = self.frame_number.wrapping_add(1);
        let batch = self.scheduler.begin_frame(self.frame_number);
        let request = request_for_sky(&sky);
        let sampling_slot = batch
            .as_ref()
            .map(sampling_slot_for_batch)
            .unwrap_or_else(|| self.scheduler.ready_slot());
        RealtimeIblPreparedFrame {
            batch,
            request,
            sky,
            sampling_slot,
        }
    }

    pub(in crate::graphics) fn record_prepared_frame(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        prepared: &RealtimeIblPreparedFrame,
        pipeline_cache: &mut IblBakeWgpuPipelineCache,
    ) -> Result<Option<RealtimeIblPendingSubmission>, String> {
        let Some(batch) = prepared.batch.as_ref() else {
            return Ok(None);
        };
        let result = self.record_prepared_frame_inner(device, encoder, prepared, pipeline_cache);
        if result.is_err() {
            self.scheduler.complete_frame(batch.token(), false);
        }
        result.map(Some)
    }

    fn record_prepared_frame_inner(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        prepared: &RealtimeIblPreparedFrame,
        pipeline_cache: &mut IblBakeWgpuPipelineCache,
    ) -> Result<RealtimeIblPendingSubmission, String> {
        let batch = prepared
            .batch
            .as_ref()
            .ok_or_else(|| "realtime IBL prepared frame has no pending batch".to_string())?;
        let mut builder = RenderGraphBuilder::new("realtime-ibl-frame");
        let plan = append_realtime_ibl_graph_plan(&mut builder, &prepared.request, batch)
            .map_err(|error| error.to_string())?;
        let graph = builder.compile().map_err(|error| error.to_string())?;
        let mut graph_resources = RenderGraphExecutionResources::new();
        self.resources
            .bind_graph_plan(&plan, &graph, &mut graph_resources)?;
        graph_resources.validate_materialized_graph_resources(&graph)?;
        let result = self.recorder.record_graph_plan(
            device,
            encoder,
            &prepared.request,
            &prepared.sky,
            &plan,
            &self.resources,
            pipeline_cache,
        )?;
        let timestamp_metadata = RealtimeIblGpuTimingMetadata {
            frame_number: self.frame_number,
            logical_state: batch.logical_state(),
            full_update: batch.is_full_update(),
            operation_label: operation_label(batch.operations()),
            pass_count: result.report.pass_count,
            dispatch_count: result.report.dispatch_count,
        };
        Ok(RealtimeIblPendingSubmission {
            token: batch.token(),
            report: result.report,
            timestamp_readback: result.timestamp_readback,
            timestamp_metadata,
        })
    }

    pub(in crate::graphics) fn complete_submission(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        submission: RealtimeIblPendingSubmission,
        gpu_succeeded: bool,
    ) {
        if gpu_succeeded {
            if let Some(readback) = submission.timestamp_readback {
                self.timestamp_collector.begin_readback(
                    readback,
                    submission.timestamp_metadata,
                    queue.get_timestamp_period(),
                );
            }
        }
        self.scheduler
            .complete_frame(submission.token, gpu_succeeded);
        self.timestamp_collector.poll(device, false);
    }

    pub(in crate::graphics) fn poll_gpu_timestamps(&mut self, device: &wgpu::Device) {
        self.timestamp_collector.poll(device, false);
    }

    pub(in crate::graphics) fn gpu_timestamps_supported(&self) -> bool {
        self.timestamp_collector.is_supported()
    }

    pub(in crate::graphics) fn take_gpu_timing_reports(
        &mut self,
        device: &wgpu::Device,
    ) -> Vec<RealtimeIblGpuTimingReport> {
        self.timestamp_collector.poll(device, true);
        self.timestamp_collector.take_completed()
    }

    pub(in crate::graphics) fn source_view(
        &self,
        slot: IblRealtimeBufferSlot,
    ) -> &wgpu::TextureView {
        self.resources.source_sampled(slot)
    }

    pub(in crate::graphics) fn pmrem_view(
        &self,
        slot: IblRealtimeBufferSlot,
    ) -> &wgpu::TextureView {
        self.resources.pmrem_sampled(slot)
    }

    pub(in crate::graphics) fn sh9_buffer(&self, slot: IblRealtimeBufferSlot) -> &wgpu::Buffer {
        self.resources.sh9(slot)
    }
}

fn request_for_sky(sky: &ProceduralSkyParams) -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        runtime_bake_key(sky),
        REALTIME_IBL_SOURCE_FACE_SIZE,
        REALTIME_IBL_SOURCE_MIP_COUNT,
    )
}

fn runtime_bake_key(sky: &ProceduralSkyParams) -> crate::core::framework::render::IblBakeKey {
    sky.ibl_bake_key()
}

fn sampling_slot_for_batch(batch: &RealtimeIblFrameBatch) -> IblRealtimeBufferSlot {
    if batch.is_full_update() {
        batch.work_slot()
    } else {
        batch.ready_slot()
    }
}

fn operation_label(operations: &[RealtimeIblOperation]) -> String {
    operations
        .iter()
        .map(|operation| match operation {
            RealtimeIblOperation::CaptureSky(_) => "capture_sky",
            RealtimeIblOperation::CaptureCloud(_) => "capture_cloud",
            RealtimeIblOperation::GenerateSourceMips => "source_mips",
            RealtimeIblOperation::Prefilter { .. } => "ggx_pmrem",
            RealtimeIblOperation::ProjectDiffuseSh9 => "diffuse_sh9",
        })
        .collect::<Vec<_>>()
        .join("+")
}

#[cfg(test)]
mod tests;
