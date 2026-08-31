use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use bytemuck::Zeroable;
use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_runtime::asset::{
    MeshSdfAsset, MeshSdfCookSettings, MeshSdfEncoding, MESH_SDF_SCHEMA_VERSION,
};
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, RenderLayerSet,
    RenderMeshBounds, RenderMeshSnapshot, RenderMeshStaticState, RendererCommon,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use zircon_runtime::graphics::{
    RenderPassBufferUploadSink, RuntimePrepareFrameTransactionRecorder,
};

use crate::hybrid_gi::scene_representation::{
    HybridGiGlobalSdfSceneState, HybridGiMeshSdfAssetState, HybridGiMeshSdfMaterialFlags,
    HybridGiMeshSdfObject,
};

use super::*;

const GLOBAL_SDF_TRACE_WGPU_PNG: &str = "plan18_hybrid_gi_m5_global_sdf_trace_wgpu_20260810.png";
const MATRIX_CELL_SIDE: u32 = 32;
const MATRIX_SIDE: u32 = MATRIX_CELL_SIDE * 4;

struct QueueUploadSink<'a>(&'a wgpu::Queue);

impl RenderPassBufferUploadSink for QueueUploadSink<'_> {
    fn write_buffer(&mut self, buffer: &wgpu::Buffer, offset: u64, bytes: &[u8]) {
        self.0.write_buffer(buffer, offset, bytes);
    }
}

#[test]
fn trace_probe_tiles_pipeline_keeps_diagnostics_entrypoint_in_the_output_owner() {
    let trace_source = include_str!("../../../../../shaders/trace_probe_tiles.wgsl");
    let global_sdf_source =
        include_str!("../../../../../shaders/trace_probe_tiles_global_sdf.wgsl");
    let aggregate_source = include_str!("../../../../../shaders/trace_probe_tiles_aggregate.wgsl");
    let output_source = include_str!("../../../../../shaders/trace_probe_tiles_output.wgsl");
    let voxel_source = include_str!("../../../../../shaders/trace_probe_tiles_voxel.wgsl");

    assert!(trace_source.contains("const TRACE_DIAGNOSTIC_WORDS_PER_ENTRY: u32 = 13u;"));
    assert!(trace_source.contains("const SCENE_PREPARE_SIGNED_POSITION_BIAS: f32 = 2048.0;"));
    assert!(voxel_source.contains("(f32(i32(value)) - SCENE_PREPARE_SIGNED_POSITION_BIAS)"));
    assert!(!trace_source.contains("fn sample_global_sdf("));
    assert!(!trace_source.contains("fn global_sdf_tile_sample("));
    assert!(!trace_source.contains("fn tile_trace("));
    assert!(!trace_source.contains("fn cs_main("));
    assert!(global_sdf_source.contains("fn sample_global_sdf("));
    assert!(global_sdf_source.contains("fn global_sdf_tile_sample("));
    assert!(!global_sdf_source.contains("fn tile_trace("));
    assert!(!global_sdf_source.contains("fn cs_main("));
    assert!(aggregate_source.contains("fn tile_trace("));
    assert!(aggregate_source.contains("dominant_intersection_source("));
    assert!(aggregate_source.contains("intersection_backend_mask"));
    assert!(aggregate_source.contains("lighting_source_mask"));
    assert!(!aggregate_source.contains("fn cs_main("));
    assert!(output_source.contains("fn cs_main("));
    assert!(output_source.contains("probe_trace_diagnostics[diagnostic_offset + 12u]"));
}

#[test]
fn trace_probe_tiles_global_sdf_lookup_is_bounded_by_clipmap_count() {
    let trace_source = include_str!("../../../../../shaders/trace_probe_tiles.wgsl");
    let global_sdf_source =
        include_str!("../../../../../shaders/trace_probe_tiles_global_sdf.wgsl");

    assert!(trace_source.contains("var<storage, read> global_sdf_page_table: array<u32>;"));
    assert!(trace_source.contains("const GLOBAL_SDF_CLIPMAP_COUNT: u32 = 4u;"));
    assert!(global_sdf_source.contains(
        "for (var clipmap_index = 0u; clipmap_index < GLOBAL_SDF_CLIPMAP_COUNT; clipmap_index += 1u)"
    ));
    assert!(global_sdf_source.contains("cell_size >= selected_cell_size"));
    assert!(!global_sdf_source.contains("global_sdf_pages[page_index]"));
    assert!(!global_sdf_source.contains(
        "for (var page_index = 0u; page_index < params.global_sdf_page_count; page_index += 1u)"
    ));
}

