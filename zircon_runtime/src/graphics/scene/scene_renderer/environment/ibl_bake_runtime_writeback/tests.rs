use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::artifact::{
    resolve_ibl_bake_artifact_runtime_dispatch, IblBakeArtifactRuntimeDispatchReadbackStatus,
};
use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    build_source_cubemap_from_equirect, build_source_cubemap_irradiance_cube,
    source_cubemap_face_mip_offset, source_cubemap_mip_size,
    source_cubemap_pmrem_mip_from_roughness, CubemapFace, IblBakeArtifactBlob,
    IblBakeArtifactContents, IblBakeArtifactDescriptor, IblBakeArtifactReadbackSections,
    IblBakeArtifactRequest, ProceduralSkyParams, RenderFrameExtract, RenderPluginRendererOutputs,
    RenderWorldSnapshotHandle, SourceCubemapIrradianceCube, SourceCubemapMipChain,
    IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES, SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
};
use crate::core::math::UVec2;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::environment::ibl_bake_wgpu_dispatch::record_ibl_bake_wgpu_pass_for_request;
use crate::graphics::scene::scene_renderer::environment::ibl_bake_wgpu_pipeline_cache::IblBakeWgpuPipelineCache;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderPassExecutionContext, RenderPassExecutorId,
    RenderPassGpuExecutionContext,
};
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::graphics::ViewportRenderFrame;
use crate::render_graph::RenderGraphBuilder;
use crate::scene::world::World;

use super::super::ibl_bake_graph_plan::{
    append_ibl_bake_artifact_graph_plan, IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID,
    IBL_BAKE_IRRADIANCE_SH9_PASS, IBL_BAKE_PMREM_EXECUTOR_ID, IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
};
use super::*;

mod metrics;

use metrics::{
    irradiance_cube_directional_stats, pmrem_seam_luma_stats, synthetic_irradiance_environment,
    synthetic_seam_stress_environment,
};

const RGBA8_BYTES_PER_TEXEL: usize = 4;

