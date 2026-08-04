use super::*;
use std::sync::{Arc, mpsc};

use crate::core::framework::render::{RenderCapabilitySummary, RenderPhase};
use crate::graphics::backend::RenderBackend;
use crate::graphics::resource_limits::{
    GPU_SCENE_COMPUTE_STORAGE_BUFFERS_PER_SHADER_STAGE,
    HZB_OCCLUSION_PASS_STORAGE_BUFFERS_PER_SHADER_STAGE,
};
use crate::graphics::scene::gpu_scene::{
    GPU_PRIMITIVE_FLAG_VISIBLE, GPU_SCENE_INVALID_PAYLOAD_SLOT, GpuInstanceData, GpuPrimitiveData,
    GpuScene, GpuSceneEntry,
};
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::mesh::IndexedIndirectArgs;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    DrawInstanceSource, INDEXED_INDIRECT_ARGS_STRIDE_BYTES, INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES,
    MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle, MeshIndirectArgsSnapshot,
    MeshIndirectDrawExecution, MeshPassPipelineKind, MeshPipelineVariantId,
};
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
    if !hzb_occlusion_supported_by_limits(&device.limits()) {
        eprintln!(
            "skipping hzb occlusion wgpu test: device limit max_storage_buffers_per_shader_stage={} is below required {}",
            device.limits().max_storage_buffers_per_shader_stage,
            HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
        );
        return;
    }
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
    let stats_readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-test-hzb-occlusion-stats-readback"),
        size: HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE,
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
    let phase_dispatch = HzbOcclusionPhaseDispatch::new(&execution).expect("test phase dispatch");
    culler.execute_indirect_args_buffer(
        device,
        &mut encoder,
        &scene_bind_group,
        gpu_scene.scene_bind_group(),
        &hzb.view,
        &phase_dispatch,
    );
    encoder.copy_buffer_to_buffer(
        culler.stats_buffer(),
        0,
        &stats_readback,
        0,
        HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE,
    );
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

    let stats = collect_hzb_stats(device, &stats_readback)
        .expect("hzb occlusion stats readback")
        .readback_stats();
    let snapshot = collect_indirect_args_snapshot(device, &args_readback, execution.args_count())
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
    assert!(
        HZB_OCCLUSION_CULL_SHADER
            .contains("@group(1) @binding(2) var<storage, read> source_indirect_args")
    );
    assert!(
        HZB_OCCLUSION_CULL_SHADER
            .contains("@group(1) @binding(3) var<storage, read> compaction_metadata")
    );
    assert!(
        HZB_OCCLUSION_CULL_SHADER
            .contains("@group(1) @binding(4) var<storage, read_write> visible_instance_indices")
    );
    assert!(
        HZB_OCCLUSION_CULL_SHADER
            .contains("@group(1) @binding(5) var<storage, read_write> draw_counts")
    );
    assert!(
        HZB_OCCLUSION_CULL_SHADER
            .contains("@group(1) @binding(6) var<storage, read_write> compacted_indirect_args")
    );
    assert!(
        HZB_OCCLUSION_CULL_SHADER
            .contains("@group(1) @binding(7) var<storage, read_write> occlusion_stats")
    );
    assert!(HZB_OCCLUSION_CULL_SHADER.contains("atomicAdd(&occlusion_stats.culled_arg_count"));
    assert!(HZB_OCCLUSION_CULL_SHADER.contains("atomicAdd(&draw_counts"));
    assert!(
        HZB_OCCLUSION_CULL_SHADER
            .contains("@group(3) @binding(0) var<storage, read> zr_primitive_data")
    );
    assert!(
        HZB_OCCLUSION_CULL_SHADER
            .contains("@group(3) @binding(5) var<storage, read> zr_visible_instance_remap")
    );
    assert!(HZB_OCCLUSION_CULL_SHADER.contains("@compute @workgroup_size(64, 1, 1)"));
}

#[test]
fn hzb_occlusion_limit_gate_matches_pipeline_storage_buffer_layout() {
    assert_eq!(
        hzb_occlusion_storage_buffer_binding_count(),
        HZB_OCCLUSION_PASS_STORAGE_BUFFERS_PER_SHADER_STAGE
    );
    assert_eq!(
        GPU_SCENE_COMPUTE_STORAGE_BUFFERS_PER_SHADER_STAGE
            + hzb_occlusion_storage_buffer_binding_count(),
        HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE
    );
}

#[test]
fn hzb_occlusion_gpu_stats_remains_copy_aligned() {
    assert_eq!(HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE, 16);
}

#[test]
fn hzb_stats_readback_queue_consumes_completed_slots_in_fifo_order() {
    let mut readbacks = HzbStatsReadbackQueue::default();
    assert!(readbacks.reserve(10));
    assert!(readbacks.reserve(11));
    readbacks.complete(11, HzbOcclusionCullReadbackStats::new(11, 0, 0, 0));

    assert_eq!(readbacks.pop_ready(), None);
    assert_eq!(readbacks.diagnostics(Some(15)), (1, 0, Some(5)));

    let first = HzbOcclusionCullReadbackStats::new(10, 0, 0, 0);
    readbacks.complete(10, first);
    assert_eq!(readbacks.pop_ready(), Some((10, first)));
    assert_eq!(
        readbacks.pop_ready(),
        Some((11, HzbOcclusionCullReadbackStats::new(11, 0, 0, 0)))
    );
    assert_eq!(readbacks.diagnostics(Some(15)), (0, 0, None));
}