#[test]
fn trace_probe_tiles_dispatch_params_match_wgsl_uniform_layout() {
    assert_eq!(std::mem::size_of::<ProbeTraceTileDispatchParams>(), 192);
    assert_eq!(
        std::mem::offset_of!(ProbeTraceTileDispatchParams, global_sdf_clipmaps),
        64
    );
}

#[test]
fn trace_probe_tiles_shader_uses_global_sdf_before_voxel_fallback() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping Global SDF trace Wgpu test because no adapter is available");
        return;
    };
    let bind_group_layout = create_probe_trace_tile_dispatch_bind_group_layout(&device);
    let pipeline = create_probe_trace_tile_dispatch_pipeline(&device, &bind_group_layout);
    let fallback_surface_cache = create_probe_trace_tile_fallback_surface_cache_textures(&device);
    let output = trace_global_sdf_tile(
        &device,
        &queue,
        &bind_group_layout,
        &pipeline,
        &fallback_surface_cache,
        0,
        &[0_u32; 512],
        [2_032, 2_048, 2_032],
    );

    assert_eq!(output.lighting, [1, 7, pack_rgb8([117, 167, 222])]);
    assert_eq!(output.diagnostics[0], 1);
    assert_eq!(output.diagnostics[1], 7);
    assert_eq!(
        output.diagnostics[2], 2,
        "Global SDF must report the actual intersection source"
    );
    assert_eq!(
        output.diagnostics[3], 0,
        "neutral ambient is used without lineage radiance"
    );
    assert_eq!(output.diagnostics[4], TRACE_BACKEND_GLOBAL_SDF);
    assert_eq!(
        output.diagnostics[5], 1,
        "neutral ambient contributes to the lighting mask"
    );
    assert_eq!(
        output.diagnostics[8], 1,
        "screen-data absence remains a typed fallback"
    );
    assert!(
        output.diagnostics[10] > 0,
        "Global SDF trace must report page tests"
    );
    assert!(
        output.diagnostics[11] > 0,
        "Global SDF trace must report SDF steps"
    );
}

#[test]
fn trace_diagnostics_aggregate_mixed_surface_and_global_sdf_tiles() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping mixed Surface/Global SDF Wgpu test because no adapter is available");
        return;
    };
    let surface_cache_params = ProbeTraceTileSurfaceCacheParams {
        texture_available: 1,
        atlas_width: 2,
        atlas_height: 1,
        atlas_columns: 2,
        tile_extent: 1,
    };
    let (_atlas_texture, atlas_view) = create_test_surface_cache_texture_with_pixels(
        &device,
        &queue,
        "zircon-hybrid-gi-mixed-surface-global-sdf-atlas",
        2,
        1,
        &[[160, 80, 40, 255], [0, 0, 0, 255]],
    );
    let (_depth_texture, depth_view) = create_test_surface_cache_texture_with_pixels(
        &device,
        &queue,
        "zircon-hybrid-gi-mixed-surface-global-sdf-depth",
        2,
        1,
        &[[128, 128, 128, 255], [255, 255, 255, 0]],
    );

    let diagnostics = trace_mixed_source_tiles(
        &device,
        &queue,
        surface_cache_params,
        &atlas_view,
        &depth_view,
        TRACE_BACKEND_SURFACE_CACHE | TRACE_BACKEND_GLOBAL_SDF,
        0,
        0,
        &[[0_u32; 12]],
        &[0, 7, 0, 1, 1, 7, 1, 1],
        &[0_u32; 512],
        [2_032, 2_048, 2_032],
    );

    assert_eq!(diagnostics[0], 1);
    assert_eq!(diagnostics[1], 7);
    assert_eq!(diagnostics[2], 1, "equal-weight ties prefer Surface Cache");
    assert_eq!(
        diagnostics[4],
        TRACE_BACKEND_SURFACE_CACHE | TRACE_BACKEND_GLOBAL_SDF
    );
    assert_eq!(
        diagnostics[5], 0b0011,
        "neutral and Surface Cache lighting both contributed"
    );
    assert_eq!(
        diagnostics[8], 4,
        "the invalid Surface Cache tile is an intersection miss fallback"
    );
    assert!(
        diagnostics[9] > 0,
        "the Surface Cache tile reports texture samples"
    );
    assert!(
        diagnostics[10] > 0,
        "the Global SDF tile reports page tests"
    );
}

