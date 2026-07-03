use std::sync::mpsc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::graphics::resource_limits::HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;
use crate::graphics::scene::scene_renderer::graph_execution::RenderPassMeshCommandLists;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshIndirectDrawExecution;
use crate::graphics::visibility::{HzbOcclusionCullReadbackStats, HzbOcclusionCullReport};

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
    stats_readback_buffer: wgpu::Buffer,
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
        let stats_readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-hzb-occlusion-cull-stats-readback"),
            size: HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            bind_group_layout,
            pipeline,
            params_buffer,
            stats_buffer,
            stats_readback_buffer,
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
        if dispatch_summary.dispatched_phase_count() > 0 {
            self.copy_stats_to_readback(encoder);
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

    fn copy_stats_to_readback(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_buffer_to_buffer(
            &self.stats_buffer,
            0,
            &self.stats_readback_buffer,
            0,
            HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE,
        );
    }

    pub(crate) fn collect_last_readback_stats(
        &self,
        device: &wgpu::Device,
    ) -> Option<HzbOcclusionCullReadbackStats> {
        let slice = self.stats_readback_buffer.slice(..);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        receiver.recv().ok()?.ok()?;

        let mapped = slice.get_mapped_range();
        let gpu_stats = *bytemuck::from_bytes::<HzbOcclusionCullGpuStats>(
            &mapped[..HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE as usize],
        );
        drop(mapped);
        self.stats_readback_buffer.unmap();
        Some(gpu_stats.readback_stats())
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
