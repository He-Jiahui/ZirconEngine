use crate::core::framework::render::{
    IblBakeArtifactRequest, IblBakeKey, ProceduralSkyParams, SOURCE_CUBEMAP_PMREM_FACE_SIZE,
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
    resources: Option<RealtimeIblGpuResources>,
    recorder: Option<RealtimeIblWgpuRecorder>,
    timestamp_collector: Option<RealtimeIblGpuTimestampCollector>,
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
            timestamp_collector: None,
            frame_number: 0,
        }
    }

    pub(in crate::graphics) fn prepare_frame(
        &mut self,
        device: &wgpu::Device,
        sky: ProceduralSkyParams,
    ) -> RealtimeIblPreparedFrame {
        self.ensure_gpu_resources(device);
        let bake_key = runtime_bake_key(&sky);
        self.scheduler.request_rebake(bake_key);
        self.frame_number = self.frame_number.wrapping_add(1);
        let batch = self.scheduler.begin_frame(self.frame_number);
        let request = request_for_key(bake_key);
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
        let resources = self
            .resources
            .as_ref()
            .expect("realtime IBL resources must be initialized by prepare_frame");
        let recorder = self
            .recorder
            .as_ref()
            .expect("realtime IBL recorder must be initialized by prepare_frame");
        resources.bind_graph_plan(&plan, &graph, &mut graph_resources)?;
        graph_resources.validate_materialized_graph_resources(&graph)?;
        let result = recorder.record_graph_plan(
            device,
            encoder,
            &prepared.request,
            &prepared.sky,
            &plan,
            resources,
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
            if let (Some(readback), Some(timestamp_collector)) = (
                submission.timestamp_readback,
                self.timestamp_collector.as_mut(),
            ) {
                timestamp_collector.begin_readback(
                    readback,
                    submission.timestamp_metadata,
                    queue.get_timestamp_period(),
                );
            }
        }
        self.scheduler
            .complete_frame(submission.token, gpu_succeeded);
        if let Some(timestamp_collector) = self.timestamp_collector.as_mut() {
            timestamp_collector.poll(device, false);
        }
    }

    pub(in crate::graphics) fn poll_gpu_timestamps(&mut self, device: &wgpu::Device) {
        if let Some(timestamp_collector) = self.timestamp_collector.as_mut() {
            timestamp_collector.poll(device, false);
        }
    }

    pub(in crate::graphics) fn gpu_timestamps_supported(&self) -> bool {
        self.timestamp_collector
            .as_ref()
            .is_some_and(RealtimeIblGpuTimestampCollector::is_supported)
    }

    pub(in crate::graphics) fn take_gpu_timing_reports(
        &mut self,
        device: &wgpu::Device,
    ) -> Vec<RealtimeIblGpuTimingReport> {
        let Some(timestamp_collector) = self.timestamp_collector.as_mut() else {
            return Vec::new();
        };
        timestamp_collector.poll(device, true);
        timestamp_collector.take_completed()
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
    let mut label = String::with_capacity(operations.len() * 16);
    for operation in operations {
        if !label.is_empty() {
            label.push('+');
        }
        label.push_str(match operation {
            RealtimeIblOperation::CaptureSky(_) => "capture_sky",
            RealtimeIblOperation::CaptureCloud(_) => "capture_cloud",
            RealtimeIblOperation::GenerateSourceMips => "source_mips",
            RealtimeIblOperation::Prefilter { .. } => "ggx_pmrem",
            RealtimeIblOperation::ProjectDiffuseSh9 => "diffuse_sh9",
        });
    }
    label
}

#[cfg(test)]
mod tests;
