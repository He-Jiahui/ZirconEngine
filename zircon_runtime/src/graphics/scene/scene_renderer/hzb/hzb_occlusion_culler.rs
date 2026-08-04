use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::graphics::resource_limits::HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;
use crate::graphics::scene::scene_renderer::graph_execution::RenderPassMeshCommandLists;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshIndirectArgsReadback, MeshIndirectDrawExecution, MeshPassIndirectDrawExecutions,
};
use crate::graphics::visibility::{
    HzbOcclusionCullReadbackStats, HzbOcclusionCullReport, HzbOcclusionIndirectArgsReadbackSummary,
};
use zr_rhi_wgpu::{GpuReadbackQueue, ReadbackError};

use super::phase_dispatch::{HzbOcclusionPhaseDispatch, HzbOcclusionPhaseDispatchSummary};

pub(crate) const HZB_OCCLUSION_CULL_PIPELINE_LABEL: &str = "zircon-hzb-occlusion-cull-pipeline";
pub(crate) const HZB_OCCLUSION_CULL_WORKGROUP_SIZE: [u32; 3] = [64, 1, 1];
pub(crate) const HZB_OCCLUSION_COMPACTION_METADATA_RESOURCE: &str =
    "mesh.indirect-compaction-metadata";
pub(crate) const HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE: &str =
    "mesh.compacted-indirect-args";
pub(crate) const HZB_OCCLUSION_DRAW_COUNT_RESOURCE: &str = "mesh.indirect-draw-count";
pub(crate) const HZB_OCCLUSION_INDIRECT_ARGS_RESOURCE: &str = "mesh.indirect-args";
pub(crate) const HZB_OCCLUSION_STATS_RESOURCE: &str = "visibility.hzb-occlusion-stats";
pub(crate) const HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE: &str =
    "mesh.visible-instance-index";

const HZB_OCCLUSION_DEPTH_BIAS: f32 = 0.001;
const HZB_OCCLUSION_RADIUS_SCALE: f32 = 1.25;
const HZB_OCCLUSION_CULL_PARAMS_BUFFER_SIZE: u64 =
    std::mem::size_of::<HzbOcclusionCullParams>() as u64;
pub(crate) const HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE: u64 =
    std::mem::size_of::<HzbOcclusionCullGpuStats>() as u64;
const MAX_PENDING_HZB_STATS_READBACKS: usize = 4;
const MAX_PENDING_HZB_INDIRECT_ARGS_READBACKS: usize = 4;
const HZB_OCCLUSION_CULL_SHADER: &str = concat!(
    include_str!("../mesh/shaders/zr_gpu_scene.wgsl"),
    "\n",
    include_str!("shaders/zr_hzb.wgsl"),
    "\n",
    include_str!("shaders/hzb_occlusion_cull.wgsl"),
);

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct HzbOcclusionCullParams {
    counts: [u32; 4],
    values: [f32; 4],
}

