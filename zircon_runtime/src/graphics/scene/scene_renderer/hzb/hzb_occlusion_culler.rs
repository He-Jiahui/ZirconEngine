use std::sync::mpsc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::graphics::resource_limits::HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;
use crate::graphics::scene::scene_renderer::graph_execution::RenderPassMeshCommandLists;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshIndirectDrawExecution;
use crate::graphics::visibility::{HzbOcclusionCullReadbackStats, HzbOcclusionCullReport};

pub(crate) const HZB_OCCLUSION_CULL_PIPELINE_LABEL: &str = "zircon-hzb-occlusion-cull-pipeline";
pub(crate) const HZB_OCCLUSION_CULL_WORKGROUP_SIZE: [u32; 3] = [64, 1, 1];
pub(crate) const HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE: &str =
    "mesh.compacted-indirect-args";
pub(crate) const HZB_OCCLUSION_DRAW_COUNT_RESOURCE: &str = "mesh.indirect-draw-count";
pub(crate) const HZB_OCCLUSION_STATS_RESOURCE: &str = "visibility.hzb-occlusion-stats";
pub(crate) const HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE: &str =
    "mesh.visible-instance-index";

const HZB_OCCLUSION_DEPTH_BIAS: f32 = 0.001;
const HZB_OCCLUSION_RADIUS_SCALE: f32 = 1.25;
const HZB_OCCLUSION_CULL_PARAMS_BUFFER_SIZE: u64 =
    std::mem::size_of::<HzbOcclusionCullParams>() as u64;
const HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE: u64 =
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

        let mut dispatched_phase_count = 0u32;
        for execution in mesh_draw_lists
            .hzb_occlusion_indirect_executions()
            .into_iter()
            .flatten()
        {
            let args_count = execution.args_count();
            if args_count == 0 {
                continue;
            }
            execution
                .compaction_resources()
                .encode_clear_outputs(encoder);
            self.execute_indirect_args_buffer(
                device,
                encoder,
                scene_bind_group,
                gpu_scene_bind_group,
                previous_hzb_view,
                execution,
            );
            execution.mark_compaction_ready_for_replay();
            dispatched_phase_count += 1;
        }
        if dispatched_phase_count > 0 {
            self.copy_stats_to_readback(encoder);
        }

        HzbOcclusionCullReport::single_frame_reproject(
            candidate_arg_count,
            candidate_instance_count,
            dispatch_group_count(candidate_arg_count),
            dispatched_phase_count,
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
        execution: &MeshIndirectDrawExecution,
    ) {
        let args_count = execution.args_count();
        if args_count == 0 {
            return;
        }

        self.encode_params_upload(device, encoder, args_count);
        let bind_group = self.create_bind_group_for_execution(device, previous_hzb_view, execution);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("zircon-hzb-occlusion-cull"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_bind_group(1, &bind_group, &[]);
        pass.set_bind_group(3, gpu_scene_bind_group, &[]);
        pass.dispatch_workgroups(dispatch_group_count(args_count), 1, 1);
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
        entries: &[
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
        ],
    })
}