#[test]
fn runtime_graph_writeback_skips_graph_readback_when_dispatch_not_required() {
    let temp_root = unique_temp_cache_root("ibl-writeback-skip");
    let store = IblBakeArtifactCacheStore::new(&temp_root);
    let request = request(IblBakeArtifactContents::SH9);
    let asset_blob = sh9_blob_for_request(&request);
    let dispatch = resolve_ibl_bake_artifact_runtime_dispatch(&store, &request, &[asset_blob])
        .expect("asset-derived artifact should resolve");
    assert!(!dispatch.requires_runtime_compute());

    let Ok(backend) = RenderBackend::new_offscreen() else {
        let _ = fs::remove_dir_all(&temp_root);
        return;
    };
    let resources = RenderGraphExecutionResources::new();
    let report = write_ibl_bake_runtime_cache_from_graph_resources(
        &backend.device,
        &backend.queue,
        &store,
        &request,
        &dispatch,
        &resources,
    )
    .expect("cache-hit dispatch should skip graph readback");

    assert_eq!(
        report.status(),
        IblBakeArtifactRuntimeDispatchReadbackStatus::SkippedRuntimeComputeNotRequired
    );
    assert!(!report.wrote_cache());
    assert!(!store.runtime_cache_path(&request).exists());
    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn runtime_graph_writeback_reads_sh9_graph_output_and_writes_runtime_cache() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let temp_root = unique_temp_cache_root("ibl-writeback-sh9");
    let store = IblBakeArtifactCacheStore::new(&temp_root);
    let request = request(IblBakeArtifactContents::SH9);
    let dispatch = resolve_ibl_bake_artifact_runtime_dispatch(&store, &request, &[])
        .expect("runtime cache miss should require compute");
    assert!(dispatch.requires_runtime_compute());

    let mut resources = dispatch_sh9_graph_output(&backend, &request);
    let report = write_ibl_bake_runtime_cache_from_graph_resources(
        &backend.device,
        &backend.queue,
        &store,
        &request,
        &dispatch,
        &resources,
    )
    .expect("SH9 graph output should write runtime cache");

    assert_eq!(
        report.status(),
        IblBakeArtifactRuntimeDispatchReadbackStatus::Written
    );
    assert!(report.wrote_cache());
    let writeback = report.writeback().expect("writeback report");
    assert_eq!(writeback.payload_len(), IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES);
    assert_eq!(
        writeback.path(),
        Some(store.runtime_cache_path(&request).as_path())
    );
    assert!(store.runtime_cache_path(&request).exists());

    let second = resolve_ibl_bake_artifact_runtime_dispatch(&store, &request, &[])
        .expect("runtime cache hit should resolve on second dispatch");
    assert!(!second.requires_runtime_compute());
    assert_eq!(second.environment_compute_dispatch_count(), 0);

    resources.release_transient_backings_into_pool(&mut Default::default());
    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn runtime_graph_writeback_reads_pmrem_graph_output_and_preserves_readback_seams() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let source = build_source_cubemap_from_equirect(32, synthetic_seam_stress_environment);
    let temp_root = unique_temp_cache_root("ibl-writeback-pmrem-seams");
    let store = IblBakeArtifactCacheStore::new(&temp_root);
    let request = IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        source.source_face_size(),
        source.source_mip_count(),
    )
    .with_required_contents(IblBakeArtifactContents::PMREM);
    let dispatch = resolve_ibl_bake_artifact_runtime_dispatch(&store, &request, &[])
        .expect("runtime cache miss should require PMREM compute");
    assert!(dispatch.requires_runtime_compute());
    assert_eq!(dispatch.environment_compute_dispatch_count(), 1);

    let mut resources = dispatch_pmrem_graph_output(&backend, &request, &source);
    let report = write_ibl_bake_runtime_cache_from_graph_resources(
        &backend.device,
        &backend.queue,
        &store,
        &request,
        &dispatch,
        &resources,
    )
    .expect("PMREM graph output should write runtime cache");

    assert_eq!(
        report.status(),
        IblBakeArtifactRuntimeDispatchReadbackStatus::Written
    );
    assert!(report.wrote_cache());
    assert!(store.runtime_cache_path(&request).exists());

    let second = resolve_ibl_bake_artifact_runtime_dispatch(&store, &request, &[])
        .expect("runtime cache hit should resolve after PMREM readback writeback");
    assert!(!second.requires_runtime_compute());
    assert_eq!(second.environment_compute_dispatch_count(), 0);
    let payload = second
        .payload()
        .expect("runtime cache hit should expose PMREM payload");
    let computed_pmrem = SourceCubemapMipChain::new(
        source.source_face_size(),
        source.source_mip_count(),
        source.source_texels().to_vec(),
        request.pmrem_face_size(),
        request.pmrem_mip_count(),
        payload
            .decode_pmrem_texels()
            .expect("PMREM payload should decode"),
    );
    let mid_mip = source_cubemap_pmrem_mip_from_roughness(0.5, computed_pmrem.pmrem_mip_count())
        .round() as u32;
    let rough_mip = source_cubemap_pmrem_mip_from_roughness(1.0, computed_pmrem.pmrem_mip_count())
        .round() as u32;
    let base = pmrem_seam_luma_stats(&computed_pmrem, 0);
    let mid = pmrem_seam_luma_stats(&computed_pmrem, mid_mip);
    let rough = pmrem_seam_luma_stats(&computed_pmrem, rough_mip);

    assert!(
        mid.mean < base.mean * 0.95,
        "live PMREM compute mid mip should reduce seam energy before cache writeback, base={base:?} mid={mid:?} rough={rough:?}"
    );
    assert!(
        rough.max < base.max * 0.9,
        "live PMREM compute rough mip should reduce worst seam delta before cache writeback, base={base:?} mid={mid:?} rough={rough:?}"
    );

    resources.release_transient_backings_into_pool(&mut Default::default());
    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn runtime_graph_writeback_reads_iem_graph_output_and_preserves_directional_irradiance() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let source = build_source_cubemap_from_equirect(32, synthetic_irradiance_environment);
    let reference_iem = build_source_cubemap_irradiance_cube(&source);
    let temp_root = unique_temp_cache_root("ibl-writeback-iem-directional");
    let store = IblBakeArtifactCacheStore::new(&temp_root);
    let request = IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        source.source_face_size(),
        source.source_mip_count(),
    )
    .with_required_contents(IblBakeArtifactContents::IEM);
    let dispatch = resolve_ibl_bake_artifact_runtime_dispatch(&store, &request, &[])
        .expect("runtime cache miss should require IEM compute");
    assert!(dispatch.requires_runtime_compute());
    assert_eq!(dispatch.environment_compute_dispatch_count(), 1);

    let mut resources = dispatch_irradiance_cube_graph_output(&backend, &request, &source);
    let report = write_ibl_bake_runtime_cache_from_graph_resources(
        &backend.device,
        &backend.queue,
        &store,
        &request,
        &dispatch,
        &resources,
    )
    .expect("IEM graph output should write runtime cache");

    assert_eq!(
        report.status(),
        IblBakeArtifactRuntimeDispatchReadbackStatus::Written
    );
    assert!(report.wrote_cache());
    assert!(store.runtime_cache_path(&request).exists());

    let second = resolve_ibl_bake_artifact_runtime_dispatch(&store, &request, &[])
        .expect("runtime cache hit should resolve after IEM readback writeback");
    assert!(!second.requires_runtime_compute());
    assert_eq!(second.environment_compute_dispatch_count(), 0);
    let payload = second
        .payload()
        .expect("runtime cache hit should expose IEM payload");
    let decoded = payload
        .decode_irradiance_cube_texels()
        .expect("IEM payload should decode");
    let expected_texel_count = SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
        * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
        * CubemapFace::ALL.len();
    assert_eq!(decoded.len(), expected_texel_count);
    let computed_iem =
        SourceCubemapIrradianceCube::new(SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE, decoded);
    let stats = irradiance_cube_directional_stats(&computed_iem, &reference_iem);
    assert!(
        stats.computed_mean > 0.05 && stats.reference_mean > 0.05,
        "live IEM output should be non-black, stats={stats:?}"
    );
    assert!(
        stats.computed_dynamic_range > 0.08 && stats.reference_dynamic_range > 0.03,
        "live IEM output should preserve directional variation, stats={stats:?}"
    );
    assert!(
        stats.normalized_rms < 0.35,
        "live IEM output should track CPU cosine-convolution direction response after scale normalization, stats={stats:?}"
    );
    assert!(
        stats.correlation > 0.8,
        "live IEM output should correlate with CPU cosine-convolution direction response, stats={stats:?}"
    );

    resources.release_transient_backings_into_pool(&mut Default::default());
    let _ = fs::remove_dir_all(&temp_root);
}