#[test]
fn trace_diagnostics_aggregate_mixed_global_sdf_and_voxel_tiles() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping mixed Global SDF/Voxel Wgpu test because no adapter is available");
        return;
    };
    let fallback_surface_cache = create_probe_trace_tile_fallback_surface_cache_textures(&device);
    let mut global_sdf_atlas = [1.0_f32.to_bits(); 512];
    // Sample 1 stays in the resident page and reaches this cell. Sample 3 exits
    // through the unavailable negative-Y neighbor and falls back to the voxel cell.
    global_sdf_atlas[7 + 8 + 6 * 64] = 0.0_f32.to_bits();

    let diagnostics = trace_mixed_source_tiles(
        &device,
        &queue,
        ProbeTraceTileSurfaceCacheParams::unavailable(),
        &fallback_surface_cache.atlas_view,
        &fallback_surface_cache.depth_view,
        TRACE_BACKEND_GLOBAL_SDF | TRACE_BACKEND_VOXEL_CLIPMAP,
        1,
        1,
        &[voxel_cell_descriptor_words(7, 3, 4, [24, 96, 160])],
        &[0, 7, 1, 1, 1, 7, 3, 12],
        &global_sdf_atlas,
        [2_032, 2_048, 2_032],
    );

    assert_eq!(diagnostics[0], 1);
    assert_eq!(diagnostics[1], 7);
    assert_eq!(
        diagnostics[2], 3,
        "the higher-weight Voxel tile is the dominant intersection source"
    );
    assert_eq!(
        diagnostics[3], 3,
        "the higher-weight Voxel tile is the dominant lighting source"
    );
    assert_eq!(
        diagnostics[4],
        TRACE_BACKEND_GLOBAL_SDF | TRACE_BACKEND_VOXEL_CLIPMAP
    );
    assert_eq!(
        diagnostics[5], 0b1001,
        "neutral and voxel radiance both contributed"
    );
    assert_eq!(
        diagnostics[8], 1,
        "the unavailable screen route remains observable"
    );
    assert!(diagnostics[10] > 0, "Global SDF page tests are accumulated");
    assert!(diagnostics[11] > 0, "Global SDF steps are accumulated");
    assert!(
        f32::from_bits(diagnostics[6]) > 0.3,
        "Voxel provenance retains the nonzero world-space distance to its cell"
    );
    assert_eq!(
        diagnostics[12], 1,
        "only the occupied voxel-cell range is scanned"
    );
}

#[test]
#[ignore = "requires a real DX12 Wgpu adapter and writes a Global SDF trace PNG"]
fn export_hybrid_gi_m5_global_sdf_trace_wgpu_png() {
    let (device, queue) = test_device_with_backends(wgpu::Backends::DX12).expect(
        "Global SDF trace PNG export requires a DX12 Wgpu adapter for RenderDoc; do not accept a skipped run",
    );
    let bind_group_layout = create_probe_trace_tile_dispatch_bind_group_layout(&device);
    let pipeline = create_probe_trace_tile_dispatch_pipeline(&device, &bind_group_layout);
    let fallback_surface_cache = create_probe_trace_tile_fallback_surface_cache_textures(&device);
    let (global_sdf_state, global_sdf_scene_state) = build_sphere_global_sdf(&device, &queue, 1.25);
    let mut upload_sink = QueueUploadSink(&queue);
    let mut frame_transactions = Vec::new();
    let mut frame_transaction_recorder =
        RuntimePrepareFrameTransactionRecorder::new(&mut frame_transactions);
    let global_sdf_bindings = global_sdf_state.create_trace_bindings(
        &mut upload_sink,
        &mut frame_transaction_recorder,
        &global_sdf_scene_state,
    );
    assert_eq!(global_sdf_bindings.page_count, 1);
    let mut samples = [0_u32; 16];
    for (tile_sample_id, sample) in samples.iter_mut().enumerate() {
        *sample = trace_global_sdf_tile_with_bindings(
            &device,
            &queue,
            &bind_group_layout,
            &pipeline,
            &fallback_surface_cache,
            tile_sample_id as u32,
            &global_sdf_bindings,
            [1_808, 2_176, 2_176],
        )
        .lighting[2];
    }

    let distinct_samples = samples
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let global_sdf_hit_cells = samples
        .iter()
        .filter(|sample| is_global_sdf_hit_palette(**sample))
        .count();
    assert!(
        distinct_samples > 1,
        "directional Global SDF trace should produce hit and miss variation; samples={samples:?}"
    );
    assert!(
        global_sdf_hit_cells > 0,
        "at least one direction should sphere-trace the uploaded Global SDF; samples={samples:?}"
    );

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_trace_matrix_png(output_dir.join(GLOBAL_SDF_TRACE_WGPU_PNG), &samples);
}