fn dispatch_group_count(args_count: u32) -> u32 {
    if args_count == 0 {
        0
    } else {
        args_count.div_ceil(HZB_OCCLUSION_CULL_WORKGROUP_SIZE[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{mpsc, Arc};

    use crate::core::framework::render::{RenderCapabilitySummary, RenderPhase};
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::scene::gpu_scene::{
        GpuInstanceData, GpuPrimitiveData, GpuScene, GpuSceneEntry, GPU_PRIMITIVE_FLAG_VISIBLE,
        GPU_SCENE_INVALID_PAYLOAD_SLOT,
    };
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        DrawInstanceSource, MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle,
        MeshIndirectArgsSnapshot, MeshIndirectDrawExecution, MeshPassPipelineKind,
        MeshPipelineVariantId, INDEXED_INDIRECT_ARGS_STRIDE_BYTES,
        INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES,
    };
    use crate::graphics::scene::scene_renderer::mesh::IndexedIndirectArgs;
    use crate::graphics::scene::scene_renderer::primitives::SceneUniform;

    const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
    const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
    const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;
    const TEST_WALL_DEPTH: f32 = 0.2;
    const TEST_VISIBLE_INSTANCE_Z: f32 = 0.1;
    const TEST_HIDDEN_INSTANCE_Z: f32 = 0.9;

    #[test]
    fn hzb_occlusion_limit_gate_requires_pipeline_storage_buffer_capacity() {
        assert!(hzb_occlusion_supported_by_limits(&wgpu::Limits {
            max_storage_buffers_per_shader_stage:
                HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
            ..wgpu::Limits::default()
        }));
        assert!(!hzb_occlusion_supported_by_limits(&wgpu::Limits {
            max_storage_buffers_per_shader_stage:
                HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE - 1,
            ..wgpu::Limits::default()
        }));
    }

    #[test]
    fn hzb_occlusion_culls_fully_hidden_indirect_args_on_wgpu() {
        let Some(backend) = test_backend() else {
            return;
        };
        let device = &backend.device;
        let queue = &backend.queue;
        let (scene_layout, _scene_uniform_buffer, scene_bind_group) = test_scene_bind_group(device);
        let mut gpu_scene = test_gpu_scene(device);
        let hidden =
            sync_occlusion_test_entry(device, &mut gpu_scene, 0x1000_0001, TEST_HIDDEN_INSTANCE_Z);
        let visible =
            sync_occlusion_test_entry(device, &mut gpu_scene, 0x1000_0002, TEST_VISIBLE_INSTANCE_Z);
        let upload = gpu_scene.flush_updates(queue);
        assert!(upload.uploaded_bytes > 0);

        let hzb = test_hzb_texture(device, queue, TEST_WALL_DEPTH);
        let culler =
            HzbOcclusionCuller::new(device, &scene_layout, gpu_scene.scene_bind_group_layout());
        let execution = MeshIndirectDrawExecution::build(
            device,
            "zircon-test-hzb-occlusion-indirect-execution",
            &[
                test_mesh_command(hidden.first_instance_index),
                test_mesh_command(visible.first_instance_index),
            ],
            &gpu_driven_capabilities(),
        )
        .expect("test indirect execution");
        let args_readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-hzb-occlusion-indirect-args-readback"),
            size: indirect_args_byte_size(execution.args_count()),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let draw_count_readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-hzb-occlusion-draw-count-readback"),
            size: INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        culler.clear_stats(queue);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-test-hzb-occlusion-cull"),
        });
        execution
            .compaction_resources()
            .encode_clear_outputs(&mut encoder);
        culler.execute_indirect_args_buffer(
            device,
            &mut encoder,
            &scene_bind_group,
            gpu_scene.scene_bind_group(),
            &hzb.view,
            &execution,
        );
        culler.copy_stats_to_readback(&mut encoder);
        encoder.copy_buffer_to_buffer(
            execution
                .compaction_resources()
                .compacted_indirect_args_buffer(),
            0,
            &args_readback,
            0,
            indirect_args_byte_size(execution.args_count()),
        );
        encoder.copy_buffer_to_buffer(
            execution.compaction_resources().draw_count_buffer(),
            0,
            &draw_count_readback,
            0,
            INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES,
        );
        queue.submit([encoder.finish()]);

        let stats = culler
            .collect_last_readback_stats(device)
            .expect("hzb occlusion stats readback");
        let snapshot =
            collect_indirect_args_snapshot(device, &args_readback, execution.args_count())
                .expect("indirect args readback");
        let draw_count = collect_u32(device, &draw_count_readback).expect("draw-count readback");

        assert_eq!(stats.tested_arg_count, 2);
        assert_eq!(stats.tested_instance_count, 2);
        assert_eq!(stats.culled_arg_count, 1);
        assert_eq!(stats.culled_instance_count, 1);
        assert_eq!(draw_count, 1);
        assert_eq!(snapshot.args_count(), 2);
        assert_eq!(snapshot.zero_instance_arg_count(), 1);
        assert_eq!(snapshot.remaining_instance_count(), 1);
    }

    #[test]
    fn hzb_occlusion_culler_shader_declares_expected_bindings() {
        assert!(HZB_OCCLUSION_CULL_SHADER.contains("@group(0) @binding(0) var<uniform> scene"));
        assert!(HZB_OCCLUSION_CULL_SHADER.contains("@group(1) @binding(0) var previous_hzb"));
        assert!(HZB_OCCLUSION_CULL_SHADER
            .contains("@group(1) @binding(2) var<storage, read> source_indirect_args"));
        assert!(HZB_OCCLUSION_CULL_SHADER
            .contains("@group(1) @binding(3) var<storage, read> compaction_metadata"));
        assert!(HZB_OCCLUSION_CULL_SHADER
            .contains("@group(1) @binding(4) var<storage, read_write> visible_instance_indices"));
        assert!(HZB_OCCLUSION_CULL_SHADER
            .contains("@group(1) @binding(5) var<storage, read_write> draw_counts"));
        assert!(HZB_OCCLUSION_CULL_SHADER
            .contains("@group(1) @binding(6) var<storage, read_write> compacted_indirect_args"));
        assert!(HZB_OCCLUSION_CULL_SHADER
            .contains("@group(1) @binding(7) var<storage, read_write> occlusion_stats"));
        assert!(HZB_OCCLUSION_CULL_SHADER.contains("atomicAdd(&occlusion_stats.culled_arg_count"));
        assert!(HZB_OCCLUSION_CULL_SHADER.contains("atomicAdd(&draw_counts"));
        assert!(HZB_OCCLUSION_CULL_SHADER
            .contains("@group(3) @binding(0) var<storage, read> zr_primitive_data"));
        assert!(HZB_OCCLUSION_CULL_SHADER
            .contains("@group(3) @binding(5) var<storage, read> zr_visible_instance_remap"));
        assert!(HZB_OCCLUSION_CULL_SHADER.contains("@compute @workgroup_size(64, 1, 1)"));
    }

    #[test]
    fn hzb_occlusion_dispatch_groups_cover_indirect_args() {
        assert_eq!(dispatch_group_count(0), 0);
        assert_eq!(dispatch_group_count(1), 1);
        assert_eq!(dispatch_group_count(64), 1);
        assert_eq!(dispatch_group_count(65), 2);
    }

    #[test]
    fn hzb_occlusion_gpu_stats_remains_copy_aligned() {
        assert_eq!(HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE, 16);
    }

    #[test]
    fn hzb_occlusion_uploads_phase_params_in_encoder_order() {
        let source = include_str!("hzb_occlusion_culler.rs");

        assert!(source.contains("zircon-hzb-occlusion-cull-params-upload"));
        assert!(source.contains("encoder.copy_buffer_to_buffer("));
        assert!(!source.contains("bytemuck::bytes_of(&HzbOcclusionCullParams::new(args_count)),\n            );\n            let bind_group"));
    }

    #[test]
    fn hzb_occlusion_culler_clears_compaction_outputs_before_culling_dispatch() {
        let source = include_str!("hzb_occlusion_culler.rs");
        let clear_index = source
            .find("execution.compaction_resources().encode_clear_outputs(encoder);")
            .expect("phase compaction output clear");
        let dispatch_index = source
            .find("self.execute_indirect_args_buffer(")
            .expect("phase hzb cull dispatch");

        assert!(clear_index < dispatch_index);
        assert!(source.contains("HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE"));
        assert!(source.contains("HZB_OCCLUSION_DRAW_COUNT_RESOURCE"));
        assert!(source.contains("HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE"));
    }

    fn test_backend() -> Option<RenderBackend> {
        RenderBackend::new_offscreen()
            .inspect_err(|error| eprintln!("skipping hzb occlusion wgpu test: {error:?}"))
            .ok()
    }

    fn test_scene_bind_group(
        device: &wgpu::Device,
    ) -> (wgpu::BindGroupLayout, wgpu::Buffer, wgpu::BindGroup) {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-scene-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-test-scene-uniform"),
            contents: bytemuck::bytes_of(&test_scene_uniform()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-test-scene-bind-group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        (layout, uniform_buffer, bind_group)
    }

    fn test_scene_uniform() -> SceneUniform {
        SceneUniform {
            view_proj: identity_matrix(),
            view_proj_unjittered: identity_matrix(),
            inverse_view_proj: identity_matrix(),
            ambient_color: [0.0, 0.0, 0.0, 1.0],
            previous_view_proj_unjittered: identity_matrix(),
            motion_params: [0.0, 0.0, 0.0, 0.0],
            jitter_params: [0.0, 0.0, 0.0, 0.0],
        }
    }

    fn test_gpu_scene(device: &wgpu::Device) -> GpuScene {
        GpuScene::new(
            device,
            test_skinned_joint_palette_buffer(device),
            test_skinned_joint_palette_min_binding_size(),
        )
    }

    fn test_skinned_joint_palette_buffer(device: &wgpu::Device) -> Arc<wgpu::Buffer> {
        Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-empty-skinned-joint-palette-buffer"),
            size: test_skinned_joint_palette_min_binding_size().get(),
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        }))
    }

    fn test_skinned_joint_palette_min_binding_size() -> wgpu::BufferSize {
        wgpu::BufferSize::new(
            TEST_SKINNED_JOINT_MATRIX_COUNT * TEST_SKINNED_JOINT_MATRIX_BYTES
                + TEST_SKINNED_JOINT_PARAMS_BYTES,
        )
        .expect("test skinned joint palette uniform size is non-zero")
    }

    fn sync_occlusion_test_entry(
        device: &wgpu::Device,
        scene: &mut GpuScene,
        stable_instance_key: u64,
        translate_z: f32,
    ) -> GpuSceneEntry {
        let entry = scene.register(device, stable_instance_key, 1);
        scene.write_primitive(entry, test_primitive_data());
        scene.write_instances(entry, &[test_instance_data(translate_z)]);
        entry
    }

    fn test_primitive_data() -> GpuPrimitiveData {
        GpuPrimitiveData {
            bounds_center: [0.0, 0.0, 0.0],
            bounds_radius: 0.01,
            tint: [1.0, 1.0, 1.0, 1.0],
            shadow_params: [0.0, 0.5, 1.0, 0.0],
            motion_params: [0.0, 0.0, 0.0, 0.0],
            flags: GPU_PRIMITIVE_FLAG_VISIBLE,
            first_instance_index: u32::MAX,
            instance_count: u32::MAX,
            payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
        }
    }

    fn test_instance_data(translate_z: f32) -> GpuInstanceData {
        let mut world_from_local = identity_matrix();
        world_from_local[3][2] = translate_z;
        GpuInstanceData {
            world_from_local,
            prev_world_from_local: world_from_local,
            primitive_index: u32::MAX,
            flags: 0,
            payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
            _pad0: 0,
        }
    }

    struct TestHzbTexture {
        _texture: wgpu::Texture,
        view: wgpu::TextureView,
    }

    fn test_hzb_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        furthest_depth: f32,
    ) -> TestHzbTexture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-test-hzb-furthest"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::bytes_of(&furthest_depth),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(std::mem::size_of::<f32>() as u32),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        TestHzbTexture {
            _texture: texture,
            view,
        }
    }

    fn collect_indirect_args_snapshot(
        device: &wgpu::Device,
        buffer: &wgpu::Buffer,
        args_count: u32,
    ) -> Option<MeshIndirectArgsSnapshot> {
        let byte_size = indirect_args_byte_size(args_count);
        let slice = buffer.slice(..);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        receiver.recv().ok()?.ok()?;

        let mapped = slice.get_mapped_range();
        let args =
            bytemuck::cast_slice::<u8, IndexedIndirectArgs>(&mapped[..byte_size as usize]).to_vec();
        drop(mapped);
        buffer.unmap();
        Some(MeshIndirectArgsSnapshot::from_args(args))
    }

    fn collect_u32(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Option<u32> {
        let slice = buffer.slice(..);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        receiver.recv().ok()?.ok()?;

        let mapped = slice.get_mapped_range();
        let value =
            *bytemuck::from_bytes::<u32>(&mapped[..INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES as usize]);
        drop(mapped);
        buffer.unmap();
        Some(value)
    }

    fn indirect_args_byte_size(args_count: u32) -> wgpu::BufferAddress {
        u64::from(args_count) * INDEXED_INDIRECT_ARGS_STRIDE_BYTES
    }

    fn test_mesh_command(first_instance: u32) -> MeshDrawCommand {
        MeshDrawCommand::new(
            RenderPhase::Opaque3d,
            MeshPassPipelineKind::Base,
            default_pipeline_key(),
            MeshPipelineVariantId::new(1),
            u64::from(first_instance),
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: first_instance,
                instance_count: 1,
            },
            MeshGeometryHandle::test(7),
            MeshDrawArgs::DirectIndexed {
                first_index: 0,
                index_count: 36,
                first_instance,
                instance_count: 1,
            },
        )
    }

    fn gpu_driven_capabilities() -> RenderCapabilitySummary {
        RenderCapabilitySummary {
            supports_indirect_draw: true,
            supports_multi_draw_indirect: true,
            supports_indirect_first_instance: true,
            ..RenderCapabilitySummary::default()
        }
    }

    fn identity_matrix() -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}
