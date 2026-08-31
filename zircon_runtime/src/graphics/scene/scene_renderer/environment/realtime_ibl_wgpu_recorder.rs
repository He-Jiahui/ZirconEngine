use std::time::Instant;

use crate::core::framework::render::{
    IblBakeArtifactContents, IblBakeArtifactDescriptor, IblBakeArtifactRequest, ProceduralSkyParams,
};
use crate::render_graph::RenderGraphComputeDispatchExtent;

use super::ibl_bake_shader_plan::ibl_bake_irradiance_sh9_kernel_plan;
use super::ibl_bake_wgpu_binding::{
    create_ibl_bake_wgpu_bind_group, create_ibl_bake_wgpu_params_buffer,
    IblBakeWgpuOutputBindingResource,
};
use super::ibl_bake_wgpu_command_plan::{
    ibl_bake_wgpu_command_plan_for_runtime_kernel, ibl_bake_wgpu_prefilter_command_for_slice,
    IblBakeWgpuCommandPlan,
};
use super::ibl_bake_wgpu_dispatch::encode_ibl_bake_wgpu_compute_dispatch;
use super::ibl_bake_wgpu_pipeline_cache::IblBakeWgpuPipelineCache;
use super::realtime_ibl_capture_wgpu::RealtimeIblCaptureWgpuPipelines;
use super::realtime_ibl_gpu_resources::RealtimeIblGpuResources;
use super::realtime_ibl_gpu_timestamps::{
    RealtimeIblGpuTimestampReadback, RealtimeIblGpuTimestampRecorder,
};
use super::realtime_ibl_graph_plan::{
    RealtimeIblGraphPass, RealtimeIblGraphPassKind, RealtimeIblGraphPlan,
};
use super::realtime_ibl_time_slice::{IblRealtimeBufferSlot, RealtimeIblPrefilterDispatchSlice};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblWgpuRecordReport {
    pub pass_count: usize,
    pub dispatch_count: usize,
    pub dispatch_groups: Vec<[u32; 3]>,
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
}

pub(in crate::graphics) struct RealtimeIblWgpuRecorder {
    capture: RealtimeIblCaptureWgpuPipelines,
    timestamps: Option<RealtimeIblGpuTimestampRecorder>,
    binding_cache: RealtimeIblWgpuBindingCache,
}

pub(in crate::graphics) struct RealtimeIblWgpuRecordResult {
    pub report: RealtimeIblWgpuRecordReport,
    pub timestamp_readback: Option<RealtimeIblGpuTimestampReadback>,
}