#[allow(clippy::too_many_arguments)]
fn trace_global_sdf_tile(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bind_group_layout: &wgpu::BindGroupLayout,
    pipeline: &wgpu::ComputePipeline,
    fallback_surface_cache: &ProbeTraceTileFallbackSurfaceCacheTextures,
    tile_sample_id: u32,
    atlas: &[u32; 512],
    probe_position_q: [u32; 3],
) -> GlobalSdfTraceOutput {
    let global_sdf_page_table_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-global-sdf-trace-test-page-table",
        &global_sdf_trace_test_page_table(),
    );
    let global_sdf_atlas_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-global-sdf-trace-test-atlas",
        atlas,
    );
    trace_global_sdf_tile_with_buffers(
        device,
        queue,
        bind_group_layout,
        pipeline,
        fallback_surface_cache,
        tile_sample_id,
        &global_sdf_page_table_buffer,
        &global_sdf_atlas_buffer,
        1,
        global_sdf_trace_test_clipmaps(),
        probe_position_q,
    )
}

#[allow(clippy::too_many_arguments)]
fn trace_global_sdf_tile_with_bindings(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bind_group_layout: &wgpu::BindGroupLayout,
    pipeline: &wgpu::ComputePipeline,
    fallback_surface_cache: &ProbeTraceTileFallbackSurfaceCacheTextures,
    tile_sample_id: u32,
    bindings: &GlobalSdfGpuTraceBindings,
    probe_position_q: [u32; 3],
) -> GlobalSdfTraceOutput {
    trace_global_sdf_tile_with_buffers(
        device,
        queue,
        bind_group_layout,
        pipeline,
        fallback_surface_cache,
        tile_sample_id,
        &bindings.page_table_buffer,
        &bindings.atlas_buffer,
        bindings.page_count,
        bindings.clipmaps,
        probe_position_q,
    )
}

