use crate::core::framework::render::IblBakeArtifactContents;
use crate::core::framework::render::{IblBakeArtifactRequest, ProceduralSkyParams};
use crate::render_graph::RenderGraphComputeDispatchExtent;

use super::ibl_bake_shader_plan::IblBakeComputeKernelKind;
use super::ibl_bake_wgpu_binding::{
    create_ibl_bake_wgpu_bind_group, create_ibl_bake_wgpu_params_buffer,
    IblBakeWgpuOutputBindingResource,
};
use super::ibl_bake_wgpu_command_plan::{
    ibl_bake_wgpu_command_plan_for_request, ibl_bake_wgpu_prefilter_command_for_slice,
    IblBakeWgpuCommandPlan,
};
use super::ibl_bake_wgpu_dispatch::encode_ibl_bake_wgpu_compute_dispatch;
use super::ibl_bake_wgpu_pipeline_cache::IblBakeWgpuPipelineCache;
use super::realtime_ibl_capture_wgpu::RealtimeIblCaptureWgpuPipelines;
use super::realtime_ibl_gpu_resources::RealtimeIblGpuResources;
use super::realtime_ibl_gpu_timestamps::{
    RealtimeIblGpuTimestampReadback, RealtimeIblGpuTimestampRecorder,
};
use super::realtime_ibl_graph_plan::{RealtimeIblGraphPassKind, RealtimeIblGraphPlan};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblWgpuRecordReport {
    pub pass_count: usize,
    pub dispatch_count: usize,
    pub dispatch_groups: Vec<[u32; 3]>,
}

pub(in crate::graphics) struct RealtimeIblWgpuRecorder {
    capture: RealtimeIblCaptureWgpuPipelines,
    timestamps: Option<RealtimeIblGpuTimestampRecorder>,
}

pub(in crate::graphics) struct RealtimeIblWgpuRecordResult {
    pub report: RealtimeIblWgpuRecordReport,
    pub timestamp_readback: Option<RealtimeIblGpuTimestampReadback>,
}

impl RealtimeIblWgpuRecorder {
    pub(in crate::graphics) fn new(device: &wgpu::Device) -> Self {
        Self {
            capture: RealtimeIblCaptureWgpuPipelines::new(device),
            timestamps: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics) fn record_graph_plan(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        gpu_timing_enabled: bool,
        request: &IblBakeArtifactRequest,
        sky: &ProceduralSkyParams,
        plan: &RealtimeIblGraphPlan,
        resources: &RealtimeIblGpuResources,
        ibl_pipeline_cache: &mut IblBakeWgpuPipelineCache,
    ) -> Result<RealtimeIblWgpuRecordResult, String> {
        let work_slot = plan.work.slot;
        let compute_request = (*request).with_required_contents(IblBakeArtifactContents::PMREM_SH9);
        let mut dispatch_groups = Vec::with_capacity(plan.passes.len());
        let capture = &self.capture;
        let timestamp_recorder = Self::timestamp_recorder(
            &mut self.timestamps,
            device,
            gpu_timing_enabled,
        );
        if let Some(timestamps) = timestamp_recorder {
            timestamps.write_start(encoder);
        }
        for pass in &plan.passes {
            match pass.kind {
                RealtimeIblGraphPassKind::CaptureSky(faces)
                | RealtimeIblGraphPassKind::CaptureCloud(faces) => {
                    capture.record_capture(
                        device,
                        encoder,
                        sky,
                        request.source_face_size(),
                        faces,
                        resources.source_storage_mip(work_slot, 0)?,
                    );
                    dispatch_groups.push(fixed_dispatch_groups(&pass.workload.dispatch_extent)?);
                }
                RealtimeIblGraphPassKind::GenerateSourceMip { mip_level } => {
                    let source_mip = mip_level.saturating_sub(1);
                    capture.record_downsample_mip(
                        device,
                        encoder,
                        mip_dimension(request.source_face_size(), source_mip),
                        mip_dimension(request.source_face_size(), mip_level),
                        resources.source_sampled_mip(work_slot, source_mip)?,
                        resources.source_storage_mip(work_slot, mip_level)?,
                    );
                    dispatch_groups.push(fixed_dispatch_groups(&pass.workload.dispatch_extent)?);
                }
                RealtimeIblGraphPassKind::Prefilter(slice) => {
                    let command = prefilter_command(&compute_request, slice)?;
                    record_ibl_command(
                        device,
                        encoder,
                        &command,
                        resources.source_sampled(work_slot),
                        IblBakeWgpuOutputBindingResource::StorageTexture2DArray(
                            resources.pmrem_storage_mip(work_slot, u32::from(slice.mip_level))?,
                        ),
                        ibl_pipeline_cache,
                    )?;
                    dispatch_groups.push(command.dispatch_groups);
                }
                RealtimeIblGraphPassKind::ProjectDiffuseSh9 => {
                    let command = sh9_command(&compute_request)?;
                    let output = resources.sh9(work_slot);
                    encoder.clear_buffer(output, 0, None);
                    record_ibl_command(
                        device,
                        encoder,
                        &command,
                        resources.source_sampled(work_slot),
                        IblBakeWgpuOutputBindingResource::StorageBuffer(output),
                        ibl_pipeline_cache,
                    )?;
                    dispatch_groups.push(command.dispatch_groups);
                }
            }
        }
        let timestamp_readback =
            timestamp_recorder.map(|timestamps| timestamps.write_end_and_resolve(encoder));
        Ok(RealtimeIblWgpuRecordResult {
            report: RealtimeIblWgpuRecordReport {
                pass_count: plan.passes.len(),
                dispatch_count: dispatch_groups.len(),
                dispatch_groups,
            },
            timestamp_readback,
        })
    }

    fn timestamp_recorder<'a>(
        timestamps: &'a mut Option<RealtimeIblGpuTimestampRecorder>,
        device: &wgpu::Device,
        gpu_timing_enabled: bool,
    ) -> Option<&'a RealtimeIblGpuTimestampRecorder> {
        if gpu_timing_enabled && timestamps.is_none() {
            *timestamps = RealtimeIblGpuTimestampRecorder::new(device);
        }
        gpu_timing_enabled.then(|| timestamps.as_ref()).flatten()
    }
}