fn dispatch_sh9_graph_output(
    backend: &RenderBackend,
    request: &IblBakeArtifactRequest,
) -> RenderGraphExecutionResources {
    let mut builder = RenderGraphBuilder::new("ibl-bake-runtime-writeback-test");
    append_ibl_bake_artifact_graph_plan(&mut builder, request)
        .expect("IBL bake graph plan should append");
    let graph = builder.compile().expect("IBL bake graph should compile");
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == IBL_BAKE_IRRADIANCE_SH9_PASS)
        .expect("SH9 IBL bake pass should exist");
    let mut resources = RenderGraphExecutionResources::new();
    resources
        .materialize_transient_resources(&backend.device, &graph)
        .expect("IBL transient outputs should materialize");
    let source_texture = create_source_cubemap_texture(&backend.device);
    resources.import_texture_view(
        IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
        source_texture.create_view(&source_cubemap_view_descriptor()),
    );

    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ibl-bake-runtime-writeback-test-encoder"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ibl-bake-runtime-writeback-test-scene-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ibl-bake-runtime-writeback-test-scene-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let mut ibl_bake_pipeline_cache = IblBakeWgpuPipelineCache::new(&backend.device);
    {
        let gpu = RenderPassGpuExecutionContext::new_for_test(
            &backend.device,
            &backend.queue,
            &mut encoder,
            &frame,
            &scene_bind_group_layout,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Depth32Float,
            &scene_bind_group,
            &mut resources,
            &mut plugin_outputs,
            &mut screen_space_ui_renderer,
        )
        .with_ibl_bake_pipeline_cache(&mut ibl_bake_pipeline_cache);
        let mut context =
            RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
                pass.name.clone(),
                RenderPassExecutorId::new(pass.executor_id.clone().unwrap()),
                pass.queue,
                pass.declared_queue,
                pass.flags,
                pass.dependencies.clone(),
                pass.resources.clone(),
            )
            .with_resource_resolver(&graph, pass.id)
            .with_gpu(gpu);

        record_ibl_bake_wgpu_pass_for_request(&mut context, request)
            .expect("SH9 IBL WGPU pass should encode from graph resources");
    }
    backend.queue.submit(std::iter::once(encoder.finish()));
    backend
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("IBL SH9 graph dispatch should finish");

    resources
}