#[allow(clippy::too_many_arguments)]
fn trace_global_sdf_tile_with_buffers(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bind_group_layout: &wgpu::BindGroupLayout,
    pipeline: &wgpu::ComputePipeline,
    fallback_surface_cache: &ProbeTraceTileFallbackSurfaceCacheTextures,
    tile_sample_id: u32,
    global_sdf_page_table_buffer: &wgpu::Buffer,
    global_sdf_atlas_buffer: &wgpu::Buffer,
    global_sdf_page_count: u32,
    global_sdf_clipmaps: [GlobalSdfGpuTraceClipmap; GLOBAL_SDF_CLIPMAP_COUNT],
    probe_position_q: [u32; 3],
) -> GlobalSdfTraceOutput {
    let params_buffer =
        create_probe_trace_tile_dispatch_params_buffer_with_route_and_global_sdf_clipmaps(
            device,
            1,
            0,
            1,
            ProbeTraceTileSurfaceCacheParams::unavailable(),
            0,
            global_sdf_page_count,
            TRACE_BACKEND_GLOBAL_SDF,
            trace_lighting_source_code(HybridGiLightingSource::ProbeLineage),
            trace_fallback_reason_code(Some(HybridGiTraceFallbackReason::ScreenDataUnavailable)),
            0,
            0,
            global_sdf_clipmaps,
        );
    let mut probe = resident_probe_input(7);
    probe.position_x_q = probe_position_q[0];
    probe.position_y_q = probe_position_q[1];
    probe.position_z_q = probe_position_q[2];
    let resident_probe_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-global-sdf-trace-test-resident-probe",
        &[probe],
    );
    let pending_probe_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-global-sdf-trace-test-pending-probe",
        &[GpuPendingProbeInput::zeroed()],
    );
    let probe_trace_tile_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-global-sdf-trace-test-tile-schedule",
        &[0_u32, 7, tile_sample_id, 12],
    );
    let trace_lighting_buffer = create_trace_lighting_buffer(device, 3);
    let readback_buffer = create_readback_buffer(device, 3);
    let trace_diagnostic_buffer = create_trace_lighting_buffer(device, 14);
    let trace_diagnostic_readback_buffer = create_readback_buffer(device, 14);
    let scene_prepare_descriptor_buffer = create_zeroed_scene_prepare_descriptor_buffer(device);
    let bind_group = create_probe_trace_tile_dispatch_bind_group_from_buffers_with_diagnostics(
        device,
        bind_group_layout,
        &params_buffer,
        &resident_probe_buffer,
        &pending_probe_buffer,
        &probe_trace_tile_buffer,
        &trace_lighting_buffer,
        &trace_diagnostic_buffer,
        &fallback_surface_cache.atlas_view,
        &fallback_surface_cache.depth_view,
        &scene_prepare_descriptor_buffer,
        global_sdf_page_table_buffer,
        global_sdf_atlas_buffer,
    );
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hybrid-gi-global-sdf-trace-test-encoder"),
    });
    encode_probe_trace_tile_dispatch(&mut encoder, pipeline, &bind_group, 1);
    encoder.copy_buffer_to_buffer(
        &trace_lighting_buffer,
        0,
        &readback_buffer,
        0,
        (3 * std::mem::size_of::<u32>()) as u64,
    );
    encoder.copy_buffer_to_buffer(
        &trace_diagnostic_buffer,
        0,
        &trace_diagnostic_readback_buffer,
        0,
        (14 * std::mem::size_of::<u32>()) as u64,
    );
    queue.submit(std::iter::once(encoder.finish()));
    GlobalSdfTraceOutput {
        lighting: readback_u32s(device, &readback_buffer, 3)
            .try_into()
            .expect("trace readback has exactly three words"),
        diagnostics: readback_u32s(device, &trace_diagnostic_readback_buffer, 14)
            .try_into()
            .expect("trace diagnostics readback has exactly fourteen words"),
    }
}

#[allow(clippy::too_many_arguments)]
fn trace_mixed_source_tiles(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    surface_cache_params: ProbeTraceTileSurfaceCacheParams,
    surface_cache_atlas_view: &wgpu::TextureView,
    surface_cache_depth_view: &wgpu::TextureView,
    intersection_backend_mask: u32,
    fallback_reason: u32,
    voxel_cell_descriptor_count: u32,
    scene_prepare_descriptors: &[[u32; 12]],
    tile_schedule: &[u32],
    global_sdf_atlas: &[u32; 512],
    probe_position_q: [u32; 3],
) -> [u32; 14] {
    assert_eq!(tile_schedule.len() % 4, 0);
    let (voxel_cell_lookup_clipmap_count, voxel_cell_lookup_words) =
        voxel_cell_lookup_words_for_descriptors(scene_prepare_descriptors);
    let params_buffer =
        create_probe_trace_tile_dispatch_params_buffer_with_route_and_global_sdf_clipmaps(
            device,
            1,
            0,
            (tile_schedule.len() / 4) as u32,
            surface_cache_params,
            0,
            1,
            intersection_backend_mask,
            0,
            fallback_reason,
            voxel_cell_descriptor_count,
            voxel_cell_lookup_clipmap_count,
            global_sdf_trace_test_clipmaps(),
        );
    let mut probe = resident_probe_input(7);
    probe.position_x_q = probe_position_q[0];
    probe.position_y_q = probe_position_q[1];
    probe.position_z_q = probe_position_q[2];
    let resident_probe_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-mixed-trace-resident-probe",
        &[probe],
    );
    let pending_probe_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-mixed-trace-pending-probe",
        &[GpuPendingProbeInput::zeroed()],
    );
    let probe_trace_tile_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-mixed-trace-tile-schedule",
        tile_schedule,
    );
    let trace_lighting_buffer = create_trace_lighting_buffer(device, 3);
    let trace_diagnostic_buffer = create_trace_lighting_buffer(device, 14);
    let trace_diagnostic_readback_buffer = create_readback_buffer(device, 14);
    let scene_prepare_descriptor_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-mixed-trace-scene-prepare-descriptors",
        scene_prepare_descriptors,
    );
    let global_sdf_page_table_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-mixed-trace-global-sdf-page-table",
        &global_sdf_trace_test_page_table(),
    );
    let global_sdf_atlas_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-mixed-trace-global-sdf-atlas",
        global_sdf_atlas,
    );
    let voxel_cell_lookup_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-mixed-trace-voxel-cell-lookup",
        &voxel_cell_lookup_words,
    );
    let bind_group_layout = create_probe_trace_tile_dispatch_bind_group_layout(device);
    let pipeline = create_probe_trace_tile_dispatch_pipeline(device, &bind_group_layout);
    let bind_group =
        create_probe_trace_tile_dispatch_bind_group_from_buffers_with_diagnostics_and_voxel_lookup(
            device,
            &bind_group_layout,
            &params_buffer,
            &resident_probe_buffer,
            &pending_probe_buffer,
            &probe_trace_tile_buffer,
            &trace_lighting_buffer,
            &trace_diagnostic_buffer,
            surface_cache_atlas_view,
            surface_cache_depth_view,
            &scene_prepare_descriptor_buffer,
            &voxel_cell_lookup_buffer,
            &global_sdf_page_table_buffer,
            &global_sdf_atlas_buffer,
        );
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hybrid-gi-mixed-trace-encoder"),
    });
    encode_probe_trace_tile_dispatch(&mut encoder, &pipeline, &bind_group, 1);
    encoder.copy_buffer_to_buffer(
        &trace_diagnostic_buffer,
        0,
        &trace_diagnostic_readback_buffer,
        0,
        (14 * std::mem::size_of::<u32>()) as u64,
    );
    queue.submit(std::iter::once(encoder.finish()));
    readback_u32s(device, &trace_diagnostic_readback_buffer, 14)
        .try_into()
        .expect("mixed trace diagnostics readback has exactly fourteen words")
}