#[derive(Default)]
struct RealtimeIblWgpuBindingCache {
    layout: Option<RealtimeIblWgpuBindingCacheLayout>,
    entries: Vec<RealtimeIblWgpuBindingCacheEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RealtimeIblWgpuBindingCacheLayout {
    source_face_size: u32,
    source_mip_count: u32,
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RealtimeIblWgpuBindingCommandKey {
    Prefilter(RealtimeIblPrefilterDispatchSlice),
    ProjectDiffuseSh9,
}

struct RealtimeIblWgpuBindingCacheEntry {
    slot: IblRealtimeBufferSlot,
    key: RealtimeIblWgpuBindingCommandKey,
    command: IblBakeWgpuCommandPlan,
    _params: wgpu::Buffer,
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
}

#[derive(Default)]
struct RealtimeIblWgpuBindingCacheStats {
    hits: usize,
    misses: usize,
    params_buffer_creations: usize,
    bind_group_creations: usize,
    resets: usize,
    command_plan_creation_micros: u64,
    pipeline_ensure_micros: u64,
    binding_creation_micros: u64,
}

impl RealtimeIblWgpuRecorder {
    pub(in crate::graphics) fn new(device: &wgpu::Device) -> Self {
        Self {
            capture: RealtimeIblCaptureWgpuPipelines::new(device),
            timestamps: None,
            binding_cache: RealtimeIblWgpuBindingCache::default(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics) fn record_graph_plan(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        gpu_timing_enabled: bool,
        cpu_timing_enabled: bool,
        request: &IblBakeArtifactRequest,
        sky: &ProceduralSkyParams,
        plan: &RealtimeIblGraphPlan,
        recording_passes: &[RealtimeIblGraphPass],
        resources: &RealtimeIblGpuResources,
        ibl_pipeline_cache: &mut IblBakeWgpuPipelineCache,
    ) -> Result<RealtimeIblWgpuRecordResult, String> {
        let work_slot = plan.work.slot;
        let compute_request = (*request).with_required_contents(IblBakeArtifactContents::PMREM_SH9);
        let mut dispatch_groups = Vec::with_capacity(recording_passes.len());
        let mut binding_cache_stats = RealtimeIblWgpuBindingCacheStats::default();
        let mut capture_binding_stats = RealtimeIblWgpuBindingCacheStats::default();
        let mut source_mip_binding_stats = RealtimeIblWgpuBindingCacheStats::default();
        binding_cache_stats.resets = usize::from(self.binding_cache.prepare_for_request(request));
        let capture = &self.capture;
        let timestamp_recorder =
            Self::timestamp_recorder(&mut self.timestamps, device, gpu_timing_enabled);
        if let Some(timestamps) = timestamp_recorder {
            timestamps.write_start(encoder);
        }
        for pass in recording_passes {
            match pass.kind {
                RealtimeIblGraphPassKind::CaptureSky(faces) => {
                    let creation_stats = capture.record_capture(
                        device,
                        encoder,
                        sky,
                        cpu_timing_enabled,
                        request.source_face_size(),
                        faces,
                        resources.source_storage_mip(work_slot, 0)?,
                    );
                    capture_binding_stats.record_creation(creation_stats);
                    dispatch_groups.push(fixed_dispatch_groups(&pass.workload.dispatch_extent)?);
                }
                RealtimeIblGraphPassKind::GenerateSourceMip { mip_level } => {
                    let source_mip = mip_level.saturating_sub(1);
                    let creation_stats = capture.record_downsample_mip(
                        device,
                        encoder,
                        cpu_timing_enabled,
                        mip_dimension(request.source_face_size(), source_mip),
                        mip_dimension(request.source_face_size(), mip_level),
                        resources.source_sampled_mip(work_slot, source_mip)?,
                        resources.source_storage_mip(work_slot, mip_level)?,
                    );
                    source_mip_binding_stats.record_creation(creation_stats);
                    dispatch_groups.push(fixed_dispatch_groups(&pass.workload.dispatch_extent)?);
                }
                RealtimeIblGraphPassKind::Prefilter(slice) => {
                    let graph_dispatch_groups =
                        fixed_dispatch_groups(&pass.workload.dispatch_extent)?;
                    let cache_stats = record_ibl_command(
                        &mut self.binding_cache,
                        device,
                        encoder,
                        work_slot,
                        cpu_timing_enabled,
                        RealtimeIblWgpuBindingCommandKey::Prefilter(slice),
                        graph_dispatch_groups,
                        || prefilter_command(&compute_request, slice),
                        resources.source_sampled(work_slot),
                        IblBakeWgpuOutputBindingResource::StorageTexture2DArray(
                            resources.pmrem_storage_mip(work_slot, u32::from(slice.mip_level))?,
                        ),
                        ibl_pipeline_cache,
                    )?;
                    binding_cache_stats.record(cache_stats);
                    dispatch_groups.push(graph_dispatch_groups);
                }
                RealtimeIblGraphPassKind::ProjectDiffuseSh9 => {
                    let graph_dispatch_groups =
                        fixed_dispatch_groups(&pass.workload.dispatch_extent)?;
                    let output = resources.sh9(work_slot);
                    encoder.clear_buffer(output, 0, None);
                    let cache_stats = record_ibl_command(
                        &mut self.binding_cache,
                        device,
                        encoder,
                        work_slot,
                        cpu_timing_enabled,
                        RealtimeIblWgpuBindingCommandKey::ProjectDiffuseSh9,
                        graph_dispatch_groups,
                        || sh9_command(&compute_request),
                        resources.source_sampled(work_slot),
                        IblBakeWgpuOutputBindingResource::StorageBuffer(output),
                        ibl_pipeline_cache,
                    )?;
                    binding_cache_stats.record(cache_stats);
                    dispatch_groups.push(graph_dispatch_groups);
                }
            }
        }
        let timestamp_readback =
            timestamp_recorder.map(|timestamps| timestamps.write_end_and_resolve(encoder));
        Ok(RealtimeIblWgpuRecordResult {
            report: RealtimeIblWgpuRecordReport {
                pass_count: recording_passes.len(),
                dispatch_count: dispatch_groups.len(),
                dispatch_groups,
                binding_cache_hits: binding_cache_stats.hits,
                binding_cache_misses: binding_cache_stats.misses,
                params_buffer_creations: binding_cache_stats.params_buffer_creations,
                bind_group_creations: binding_cache_stats.bind_group_creations,
                binding_cache_resets: binding_cache_stats.resets,
                command_plan_creation_micros: binding_cache_stats.command_plan_creation_micros,
                pipeline_ensure_micros: binding_cache_stats.pipeline_ensure_micros,
                binding_creation_micros: binding_cache_stats.binding_creation_micros,
                capture_params_buffer_creations: capture_binding_stats.params_buffer_creations,
                capture_bind_group_creations: capture_binding_stats.bind_group_creations,
                capture_binding_creation_micros: capture_binding_stats.binding_creation_micros,
                source_mip_params_buffer_creations: source_mip_binding_stats
                    .params_buffer_creations,
                source_mip_bind_group_creations: source_mip_binding_stats.bind_group_creations,
                source_mip_binding_creation_micros: source_mip_binding_stats
                    .binding_creation_micros,
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

impl RealtimeIblWgpuBindingCache {
    fn prepare_for_request(&mut self, request: &IblBakeArtifactRequest) -> bool {
        let layout = RealtimeIblWgpuBindingCacheLayout {
            source_face_size: request.source_face_size(),
            source_mip_count: request.source_mip_count(),
            pmrem_face_size: request.pmrem_face_size(),
            pmrem_mip_count: request.pmrem_mip_count(),
        };
        let Some(previous_layout) = self.layout else {
            self.layout = Some(layout);
            return false;
        };
        if previous_layout == layout {
            return false;
        }

        let cleared_entries = !self.entries.is_empty();
        self.entries.clear();
        self.layout = Some(layout);
        cleared_entries
    }

    #[allow(clippy::too_many_arguments)]
    fn record(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        slot: IblRealtimeBufferSlot,
        cpu_timing_enabled: bool,
        key: RealtimeIblWgpuBindingCommandKey,
        graph_dispatch_groups: [u32; 3],
        create_command: impl FnOnce() -> Result<IblBakeWgpuCommandPlan, String>,
        source: &wgpu::TextureView,
        output: IblBakeWgpuOutputBindingResource<'_>,
        pipeline_cache: &mut IblBakeWgpuPipelineCache,
    ) -> Result<RealtimeIblWgpuBindingCacheStats, String> {
        if let Some(entry) = self
            .entries
            .iter()
            .find(|entry| entry.slot == slot && entry.key == key)
        {
            validate_graph_command_dispatch_groups(
                key,
                graph_dispatch_groups,
                entry.command.dispatch_groups,
            )?;
            encode_ibl_bake_wgpu_compute_dispatch(
                encoder,
                &entry.command,
                &entry.pipeline,
                &entry.bind_group,
            )?;
            return Ok(RealtimeIblWgpuBindingCacheStats {
                hits: 1,
                ..Default::default()
            });
        }

        let command_plan_started = cpu_timing_enabled.then(Instant::now);
        let command = create_command()?;
        let command_plan_creation_micros = elapsed_micros(command_plan_started);
        validate_graph_command_dispatch_groups(
            key,
            graph_dispatch_groups,
            command.dispatch_groups,
        )?;
        let pipeline_ensure_started = cpu_timing_enabled.then(Instant::now);
        let pipeline = pipeline_cache.ensure_compute_pipeline(device, &command);
        let pipeline_ensure_micros = elapsed_micros(pipeline_ensure_started);
        // Match the capture/source-mip metric boundary: only WGPU resource creation.
        let binding_creation_started = cpu_timing_enabled.then(Instant::now);
        let params = create_ibl_bake_wgpu_params_buffer(device, &command);
        let bind_group = create_ibl_bake_wgpu_bind_group(
            device,
            pipeline_cache.bind_group_layouts(),
            &command,
            &params,
            source,
            pipeline_cache.source_sampler(),
            output,
        )?;
        let binding_creation_micros = elapsed_micros(binding_creation_started);
        self.entries.push(RealtimeIblWgpuBindingCacheEntry {
            slot,
            key,
            command,
            _params: params,
            pipeline,
            bind_group,
        });
        let entry = self
            .entries
            .last()
            .expect("realtime IBL binding cache entry was just inserted");
        encode_ibl_bake_wgpu_compute_dispatch(
            encoder,
            &entry.command,
            &entry.pipeline,
            &entry.bind_group,
        )?;
        Ok(RealtimeIblWgpuBindingCacheStats {
            misses: 1,
            params_buffer_creations: 1,
            bind_group_creations: 1,
            command_plan_creation_micros,
            pipeline_ensure_micros,
            binding_creation_micros,
            ..Default::default()
        })
    }
}

impl RealtimeIblWgpuBindingCacheStats {
    fn record_creation(
        &mut self,
        creation: super::realtime_ibl_capture_wgpu::RealtimeIblWgpuBindingCreationStats,
    ) {
        self.params_buffer_creations += creation.params_buffer_creations;
        self.bind_group_creations += creation.bind_group_creations;
        self.binding_creation_micros = self
            .binding_creation_micros
            .saturating_add(creation.creation_micros);
    }

    fn record(&mut self, other: Self) {
        self.hits += other.hits;
        self.misses += other.misses;
        self.params_buffer_creations += other.params_buffer_creations;
        self.bind_group_creations += other.bind_group_creations;
        self.resets += other.resets;
        self.command_plan_creation_micros = self
            .command_plan_creation_micros
            .saturating_add(other.command_plan_creation_micros);
        self.pipeline_ensure_micros = self
            .pipeline_ensure_micros
            .saturating_add(other.pipeline_ensure_micros);
        self.binding_creation_micros = self
            .binding_creation_micros
            .saturating_add(other.binding_creation_micros);
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

#[allow(clippy::too_many_arguments)]
fn record_ibl_command(
    binding_cache: &mut RealtimeIblWgpuBindingCache,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    slot: IblRealtimeBufferSlot,
    cpu_timing_enabled: bool,
    key: RealtimeIblWgpuBindingCommandKey,
    graph_dispatch_groups: [u32; 3],
    create_command: impl FnOnce() -> Result<IblBakeWgpuCommandPlan, String>,
    source: &wgpu::TextureView,
    output: IblBakeWgpuOutputBindingResource<'_>,
    pipeline_cache: &mut IblBakeWgpuPipelineCache,
) -> Result<RealtimeIblWgpuBindingCacheStats, String> {
    binding_cache.record(
        device,
        encoder,
        slot,
        cpu_timing_enabled,
        key,
        graph_dispatch_groups,
        create_command,
        source,
        output,
        pipeline_cache,
    )
}

fn elapsed_micros(started: Option<Instant>) -> u64 {
    started
        .map(|started| u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX))
        .unwrap_or_default()
}

fn sh9_command(request: &IblBakeArtifactRequest) -> Result<IblBakeWgpuCommandPlan, String> {
    Ok(ibl_bake_wgpu_command_plan_for_runtime_kernel(
        request,
        IblBakeArtifactDescriptor::current_for_runtime_cache_request(request),
        ibl_bake_irradiance_sh9_kernel_plan(request),
    ))
}

fn validate_graph_command_dispatch_groups(
    key: RealtimeIblWgpuBindingCommandKey,
    graph_dispatch_groups: [u32; 3],
    command_dispatch_groups: [u32; 3],
) -> Result<(), String> {
    if graph_dispatch_groups == command_dispatch_groups {
        return Ok(());
    }

    Err(format!(
        "realtime IBL graph/command dispatch mismatch for {key:?}: graph={graph_dispatch_groups:?}, command={command_dispatch_groups:?}"
    ))
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