fn dispatch_pmrem_graph_output(
    backend: &RenderBackend,
    request: &IblBakeArtifactRequest,
    source: &SourceCubemapMipChain,
) -> RenderGraphExecutionResources {
    let mut builder = RenderGraphBuilder::new("ibl-bake-runtime-writeback-pmrem-test");
    append_ibl_bake_artifact_graph_plan(&mut builder, request)
        .expect("IBL bake graph plan should append");
    let graph = builder.compile().expect("IBL bake graph should compile");
    let mut resources = RenderGraphExecutionResources::new();
    resources
        .materialize_transient_resources(&backend.device, &graph)
        .expect("IBL PMREM transient outputs should materialize");
    let source_texture =
        create_source_cubemap_texture_from_chain(&backend.device, &backend.queue, source);
    resources.import_texture_view(
        IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
        source_texture.create_view(&source_cubemap_mip_view_descriptor(
            source.source_mip_count(),
        )),
    );

    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ibl-bake-runtime-writeback-pmrem-test-encoder"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ibl-bake-runtime-writeback-pmrem-test-scene-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ibl-bake-runtime-writeback-pmrem-test-scene-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(32, 32));
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let mut ibl_bake_pipeline_cache = IblBakeWgpuPipelineCache::new(&backend.device);

    for pass in graph
        .passes()
        .iter()
        .filter(|pass| pass.executor_id.as_deref() == Some(IBL_BAKE_PMREM_EXECUTOR_ID))
    {
        let gpu = RenderPassGpuExecutionContext::new_for_test(
            &backend.device,
            &backend.queue,
            &mut encoder,
            &frame,
            &scene_bind_group_layout,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Depth32Float,
            &scene_bind_group,
            &mut resources,
            &mut plugin_outputs,
            &mut screen_space_ui_renderer,
        )
        .with_ibl_bake_pipeline_cache(&mut ibl_bake_pipeline_cache);
        let mut context =
            RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
                pass.name.clone(),
                RenderPassExecutorId::new(pass.executor_id.clone().unwrap()),
                pass.queue,
                pass.declared_queue,
                pass.flags,
                pass.dependencies.clone(),
                pass.resources.clone(),
            )
            .with_resource_resolver(&graph, pass.id)
            .with_gpu(gpu);

        record_ibl_bake_wgpu_pass_for_request(&mut context, request)
            .expect("PMREM IBL WGPU pass should encode from graph resources");
    }

    backend.queue.submit(std::iter::once(encoder.finish()));
    backend
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("IBL PMREM graph dispatches should finish");

    resources
}