impl HzbOcclusionCullParams {
    fn new(args_count: u32) -> Self {
        Self {
            counts: [args_count, 0, 0, 0],
            values: [
                HZB_OCCLUSION_DEPTH_BIAS,
                HZB_OCCLUSION_RADIUS_SCALE,
                0.0,
                0.0,
            ],
        }
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct HzbOcclusionCullGpuStats {
    tested_arg_count: u32,
    tested_instance_count: u32,
    culled_arg_count: u32,
    culled_instance_count: u32,
}

impl HzbOcclusionCullGpuStats {
    fn readback_stats(self) -> HzbOcclusionCullReadbackStats {
        HzbOcclusionCullReadbackStats::new(
            self.tested_arg_count,
            self.tested_instance_count,
            self.culled_arg_count,
            self.culled_instance_count,
        )
    }
}

pub(crate) struct HzbOcclusionCuller {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
    params_buffer: wgpu::Buffer,
    stats_buffer: wgpu::Buffer,
    stats_readbacks: Arc<Mutex<HzbStatsReadbackQueue>>,
    pending_indirect_args: Mutex<VecDeque<PendingHzbIndirectArgs>>,
}

struct HzbStatsReadbackSlot {
    source_frame_index: u64,
    stats: Option<HzbOcclusionCullReadbackStats>,
}

#[derive(Default)]
struct HzbStatsReadbackQueue {
    slots: VecDeque<HzbStatsReadbackSlot>,
    dropped_count: u32,
}

struct PendingHzbIndirectArgs {
    source_frame_index: u64,
    readbacks: Vec<MeshIndirectArgsReadback>,
}

impl HzbStatsReadbackQueue {
    fn reserve(&mut self, source_frame_index: u64) -> bool {
        if self.slots.len() >= MAX_PENDING_HZB_STATS_READBACKS {
            self.dropped_count = self.dropped_count.saturating_add(1);
            return false;
        }
        self.slots.push_back(HzbStatsReadbackSlot {
            source_frame_index,
            stats: None,
        });
        true
    }

    fn cancel(&mut self, source_frame_index: u64) {
        if let Some(index) = self
            .slots
            .iter()
            .position(|slot| slot.source_frame_index == source_frame_index && slot.stats.is_none())
        {
            self.slots.remove(index);
        }
    }

    fn complete(&mut self, source_frame_index: u64, stats: HzbOcclusionCullReadbackStats) {
        if let Some(slot) = self
            .slots
            .iter_mut()
            .find(|slot| slot.source_frame_index == source_frame_index && slot.stats.is_none())
        {
            slot.stats = Some(stats);
        }
    }

    fn fail(&mut self, source_frame_index: u64) {
        if self
            .slots
            .iter()
            .any(|slot| slot.source_frame_index == source_frame_index && slot.stats.is_none())
        {
            self.cancel(source_frame_index);
            self.dropped_count = self.dropped_count.saturating_add(1);
        }
    }

    fn record_drop(&mut self) {
        self.dropped_count = self.dropped_count.saturating_add(1);
    }

    fn pop_ready(&mut self) -> Option<(u64, HzbOcclusionCullReadbackStats)> {
        let ready = self.slots.front()?.stats?;
        let source_frame_index = self
            .slots
            .pop_front()
            .expect("ready HZB stats slot must remain queued")
            .source_frame_index;
        Some((source_frame_index, ready))
    }

    fn diagnostics(&self, current_frame_index: Option<u64>) -> (u32, u32, Option<u64>) {
        let pending_count = self
            .slots
            .iter()
            .filter(|slot| slot.stats.is_none())
            .count()
            .try_into()
            .unwrap_or(u32::MAX);
        let oldest_pending_age_frames = current_frame_index.and_then(|current_frame_index| {
            self.slots
                .iter()
                .find(|slot| slot.stats.is_none())
                .map(|slot| current_frame_index.saturating_sub(slot.source_frame_index))
        });
        (pending_count, self.dropped_count, oldest_pending_age_frames)
    }
}

pub(crate) fn hzb_occlusion_supported_by_limits(limits: &wgpu::Limits) -> bool {
    limits.max_storage_buffers_per_shader_stage
        >= HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
}

impl HzbOcclusionCuller {
    pub(crate) fn new(
        device: &wgpu::Device,
        scene_layout: &wgpu::BindGroupLayout,
        gpu_scene_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group_layout = create_hzb_occlusion_bind_group_layout(device);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-hzb-occlusion-cull-shader"),
            source: wgpu::ShaderSource::Wgsl(HZB_OCCLUSION_CULL_SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-hzb-occlusion-cull-layout"),
            bind_group_layouts: &[
                Some(scene_layout),
                Some(&bind_group_layout),
                None,
                Some(gpu_scene_layout),
            ],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(HZB_OCCLUSION_CULL_PIPELINE_LABEL),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-hzb-occlusion-cull-params"),
            size: HZB_OCCLUSION_CULL_PARAMS_BUFFER_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let stats_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-hzb-occlusion-cull-stats"),
            size: HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        Self {
            bind_group_layout,
            pipeline,
            params_buffer,
            stats_buffer,
            stats_readbacks: Arc::new(Mutex::new(HzbStatsReadbackQueue::default())),
            pending_indirect_args: Mutex::new(VecDeque::new()),
        }
    }

    pub(crate) fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        scene_bind_group: &wgpu::BindGroup,
        gpu_scene_bind_group: &wgpu::BindGroup,
        previous_hzb_view: &wgpu::TextureView,
        mesh_draw_lists: RenderPassMeshCommandLists<'_>,
        history_available: bool,
    ) -> HzbOcclusionCullReport {
        let candidate_arg_count = mesh_draw_lists.occlusion_cull_candidate_arg_count();
        let candidate_instance_count = mesh_draw_lists.occlusion_cull_candidate_instance_count();
        if candidate_arg_count == 0 {
            return HzbOcclusionCullReport::single_frame_reproject(0, 0, 0, 0, history_available);
        }

        self.clear_stats(queue);

        let mut dispatch_summary = HzbOcclusionPhaseDispatchSummary::default();
        for execution in mesh_draw_lists
            .hzb_occlusion_indirect_executions()
            .into_iter()
            .flatten()
        {
            let Some(phase_dispatch) = HzbOcclusionPhaseDispatch::new(execution) else {
                continue;
            };
            execution
                .compaction_resources()
                .encode_clear_outputs(encoder);
            self.execute_indirect_args_buffer(
                device,
                encoder,
                scene_bind_group,
                gpu_scene_bind_group,
                previous_hzb_view,
                &phase_dispatch,
            );
            execution.mark_compaction_ready_for_replay();
            dispatch_summary.record_phase(&phase_dispatch);
        }
        HzbOcclusionCullReport::single_frame_reproject(
            candidate_arg_count,
            candidate_instance_count,
            dispatch_summary.dispatch_group_count(),
            dispatch_summary.dispatched_phase_count(),
            history_available,
        )
    }

    fn clear_stats(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.stats_buffer,
            0,
            bytemuck::bytes_of(&HzbOcclusionCullGpuStats::zeroed()),
        );
    }

    fn execute_indirect_args_buffer(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        scene_bind_group: &wgpu::BindGroup,
        gpu_scene_bind_group: &wgpu::BindGroup,
        previous_hzb_view: &wgpu::TextureView,
        phase_dispatch: &HzbOcclusionPhaseDispatch<'_>,
    ) {
        let execution = phase_dispatch.execution();
        self.encode_params_upload(device, encoder, phase_dispatch.args_count());
        let bind_group = self.create_bind_group_for_execution(device, previous_hzb_view, execution);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("zircon-hzb-occlusion-cull"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_bind_group(1, &bind_group, &[]);
        pass.set_bind_group(3, gpu_scene_bind_group, &[]);
        pass.dispatch_workgroups(phase_dispatch.dispatch_group_count(), 1, 1);
    }

    fn encode_params_upload(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        args_count: u32,
    ) {
        let upload = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-hzb-occlusion-cull-params-upload"),
            contents: bytemuck::bytes_of(&HzbOcclusionCullParams::new(args_count)),
            usage: wgpu::BufferUsages::COPY_SRC,
        });
        encoder.copy_buffer_to_buffer(
            &upload,
            0,
            &self.params_buffer,
            0,
            HZB_OCCLUSION_CULL_PARAMS_BUFFER_SIZE,
        );
    }