struct GlobalSdfTraceOutput {
    lighting: [u32; 3],
    diagnostics: [u32; 14],
}

fn global_sdf_trace_test_page_table() -> [u32; 2_048] {
    let mut page_table = [u32::MAX; 2_048];
    // Clipmap 0 has origin [-4, -4, -4]; the tested world position lies in
    // absolute page [-1, 0, -1], or local coordinate [3, 4, 3].
    page_table[3 + 4 * 8 + 3 * 8 * 8] = 0;
    page_table
}

fn global_sdf_trace_test_clipmaps() -> [GlobalSdfGpuTraceClipmap; GLOBAL_SDF_CLIPMAP_COUNT] {
    let mut clipmaps = [GlobalSdfGpuTraceClipmap::zeroed(); GLOBAL_SDF_CLIPMAP_COUNT];
    clipmaps[0] = GlobalSdfGpuTraceClipmap {
        page_coordinate_origin_and_padding: [-4, -4, -4, 0],
        page_world_size_and_padding: [8.0, 0.0, 0.0, 0.0],
    };
    clipmaps
}

fn build_sphere_global_sdf(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    radius: f32,
) -> (GlobalSdfGpuState, HybridGiGlobalSdfSceneState) {
    let mut scene = HybridGiGlobalSdfSceneState::default();
    scene.synchronize(Vec3::new(-4.0, 0.0, 0.0), &[], 1);
    let requests = scene.dirty_page_build_requests();
    assert_eq!(requests.len(), 1);
    let clipmaps = scene.clipmap_bounds().to_vec();
    let local_bounds = RenderMeshBounds::from_min_max([-2.0; 3], [2.0; 3]);
    let object = HybridGiMeshSdfObject::from_sources(
        &global_sdf_test_mesh(Vec3::new(-2.0, 2.0, 2.0)),
        local_bounds,
        1,
        1,
        HybridGiMeshSdfAssetState::Ready(Arc::<[MeshSdfAsset]>::from(vec![sphere_mesh_sdf_asset(
            radius,
        )])),
        HybridGiMeshSdfMaterialFlags::default(),
        &clipmaps,
    );
    scene.synchronize_influence(std::slice::from_ref(&object));
    let resources = HybridGiGpuResources::new(device);
    let state = GlobalSdfGpuState::new(device);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hybrid-gi-global-sdf-product-build-encoder"),
    });
    let dispatch = resources.dispatch_global_sdf_pages(
        &state,
        device,
        &mut encoder,
        &mut scene,
        &[object],
        &requests,
        1,
    );
    let build_stats = dispatch.stats();
    assert_eq!(build_stats.dispatched_page_count, 1);
    let pending = dispatch
        .into_pending()
        .expect("ready Mesh SDF page must dispatch a Global SDF build");
    let persistent_resource_byte_count = state.persistent_resource_byte_count();
    assert!(persistent_resource_byte_count > 0);
    let completion_readback = create_readback_buffer(device, pending.request_count());
    pending.copy_completion_to(&mut encoder, &completion_readback);
    queue.submit(std::iter::once(encoder.finish()));
    let completed = pending
        .completed_requests_from_words(&readback_u32s(device, &completion_readback, requests.len()))
        .expect("completion readback must match the dispatched Global SDF pages");
    assert_eq!(completed, requests);
    scene.commit_pages(&completed);
    assert!(scene.is_page_sampleable(requests[0].key()));
    (state, scene)
}