fn dispatch_irradiance_cube_graph_output(
    backend: &RenderBackend,
    request: &IblBakeArtifactRequest,
    source: &SourceCubemapMipChain,
) -> RenderGraphExecutionResources {
    let mut builder = RenderGraphBuilder::new("ibl-bake-runtime-writeback-iem-test");
    append_ibl_bake_artifact_graph_plan(&mut builder, request)
        .expect("IBL bake graph plan should append");
    let graph = builder.compile().expect("IBL bake graph should compile");
    let mut resources = RenderGraphExecutionResources::new();
    resources
        .materialize_transient_resources(&backend.device, &graph)
        .expect("IBL IEM transient output should materialize");
    let source_texture =
        create_source_cubemap_texture_from_chain(&backend.device, &backend.queue, source);
    resources.import_texture_view(
        IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
        source_texture.create_view(&source_cubemap_mip_view_descriptor(
            source.source_mip_count(),
        )),
    );

    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ibl-bake-runtime-writeback-iem-test-encoder"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ibl-bake-runtime-writeback-iem-test-scene-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ibl-bake-runtime-writeback-iem-test-scene-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(32, 32));
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let mut ibl_bake_pipeline_cache = IblBakeWgpuPipelineCache::new(&backend.device);

    for pass in graph
        .passes()
        .iter()
        .filter(|pass| pass.executor_id.as_deref() == Some(IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID))
    {
        let gpu = RenderPassGpuExecutionContext::new_for_test(
            &backend.device,
            &backend.queue,
            &mut encoder,
            &frame,
            &scene_bind_group_layout,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Depth32Float,
            &scene_bind_group,
            &mut resources,
            &mut plugin_outputs,
            &mut screen_space_ui_renderer,
        )
        .with_ibl_bake_pipeline_cache(&mut ibl_bake_pipeline_cache);
        let mut context =
            RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
                pass.name.clone(),
                RenderPassExecutorId::new(pass.executor_id.clone().unwrap()),
                pass.queue,
                pass.declared_queue,
                pass.flags,
                pass.dependencies.clone(),
                pass.resources.clone(),
            )
            .with_resource_resolver(&graph, pass.id)
            .with_gpu(gpu);

        record_ibl_bake_wgpu_pass_for_request(&mut context, request)
            .expect("IEM IBL WGPU pass should encode from graph resources");
    }

    backend.queue.submit(std::iter::once(encoder.finish()));
    backend
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("IBL IEM graph dispatch should finish");

    resources
}

fn request(contents: IblBakeArtifactContents) -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        16,
        5,
    )
    .with_required_contents(contents)
}

fn sh9_blob_for_request(request: &IblBakeArtifactRequest) -> IblBakeArtifactBlob {
    let descriptor = IblBakeArtifactDescriptor::current_for_request(request);
    let readback = IblBakeArtifactReadbackSections::new(descriptor)
        .with_irradiance_sh9_bytes(vec![0_u8; IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES]);
    IblBakeArtifactBlob::from_payload(
        readback
            .into_payload()
            .expect("SH9 readback sections should assemble"),
    )
}

fn create_source_cubemap_texture(device: &wgpu::Device) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ibl-bake-runtime-writeback-source-cubemap"),
        size: wgpu::Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn create_source_cubemap_texture_from_chain(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source: &SourceCubemapMipChain,
) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ibl-bake-runtime-writeback-pmrem-source-cubemap"),
        size: wgpu::Extent3d {
            width: source.source_face_size(),
            height: source.source_face_size(),
            depth_or_array_layers: 6,
        },
        mip_level_count: source.source_mip_count(),
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    for face in CubemapFace::ALL {
        for mip_level in 0..source.source_mip_count() {
            let mip_size = source_cubemap_mip_size(source.source_face_size(), mip_level);
            let offset = source_cubemap_face_mip_offset(
                source.source_face_size(),
                source.source_mip_count(),
                face,
                mip_level,
            );
            let mut rgba =
                Vec::with_capacity(mip_size as usize * mip_size as usize * RGBA8_BYTES_PER_TEXEL);
            for index in 0..mip_size as usize * mip_size as usize {
                let texel = source.source_texels()[offset + index];
                rgba.extend_from_slice(&rgba8_from_texel(texel));
            }
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: face.index() as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(mip_size * RGBA8_BYTES_PER_TEXEL as u32),
                    rows_per_image: Some(mip_size),
                },
                wgpu::Extent3d {
                    width: mip_size,
                    height: mip_size,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    texture
}

fn rgba8_from_texel(texel: [f32; 4]) -> [u8; 4] {
    [
        quantize_unorm8(texel[0]),
        quantize_unorm8(texel[1]),
        quantize_unorm8(texel[2]),
        quantize_unorm8(texel[3]),
    ]
}

fn quantize_unorm8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn source_cubemap_view_descriptor() -> wgpu::TextureViewDescriptor<'static> {
    source_cubemap_mip_view_descriptor(1)
}

fn source_cubemap_mip_view_descriptor(mip_count: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        label: Some("ibl-bake-runtime-writeback-source-cubemap-view"),
        format: Some(wgpu::TextureFormat::Rgba8Unorm),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(mip_count.max(1)),
        base_array_layer: 0,
        array_layer_count: Some(6),
    }
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

fn unique_temp_cache_root(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("zircon-{label}-{}-{nanos}", std::process::id()))
}