    pub(crate) fn request_frame_readbacks(
        &self,
        queue: &mut GpuReadbackQueue,
        indirect_draws: &MeshPassIndirectDrawExecutions,
        source_frame_index: u64,
        capture_indirect_args: bool,
    ) -> Result<(), ReadbackError> {
        let stats_readbacks = Arc::clone(&self.stats_readbacks);
        if stats_readbacks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reserve(source_frame_index)
        {
            if let Err(error) = queue.request_readback_external(
                "hzb-occlusion.stats",
                &self.stats_buffer,
                0..HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE,
                Box::new(move |result| {
                    let mut readbacks = stats_readbacks
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    let Some(stats) = result
                        .ok()
                        .and_then(|bytes| decode_gpu_stats(&bytes))
                        .map(HzbOcclusionCullGpuStats::readback_stats)
                    else {
                        readbacks.fail(source_frame_index);
                        return;
                    };
                    readbacks.complete(source_frame_index, stats);
                }),
            ) {
                self.stats_readbacks
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .cancel(source_frame_index);
                return Err(error);
            }
        }
        if capture_indirect_args {
            let can_request_indirect_args = {
                let pending = self
                    .pending_indirect_args
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                pending.len() < MAX_PENDING_HZB_INDIRECT_ARGS_READBACKS
            };
            if !can_request_indirect_args {
                self.stats_readbacks
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .record_drop();
                return Ok(());
            }
            let indirect_args = indirect_draws
                .request_hzb_occlusion_args_readbacks(queue, "hzb-occlusion.indirect-args")?;
            self.pending_indirect_args
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push_back(PendingHzbIndirectArgs {
                    source_frame_index,
                    readbacks: indirect_args,
                });
        }
        Ok(())
    }