fn sphere_mesh_sdf_asset(radius: f32) -> MeshSdfAsset {
    let half_extent = 2.0;
    let voxel_size = 0.5;
    let voxels = (0..8)
        .flat_map(|z| {
            (0..8).flat_map(move |y| {
                (0..8).map(move |x| {
                    let position =
                        Vec3::new(x as f32 - 3.5, y as f32 - 3.5, z as f32 - 3.5) * voxel_size;
                    (((position.length() - radius) / half_extent).clamp(-1.0, 1.0)
                        * i16::MAX as f32)
                        .round() as i16
                })
            })
        })
        .collect();
    MeshSdfAsset {
        schema_version: MESH_SDF_SCHEMA_VERSION,
        source_hash: [1; 32],
        local_bounds: RenderMeshBounds::from_min_max([-half_extent; 3], [half_extent; 3]),
        dimensions: [8; 3],
        voxel_size: [voxel_size; 3],
        distance_range: [-half_extent, half_extent],
        encoding: MeshSdfEncoding::SignedNormalized16,
        cook_settings: MeshSdfCookSettings::default(),
        voxels,
    }
}

fn global_sdf_test_mesh(translation: Vec3) -> RenderMeshSnapshot {
    let transform = Transform::from_translation(translation);
    RenderMeshSnapshot {
        node_id: 700,
        stable_instance_key: render_mesh_stable_instance_key(700, 0),
        transform_revision: render_mesh_transform_revision(&transform),
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "res://models/global-sdf-product.model.toml",
        )),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "res://materials/global-sdf-product.zmaterial",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: RenderMeshStaticState::from_transform_static(true),
        common: RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            is_static: true,
            ..RendererCommon::default()
        },
    }
}

fn sphere_distance_atlas(radius: f32) -> [u32; 512] {
    let mut atlas = [0_u32; 512];
    for z in 0..8 {
        for y in 0..8 {
            for x in 0..8 {
                let position = zircon_runtime::core::math::Vec3::new(
                    x as f32 - 3.5,
                    y as f32 - 3.5,
                    z as f32 - 3.5,
                );
                atlas[x + y * 8 + z * 64] = (position.length() - radius).to_bits();
            }
        }
    }
    atlas
}

fn is_global_sdf_hit_palette(packed: u32) -> bool {
    (0..16).any(|step_index| {
        let visibility = 255_u32.saturating_sub((step_index * 11).min(176));
        packed
            == pack_rgb8([
                32 + visibility / 3,
                40 + visibility / 2,
                52 + visibility * 2 / 3,
            ])
    })
}

fn write_trace_matrix_png(path: PathBuf, samples: &[u32; 16]) {
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(MATRIX_SIDE, MATRIX_SIDE);
    for (cell_index, sample) in samples.iter().copied().enumerate() {
        let rgb = [
            (sample & 0xff) as u8,
            ((sample >> 8) & 0xff) as u8,
            ((sample >> 16) & 0xff) as u8,
        ];
        let origin_x = cell_index as u32 % 4 * MATRIX_CELL_SIDE;
        let origin_y = cell_index as u32 / 4 * MATRIX_CELL_SIDE;
        for y in 0..MATRIX_CELL_SIDE {
            for x in 0..MATRIX_CELL_SIDE {
                let pixel = if x == 0 || y == 0 {
                    Rgba([5, 7, 9, 255])
                } else {
                    Rgba([rgb[0], rgb[1], rgb[2], 255])
                };
                image.put_pixel(origin_x + x, origin_y + y, pixel);
            }
        }
    }
    image.save_with_format(path, ImageFormat::Png).unwrap();
}

fn render_test_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
}