fn prefilter_command(
    request: &IblBakeArtifactRequest,
    slice: super::realtime_ibl_time_slice::RealtimeIblPrefilterDispatchSlice,
) -> Result<IblBakeWgpuCommandPlan, String> {
    ibl_bake_wgpu_prefilter_command_for_slice(request, slice).ok_or_else(|| {
        format!(
            "realtime IBL PMREM slice is outside request bounds: mip={}, first_face={}, face_count={}",
            slice.mip_level, slice.first_face, slice.face_count
        )
    })
}

fn record_ibl_command(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    command: &IblBakeWgpuCommandPlan,
    source: &wgpu::TextureView,
    output: IblBakeWgpuOutputBindingResource<'_>,
    pipeline_cache: &mut IblBakeWgpuPipelineCache,
) -> Result<(), String> {
    let params = create_ibl_bake_wgpu_params_buffer(device, command);
    let pipeline = pipeline_cache.ensure_compute_pipeline(device, command);
    let bind_group = create_ibl_bake_wgpu_bind_group(
        device,
        pipeline_cache.bind_group_layouts(),
        command,
        &params,
        source,
        pipeline_cache.source_sampler(),
        output,
    )?;
    encode_ibl_bake_wgpu_compute_dispatch(encoder, command, &pipeline, &bind_group)?;
    Ok(())
}

fn sh9_command(request: &IblBakeArtifactRequest) -> Result<IblBakeWgpuCommandPlan, String> {
    let plan = ibl_bake_wgpu_command_plan_for_request(request);
    plan.commands
        .into_iter()
        .find(|command| command.kind == IblBakeComputeKernelKind::IrradianceSh9)
        .ok_or_else(|| "realtime IBL request did not produce an SH9 command".to_string())
}

fn fixed_dispatch_groups(extent: &RenderGraphComputeDispatchExtent) -> Result<[u32; 3], String> {
    match extent {
        RenderGraphComputeDispatchExtent::Fixed(groups) => Ok(*groups),
        other => Err(format!(
            "realtime IBL graph requires fixed compute dispatches, got {other:?}"
        )),
    }
}

const fn mip_dimension(base_size: u32, mip_level: u32) -> u32 {
    let shifted = base_size >> mip_level;
    if shifted == 0 {
        1
    } else {
        shifted
    }
}

#[cfg(test)]
mod tests;