    pub(crate) fn record_skipped_readback(&self) {
        self.stats_readbacks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .record_drop();
    }

    pub(crate) fn collect_last_readback_stats(
        &self,
    ) -> Option<(u64, HzbOcclusionCullReadbackStats)> {
        self.stats_readbacks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pop_ready()
    }

    pub(crate) fn with_readback_queue_diagnostics(
        &self,
        report: HzbOcclusionCullReport,
        current_frame_index: Option<u64>,
    ) -> HzbOcclusionCullReport {
        let readbacks = self
            .stats_readbacks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (pending_count, dropped_count, oldest_pending_age_frames) =
            readbacks.diagnostics(current_frame_index);
        report.with_readback_queue_diagnostics(
            pending_count,
            dropped_count,
            oldest_pending_age_frames,
        )
    }

    pub(crate) fn collect_last_indirect_args_summary(
        &self,
    ) -> Option<(u64, HzbOcclusionIndirectArgsReadbackSummary)> {
        let mut pending = self
            .pending_indirect_args
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !pending.front().is_some_and(|pending| {
            pending
                .readbacks
                .iter()
                .all(MeshIndirectArgsReadback::is_ready)
        }) {
            return None;
        }
        let mut summary = HzbOcclusionIndirectArgsReadbackSummary::default();
        let pending = pending.pop_front()?;
        for readback in pending.readbacks {
            let snapshot = readback.collect()?;
            summary.add_assign(HzbOcclusionIndirectArgsReadbackSummary::new(
                snapshot.args_count(),
                snapshot.compacted_draw_count(),
                snapshot.zero_instance_arg_count(),
                snapshot.remaining_instance_count(),
            ));
        }
        Some((pending.source_frame_index, summary))
    }

    pub(crate) fn stats_buffer(&self) -> &wgpu::Buffer {
        &self.stats_buffer
    }

    fn create_bind_group_for_execution(
        &self,
        device: &wgpu::Device,
        previous_hzb_view: &wgpu::TextureView,
        execution: &MeshIndirectDrawExecution,
    ) -> wgpu::BindGroup {
        let compaction_resources = execution.compaction_resources();
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-hzb-occlusion-cull-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(previous_hzb_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: execution.args_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: compaction_resources.metadata_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: compaction_resources
                        .visible_instance_index_buffer()
                        .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: compaction_resources.draw_count_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: compaction_resources
                        .compacted_indirect_args_buffer()
                        .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: self.stats_buffer.as_entire_binding(),
                },
            ],
        })
    }
}

fn decode_gpu_stats(bytes: &[u8]) -> Option<HzbOcclusionCullGpuStats> {
    let bytes = bytes.get(..HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE as usize)?;
    Some(HzbOcclusionCullGpuStats {
        tested_arg_count: decode_u32(bytes, 0),
        tested_instance_count: decode_u32(bytes, 4),
        culled_arg_count: decode_u32(bytes, 8),
        culled_instance_count: decode_u32(bytes, 12),
    })
}

fn decode_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn create_hzb_occlusion_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-hzb-occlusion-cull-bind-group-layout"),
        entries: &hzb_occlusion_bind_group_layout_entries(),
    })
}

fn hzb_occlusion_bind_group_layout_entries() -> [wgpu::BindGroupLayoutEntry; 8] {
    [
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 4,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 5,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 6,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 7,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

#[cfg(test)]
fn hzb_occlusion_storage_buffer_binding_count() -> u32 {
    hzb_occlusion_bind_group_layout_entries()
        .iter()
        .filter(|entry| {
            entry.visibility.contains(wgpu::ShaderStages::COMPUTE)
                && matches!(
                    entry.ty,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { .. },
                        ..
                    }
                )
        })
        .count() as u32
}

#[cfg(test)]
mod tests;