#[test]
fn hzb_stats_readback_queue_bounds_pending_requests_and_records_drops() {
    let mut readbacks = HzbStatsReadbackQueue::default();
    for source_frame_index in 0..MAX_PENDING_HZB_STATS_READBACKS as u64 {
        assert!(readbacks.reserve(source_frame_index));
    }

    assert!(!readbacks.reserve(MAX_PENDING_HZB_STATS_READBACKS as u64));
    assert_eq!(
        readbacks.diagnostics(Some(MAX_PENDING_HZB_STATS_READBACKS as u64)),
        (
            MAX_PENDING_HZB_STATS_READBACKS as u32,
            1,
            Some(MAX_PENDING_HZB_STATS_READBACKS as u64)
        )
    );

    readbacks.fail(0);
    assert_eq!(
        readbacks.diagnostics(Some(MAX_PENDING_HZB_STATS_READBACKS as u64)),
        (
            (MAX_PENDING_HZB_STATS_READBACKS - 1) as u32,
            2,
            Some((MAX_PENDING_HZB_STATS_READBACKS - 1) as u64)
        )
    );
}

#[test]
fn hzb_stats_readback_queue_records_a_skipped_shared_readback_frame() {
    let mut readbacks = HzbStatsReadbackQueue::default();

    readbacks.record_drop();

    assert_eq!(readbacks.diagnostics(Some(10)), (0, 1, None));
}

#[test]
fn hzb_indirect_args_readback_is_explicitly_gated_from_default_stats_readback() {
    let source = include_str!("../hzb_occlusion_culler.rs");
    let start = source
        .find("pub(crate) fn request_frame_readbacks")
        .expect("HZB async readback entrypoint");
    let request_readbacks = &source[start..];

    assert!(request_readbacks.contains("capture_indirect_args: bool"));
    assert!(request_readbacks.contains("source_frame_index: u64"));
    assert!(request_readbacks.contains("if capture_indirect_args"));
    assert!(request_readbacks.contains("hzb-occlusion.stats"));
    assert!(request_readbacks.contains("PendingHzbIndirectArgs"));
    assert!(request_readbacks.contains("source_frame_index,"));
}

#[test]
fn hzb_occlusion_uploads_phase_params_in_encoder_order() {
    let source = include_str!("../hzb_occlusion_culler.rs");

    assert!(source.contains("zircon-hzb-occlusion-cull-params-upload"));
    assert!(source.contains("encoder.copy_buffer_to_buffer("));
    assert!(!source.contains("bytemuck::bytes_of(&HzbOcclusionCullParams::new(args_count)),\n            );\n            let bind_group"));
}

#[test]
fn hzb_occlusion_culler_clears_compaction_outputs_before_culling_dispatch() {
    let source = include_str!("../hzb_occlusion_culler.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation source");
    let clear_index = implementation
        .find(".encode_clear_outputs(encoder);")
        .expect("phase compaction output clear");
    let dispatch_index = implementation
        .find("self.execute_indirect_args_buffer(")
        .expect("phase hzb cull dispatch");

    assert!(clear_index < dispatch_index);
    assert!(implementation.contains("HZB_OCCLUSION_VISIBLE_INSTANCE_INDEX_RESOURCE"));
    assert!(implementation.contains("HZB_OCCLUSION_DRAW_COUNT_RESOURCE"));
    assert!(implementation.contains("HZB_OCCLUSION_COMPACTED_INDIRECT_ARGS_RESOURCE"));
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
        camera_world_position: [0.0, 0.0, 0.0, 1.0],
        camera_view_direction: [0.0, 0.0, 1.0, 0.0],
        sky_horizon_color: [0.0, 0.0, 0.0, 1.0],
        sky_zenith_color: [0.0, 0.0, 0.0, 1.0],
        sky_ground_color: [0.0, 0.0, 0.0, 1.0],
        sky_sun_direction: [0.0, 0.0, 0.0, 0.0],
        sky_sun_color_radius: [0.0, 0.0, 0.0, 0.0],
        sky_sun_params: [0.0, 0.0, 0.0, 0.0],
        environment_params: [0.0, 0.0, 0.0, 0.0],
        environment_sample_params: [0.0, 0.0, 0.0, 0.0],
        environment_rotation_sin_cos: [0.0, 1.0, 0.0, 0.0],
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
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    }))
}

fn test_skinned_joint_palette_min_binding_size() -> wgpu::BufferSize {
    wgpu::BufferSize::new(
        TEST_SKINNED_JOINT_MATRIX_COUNT * TEST_SKINNED_JOINT_MATRIX_BYTES
            + TEST_SKINNED_JOINT_PARAMS_BYTES,
    )
    .expect("test skinned joint palette storage size is non-zero")
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
        material_payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
        material_payload_padding: [0; 3],
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
        morph_payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
        lightmap_uv_rect: [0.0; 4],
        lightmap_params: [0; 4],
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

fn collect_hzb_stats(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
) -> Option<HzbOcclusionCullGpuStats> {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
    receiver.recv().ok()?.ok()?;

    let mapped = slice.get_mapped_range();
    let stats = *bytemuck::from_bytes::<HzbOcclusionCullGpuStats>(
        &mapped[..HZB_OCCLUSION_CULL_STATS_BUFFER_SIZE as usize],
    );
    drop(mapped);
    buffer.unmap();
    Some(stats)
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
