use std::fs;
use std::sync::Arc;

use crate::asset::{AssetReference, pipeline::manager::ProjectAssetManager};
use crate::core::framework::render::{
    GEOMETRY_SOURCE_PLUGIN_ID_START, GeometrySourceBindingKind, GeometrySourceBindingRequirement,
    GeometrySourceDescriptor, GeometrySourceId, GeometrySourceVertexAttribute,
    RenderShaderDefinitionValue, SHADING_MODEL_ID_STANDARD_PBR, ShaderFeatureBits, ShaderPassType,
    ShaderQualityTier, ShaderVariantPrewarmManifest,
};
use crate::core::resource::ResourceId;
use crate::dynamic_api::{
    builtin_fallback_shader_prewarm_manifest,
    builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::pipeline::{
    PipelineAdmission, PipelineAdmissionReason, PipelineAsyncQueueResult,
};
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::resources::{
    GPU_MATERIAL_UNIFORM_MIN_SIZE, PipelineKey, ResourceStreamer, default_pipeline_key,
    fallback_shader_uri,
};
use crate::graphics::scene::scene_renderer::environment::scene_bind_group_layout_entries;
use crate::graphics::shader::{ShaderVariantCacheDisk, prewarm_shader_variants_to_disk};

use super::super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline_standard_material_template_source;
use super::super::{
    MAX_ASYNC_BASE_PIPELINES_IN_FLIGHT, MAX_PENDING_PIPELINE_CREATION_DIAGNOSTICS,
    MeshPipelineCache, PipelineCreationTarget,
};
use super::mesh_shader_module_cache_key;

struct AsyncWorkerReleaseGuard {
    release: Option<std::sync::mpsc::SyncSender<()>>,
}

impl Drop for AsyncWorkerReleaseGuard {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
    }
}

#[test]
fn mesh_pipeline_template_source_hashes_feed_disk_and_module_keys() {
    let key = default_pipeline_key();
    let variant_key = key.shader_variant_key(ShaderPassType::Forward, "wgpu-test");

    let source = match mesh_pipeline_standard_material_template_source(&key) {
        Ok(source) => source,
        Err(error) => panic!("standard material template assembly failed: {error:?}"),
    };
    let module_key = mesh_shader_module_cache_key(&key, &variant_key, &source.source_hash);

    assert!(source.cache_content_hashes.len() > 1);
    assert!(source.cache_content_hashes.contains(&source.source_hash));
    assert!(module_key.contains(&source.source_hash));
    assert_eq!(source.template_revision, "zr-material-template-v1");
}

#[test]
fn runtime_base_mesh_pipeline_uses_staged_prewarm_without_compile_miss() {
    let root = std::env::temp_dir().join(format!(
        "zircon_runtime_base_mesh_staged_cache_hit_test_{}",
        std::process::id()
    ));
    let runtime_root = root.join("runtime");
    let staged_root = root.join("staged");
    let _ = fs::remove_dir_all(&root);

    let manifest = builtin_fallback_shader_prewarm_manifest();
    let prewarm_report = prewarm_shader_variants_to_disk(&manifest, &staged_root);
    assert_eq!(prewarm_report.requested_count, manifest.variants.len());
    assert_eq!(prewarm_report.written_count, manifest.variants.len());
    assert_eq!(prewarm_report.failed_count, 0);

    let Ok(backend) = RenderBackend::new_offscreen() else {
        let _ = fs::remove_dir_all(root);
        return;
    };
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    cache.shader_variant_disk_cache =
        ShaderVariantCacheDisk::with_fallback_roots(&runtime_root, [&staged_root]);

    cache.reset_shader_variant_miss_report();
    let pipeline_key = default_pipeline_key();
    let variant_id = cache.resolve_variant(
        MeshPassPipelineKind::Base,
        &pipeline_key,
        ShaderQualityTier::Medium,
    );
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

    assert!(
        ensure_base_pipeline_after_shader_validation(&mut cache, &device, &streamer, variant_id,)
            .is_ready(),
        "runtime Base mesh pipeline should be created from staged prewarm cache hit"
    );

    let error = pollster::block_on(error_scope.pop());
    assert!(
        error.is_none(),
        "staged prewarm cache hit source should create the runtime Base mesh WGPU pipeline: {error:?}"
    );
    let miss_report = cache.shader_variant_miss_report();
    assert_eq!(miss_report.request_count, 1);
    assert_eq!(miss_report.disk_hit_count, 1);
    assert_eq!(miss_report.compile_miss_count, 0);
    assert_eq!(miss_report.disk_write_count, 0);
    assert_eq!(miss_report.disk_error_count, 0);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn pipeline_creation_diagnostic_capacity_rollover_preserves_a_valid_new_pipeline() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    let pipeline_key = default_pipeline_key();
    let shader_variant_key = pipeline_key.shader_variant_key(ShaderPassType::Forward, "wgpu-test");
    let variant_id = MeshPipelineVariantId::new(u32::MAX - 1);

    for index in 0..MAX_PENDING_PIPELINE_CREATION_DIAGNOSTICS {
        cache.track_pipeline_creation_error_scope(
            &shader_variant_key,
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            variant_id,
            format!("pending-shader-{index}"),
            device.push_error_scope(wgpu::ErrorFilter::Validation),
        );
    }

    let saturated_shader_key = "saturated-shader".to_string();
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-saturated-diagnostic-test-shader"),
        source: wgpu::ShaderSource::Wgsl(
            "@vertex\nfn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {\n    let positions = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));\n    return vec4<f32>(positions[index], 0.0, 1.0);\n}\n".into(),
        ),
    });
    cache.shader_modules.insert(
        saturated_shader_key.clone(),
        super::super::shader_source_validation_admission::CachedMeshShaderModule::from_test_module(
            shader_module,
        ),
    );
    let saturated_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-saturated-diagnostic-test-pipeline"),
        layout: Some(&cache.mesh_pipeline_layout),
        vertex: wgpu::VertexState {
            module: cache
                .shader_modules
                .get(&saturated_shader_key)
                .expect("the saturation fixture must retain its shader module before tracking"),
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: None,
        multiview_mask: None,
        cache: None,
    });
    cache
        .mesh_variant_pipelines
        .insert(variant_id, saturated_pipeline);

    let validation_failed = cache.track_pipeline_creation_error_scope(
        &shader_variant_key,
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
        variant_id,
        saturated_shader_key.clone(),
        device.push_error_scope(wgpu::ErrorFilter::Validation),
    );

    assert!(!validation_failed);
    assert!(
        cache.shader_modules.contains_key(&saturated_shader_key),
        "resolved successful scopes must not evict a newly validated shader module"
    );
    assert!(
        cache.mesh_variant_pipelines.contains_key(&variant_id),
        "resolved successful scopes must not make a newly validated pipeline unavailable"
    );
    assert_eq!(
        cache.pending_pipeline_creation_diagnostics.len(),
        1,
        "capacity rollover must consume the previous resolved batch and retain the newest receipt"
    );
    assert!(
        cache
            .finish_pipeline_creation_diagnostics_for_variant(&shader_variant_key)
            .expect("the retained successful scope must finish without a validation error")
    );
}

#[test]
fn runtime_environment_only_pbr_base_prewarm_populates_the_renderer_cache() {
    let backend = RenderBackend::new_offscreen()
        .expect("offscreen backend required for environment-only PBR Base prewarm gate");
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

    let first_prewarm = cache
        .prewarm_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect("environment-only PBR prewarm must resolve the builtin shader revision");
    assert!(
        first_prewarm.pipeline_ready(),
        "environment-only PBR prewarm must create its runtime first-frame mesh pipelines"
    );
    assert!(
        first_prewarm.created_pipeline(),
        "the first prewarm must identify synchronous runtime PSO creation"
    );
    assert!(
        !first_prewarm.cache_hit(),
        "the empty renderer cache cannot report a reuse on its first prewarm"
    );
    assert!(
        first_prewarm.elapsed() >= first_prewarm.shader_source_resolution()
            && first_prewarm.elapsed() >= first_prewarm.pipeline_creation(),
        "the total prewarm timing must include separately reported source and PSO work"
    );
    let mut viewer_pipeline_key = default_pipeline_key();
    viewer_pipeline_key.shader_revision = streamer
        .resource_revision(viewer_pipeline_key.shader_id.clone())
        .expect("the prewarm must register the builtin PBR shader revision");
    viewer_pipeline_key.receive_shadows = false;
    let viewer_variant_id = cache.resolve_variant(
        MeshPassPipelineKind::Base,
        &viewer_pipeline_key,
        ShaderQualityTier::default(),
    );
    let (_, _, viewer_shader_variant) = cache
        .pipeline_and_shader_key_for_variant(viewer_variant_id)
        .expect("environment-only prewarm must retain its exact Base variant identity");
    assert!(
        viewer_shader_variant
            .features
            .contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR),
        "the environment-only renderer cache must reuse the reduced Forward shader variant"
    );
    assert_eq!(cache.mesh_variant_pipelines.len(), 1);
    assert!(
        cache
            .mesh_variant_pipelines
            .contains_key(&viewer_variant_id),
        "prewarm must populate the viewer's no-shadow-receiver Base variant"
    );
    let first_creation_metrics = cache.shader_variant_miss_report();
    assert_eq!(first_creation_metrics.render_pipeline_creation_count, 1);
    assert_eq!(first_creation_metrics.shader_module_creation_count, 1);
    let repeated_prewarm = cache
        .prewarm_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect("repeated environment-only PBR prewarm must resolve the builtin shader revision");
    assert!(
        repeated_prewarm.pipeline_ready(),
        "a repeated environment-only PBR prewarm must reuse the runtime cache"
    );
    assert!(
        repeated_prewarm.cache_hit(),
        "the repeated prewarm must identify the same renderer cache entry"
    );
    assert!(
        !repeated_prewarm.created_pipeline(),
        "a cache-hit prewarm must not claim a second synchronous PSO creation"
    );
    assert!(
        repeated_prewarm.elapsed() >= repeated_prewarm.shader_source_resolution()
            && repeated_prewarm.elapsed() >= repeated_prewarm.pipeline_creation(),
        "the cache-hit timing must retain its source and cache lookup accounting"
    );
    assert_eq!(cache.mesh_variant_pipelines.len(), 1);
    let repeated_creation_metrics = cache.shader_variant_miss_report();
    assert_eq!(
        repeated_creation_metrics.render_pipeline_creation_count,
        first_creation_metrics.render_pipeline_creation_count
    );
    assert_eq!(
        repeated_creation_metrics.shader_module_creation_count,
        first_creation_metrics.shader_module_creation_count
    );
    assert_eq!(
        repeated_creation_metrics.render_pipeline_creation_cpu_microseconds,
        first_creation_metrics.render_pipeline_creation_cpu_microseconds
    );
    assert_eq!(
        repeated_creation_metrics.shader_module_creation_cpu_microseconds,
        first_creation_metrics.shader_module_creation_cpu_microseconds
    );

    let viewer_shader_key = cache
        .shader_modules
        .keys()
        .next()
        .cloned()
        .expect("the prewarm must retain the Base shader module before its cache-hit check");
    let residual_error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let _ = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-prewarm-residual-validation-error"),
        source: wgpu::ShaderSource::Wgsl("this is intentionally invalid WGSL".into()),
    });
    cache.track_pipeline_creation_error_scope(
        &viewer_shader_variant,
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
        viewer_variant_id,
        viewer_shader_key.clone(),
        residual_error_scope,
    );
    let error = cache
        .prewarm_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect_err("a cache-hit prewarm must report a retained WGPU validation error");
    assert!(
        format!("{error:?}").contains("pipeline validation failed"),
        "the cache-hit prewarm must expose its retained pipeline validation error: {error:?}"
    );
    assert!(
        !cache
            .mesh_variant_pipelines
            .contains_key(&viewer_variant_id),
        "a retained WGPU error must evict the cache-hit Base pipeline"
    );
    assert!(
        !cache.shader_modules.contains_key(&viewer_shader_key),
        "a retained WGPU error must evict the cache-hit Base shader module"
    );

    let error = pollster::block_on(error_scope.pop());
    assert!(
        error.is_none(),
        "environment-only PBR runtime Base prewarm should pass WGPU validation: {error:?}"
    );
}

#[test]
fn runtime_environment_only_pbr_base_prewarm_stays_synchronous_when_async_compile_is_enabled() {
    let backend = RenderBackend::new_offscreen()
        .expect("offscreen backend required for environment-only PBR Base prewarm gate");
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    cache.set_async_pipeline_compile_enabled(true);

    let prewarm = cache
        .prewarm_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect("environment-only PBR prewarm must not defer draw admission");

    assert!(prewarm.pipeline_ready());
    assert!(prewarm.created_pipeline());
    assert!(cache.async_pipeline_compile_enabled());
    assert_eq!(cache.async_pipeline_compile_pending_count(), 0);
}

#[test]
fn runtime_environment_only_pbr_base_queue_uses_the_async_skip_draw_placeholder() {
    let backend = RenderBackend::new_offscreen()
        .expect("offscreen backend required for environment-only PBR Base queue gate");
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    cache.set_async_pipeline_compile_enabled(true);

    let queue_report = cache
        .queue_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect("environment-only PBR queue should resolve the builtin shader revision");

    assert!(!queue_report.pipeline_ready());
    assert!(!queue_report.cache_hit());
    assert!(cache.async_pipeline_compile_enabled());
    assert_eq!(cache.async_pipeline_compile_pending_count(), 1);
}

#[test]
fn runtime_environment_only_pbr_base_queue_never_falls_back_to_sync_when_worker_is_unavailable() {
    let backend = RenderBackend::new_offscreen()
        .expect("offscreen backend required for environment-only PBR Base queue gate");
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    cache
        .pipeline_variant_registry
        .enable_environment_only_pbr_base_profile();
    let mut pipeline_key = default_pipeline_key();
    let (_, shader_revision, shader_dependency_revision, _) = streamer
        .ensure_shader_source(&AssetReference::from_locator(fallback_shader_uri()))
        .expect("builtin PBR shader should resolve before the nonblocking queue test");
    pipeline_key.shader_revision = shader_revision;
    pipeline_key.shader_dependency_revision = shader_dependency_revision;
    pipeline_key.receive_shadows = false;
    let expected_variant = cache.resolve_variant(
        MeshPassPipelineKind::Base,
        &pipeline_key,
        ShaderQualityTier::default(),
    );
    cache.set_async_pipeline_compile_enabled(true);
    cache.async_base_pipeline_compiler = None;

    let queue_report = cache
        .queue_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect("a missing worker must preserve the nonblocking queue contract");

    assert!(!queue_report.pipeline_ready());
    assert!(matches!(
        cache.ensure_pipeline_admission_for_variant(&device, &streamer, expected_variant),
        PipelineAdmission::Failed(unavailable)
            if unavailable.reason() == PipelineAdmissionReason::WorkerUnavailable
    ));
    assert!(
        cache
            .shader_variant_miss_report()
            .pipeline_diagnostics()
            .iter()
            .any(|diagnostic| diagnostic
                .message
                .contains("async Base pipeline compiler is unavailable")),
        "worker loss must remain visible through the pipeline diagnostics"
    );
    let diagnostic_count = cache
        .shader_variant_miss_report()
        .pipeline_diagnostics()
        .len();
    assert!(matches!(
        cache.ensure_pipeline_admission_for_variant(&device, &streamer, expected_variant),
        PipelineAdmission::Failed(unavailable)
            if unavailable.reason() == PipelineAdmissionReason::WorkerUnavailable
    ));
    assert_eq!(
        cache
            .shader_variant_miss_report()
            .pipeline_diagnostics()
            .len(),
        diagnostic_count,
        "terminal background failures must not requeue or rediagnose every frame"
    );
    let error = cache
        .environment_only_pbr_base_pipeline_ready()
        .expect_err("one-shot evidence must receive a terminal worker error");
    assert!(
        error
            .to_string()
            .contains("async Base pipeline compiler is unavailable")
    );
}

#[test]
fn runtime_environment_only_pbr_base_queue_defers_admission_while_async_budget_is_full() {
    let backend = RenderBackend::new_offscreen()
        .expect("offscreen backend required for environment-only PBR Base queue gate");
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    let (blocker_started, wait_for_blocker_start) = std::sync::mpsc::sync_channel(0);
    let (release_blocker, wait_for_blocker_release) = std::sync::mpsc::sync_channel(0);
    let blocker_id = MeshPipelineVariantId::new(u32::MAX);
    assert_eq!(
        cache
            .async_base_pipeline_compiler
            .as_mut()
            .expect("the test backend should create its async pipeline worker")
            .try_queue(blocker_id, move || {
                blocker_started
                    .send(())
                    .expect("test should still observe the blocked worker");
                wait_for_blocker_release
                    .recv()
                    .expect("test should release the blocked worker");
                Err("test-only blocked predecessor".to_string())
            }),
        PipelineAsyncQueueResult::Queued
    );
    wait_for_blocker_start
        .recv()
        .expect("the predecessor should block the async compiler before filling its budget");
    let _blocker_release_guard = AsyncWorkerReleaseGuard {
        release: Some(release_blocker),
    };
    for offset in 1..MAX_ASYNC_BASE_PIPELINES_IN_FLIGHT {
        let queued_id = MeshPipelineVariantId::new(u32::MAX - offset as u32);
        assert_eq!(
            cache
                .async_base_pipeline_compiler
                .as_mut()
                .expect("the async compiler should remain available while its budget is filled")
                .try_queue(
                    queued_id,
                    || Err("test-only queued predecessor".to_string()),
                ),
            PipelineAsyncQueueResult::Queued
        );
    }
    assert_eq!(
        cache.async_pipeline_compile_pending_count(),
        MAX_ASYNC_BASE_PIPELINES_IN_FLIGHT as u32,
        "the blocked predecessor and queued successors must consume the full async budget"
    );
    cache
        .pipeline_variant_registry
        .enable_environment_only_pbr_base_profile();
    let mut pipeline_key = default_pipeline_key();
    let (_, shader_revision, shader_dependency_revision, _) = streamer
        .ensure_shader_source(&AssetReference::from_locator(fallback_shader_uri()))
        .expect("builtin PBR shader should resolve before the full-budget queue test");
    pipeline_key.shader_revision = shader_revision;
    pipeline_key.shader_dependency_revision = shader_dependency_revision;
    pipeline_key.receive_shadows = false;
    let expected_variant = cache.resolve_variant(
        MeshPassPipelineKind::Base,
        &pipeline_key,
        ShaderQualityTier::default(),
    );
    cache.set_async_pipeline_compile_enabled(true);

    let queue_report = cache
        .queue_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect("a full worker budget must preserve the nonblocking retry contract");

    assert!(!queue_report.pipeline_ready());
    assert!(matches!(
        cache.ensure_pipeline_admission_for_variant(&device, &streamer, expected_variant),
        PipelineAdmission::Deferred(unavailable)
            if unavailable.reason() == PipelineAdmissionReason::QueueSaturated
    ));
    assert_eq!(
        cache
            .environment_only_pbr_base_pipeline_ready()
            .expect("a full async budget must remain a recoverable admission state"),
        false,
        "the environment variant must remain pending until capacity is reclaimed"
    );
    assert!(
        cache
            .shader_variant_miss_report()
            .pipeline_diagnostics()
            .iter()
            .all(|diagnostic| !diagnostic
                .message
                .contains("queue remained full after draining ready completions")),
        "bounded queue backpressure must not become a terminal pipeline diagnostic"
    );
}

#[test]
fn runtime_environment_only_pbr_base_queue_retries_after_async_capacity_is_reclaimed() {
    let backend = RenderBackend::new_offscreen()
        .expect("offscreen backend required for environment-only PBR Base queue recovery");
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    let (blocker_started, wait_for_blocker_start) = std::sync::mpsc::sync_channel(0);
    let (release_blocker, wait_for_blocker_release) = std::sync::mpsc::sync_channel(0);
    let blocker_id = MeshPipelineVariantId::new(u32::MAX);
    assert_eq!(
        cache
            .async_base_pipeline_compiler
            .as_mut()
            .expect("the test backend should create its async pipeline worker")
            .try_queue(blocker_id, move || {
                blocker_started
                    .send(())
                    .expect("test should still observe the blocked worker");
                wait_for_blocker_release
                    .recv()
                    .expect("test should release the blocked worker");
                Err("test-only blocked predecessor".to_string())
            }),
        PipelineAsyncQueueResult::Queued
    );
    wait_for_blocker_start
        .recv()
        .expect("the predecessor should block the async compiler before filling its budget");
    let mut blocker_release_guard = AsyncWorkerReleaseGuard {
        release: Some(release_blocker.clone()),
    };
    let (successor_started, wait_for_successor_start) = std::sync::mpsc::sync_channel(0);
    let (release_successor, wait_for_successor_release) = std::sync::mpsc::sync_channel(0);
    let mut successor_release_guard = AsyncWorkerReleaseGuard {
        release: Some(release_successor.clone()),
    };
    let blocked_successor_id = MeshPipelineVariantId::new(u32::MAX - 1);
    assert_eq!(
        cache
            .async_base_pipeline_compiler
            .as_mut()
            .expect("the async compiler should accept the blocked successor")
            .try_queue(blocked_successor_id, move || {
                successor_started
                    .send(())
                    .expect("test should observe the blocked successor");
                wait_for_successor_release
                    .recv()
                    .expect("test should release the blocked successor");
                Err("test-only blocked successor".to_string())
            }),
        PipelineAsyncQueueResult::Queued
    );
    for offset in 2..MAX_ASYNC_BASE_PIPELINES_IN_FLIGHT {
        let queued_id = MeshPipelineVariantId::new(u32::MAX - offset as u32);
        assert_eq!(
            cache
                .async_base_pipeline_compiler
                .as_mut()
                .expect("the async compiler should remain available while its budget is filled")
                .try_queue(
                    queued_id,
                    || Err("test-only queued predecessor".to_string())
                ),
            PipelineAsyncQueueResult::Queued
        );
    }
    let (completion_observed, wait_for_completion) = std::sync::mpsc::channel();
    cache
        .async_base_pipeline_compiler
        .as_mut()
        .expect("the async compiler must observe the blocker completion")
        .set_completion_observer(completion_observed);
    cache.set_async_pipeline_compile_enabled(true);

    let queue_report = cache
        .queue_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect("a full async budget must retain a recoverable environment admission");
    assert!(!queue_report.pipeline_ready());
    let repeated_queue_report = cache
        .queue_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect("a repeated full-budget admission attempt must remain nonblocking");
    assert!(
        !repeated_queue_report.pipeline_ready(),
        "a repeated full-budget admission attempt must remain deferred"
    );
    release_blocker
        .send(())
        .expect("the blocked predecessor must still be waiting for its release");
    wait_for_completion
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("the predecessor completion must be enqueued before retrying admission");
    wait_for_successor_start
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("the worker must block on the successor before retry admission");
    blocker_release_guard.release = None;

    assert!(
        !cache
            .environment_only_pbr_base_pipeline_ready()
            .expect("reclaiming capacity must not create a terminal pipeline error"),
        "the target has not been admitted before the host retries it"
    );
    let retry_report = cache
        .queue_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect(
            "the next admission attempt must queue the environment variant after capacity returns",
        );
    assert!(!retry_report.pipeline_ready());
    assert_eq!(
        cache.async_pipeline_compile_pending_count(),
        MAX_ASYNC_BASE_PIPELINES_IN_FLIGHT as u32,
        "the retry must restore the full async budget by admitting the target instead of falling back synchronously"
    );
    release_successor
        .send(())
        .expect("the target must be admitted before releasing the successor");
    successor_release_guard.release = None;
    let prewarm = cache
        .prewarm_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect(
            "the retried environment variant must complete instead of retaining a full-budget failure",
        );
    assert!(prewarm.pipeline_ready());
    assert!(
        cache
            .environment_only_pbr_base_pipeline_ready()
            .expect("the recovered environment variant must not retain a full-queue failure")
    );
}

#[test]
fn runtime_environment_only_provider_fallback_resolves_generic_base_without_async_defer() {
    let backend = RenderBackend::new_offscreen()
        .expect("offscreen backend required for environment-only PBR provider fallback");
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    cache.set_async_pipeline_compile_enabled(true);
    cache
        .prewarm_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect("specialized Base prewarm must be ready before the provider upgrade");

    cache.disable_environment_only_pbr_base_profile();
    let mut pipeline_key = default_pipeline_key();
    let (_, shader_revision, shader_dependency_revision, _) = streamer
        .ensure_shader_source(&AssetReference::from_locator(fallback_shader_uri()))
        .expect("builtin PBR shader should remain streamed for generic fallback");
    pipeline_key.shader_revision = shader_revision;
    pipeline_key.shader_dependency_revision = shader_dependency_revision;
    pipeline_key.receive_shadows = false;
    let generic_variant = cache.resolve_variant(
        MeshPassPipelineKind::Base,
        &pipeline_key,
        ShaderQualityTier::default(),
    );

    assert!(
        !cache
            .pipeline_and_shader_key_for_variant(generic_variant)
            .expect("generic Base variant key after provider upgrade")
            .2
            .features
            .contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR)
    );
    assert!(
        ensure_base_pipeline_after_shader_validation(
            &mut cache,
            &device,
            &streamer,
            generic_variant,
        )
        .is_ready(),
        "provider fallback must not defer BaseScenePass admission"
    );
    assert_eq!(cache.async_pipeline_compile_pending_count(), 0);
}

#[test]
fn pbr_ior_forward_queue_tracks_the_generic_forward_base_variant() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    cache
        .pipeline_variant_registry
        .enable_environment_only_pbr_base_profile();
    cache.set_async_pipeline_compile_enabled(true);

    assert!(
        !cache
            .pbr_ior_forward_base_pipeline_ready()
            .expect("an unrequested IOR pipeline has no terminal failure")
    );
    cache
        .queue_pbr_ior_forward_base_pipeline(&device, &mut streamer)
        .expect("IOR queue admission should retain a generic Forward variant");
    assert!(
        !cache.environment_only_pbr_base_profile_enabled(),
        "the generic IOR queue must not enable the environment-only profile"
    );

    let variant_id = cache
        .pbr_ior_forward_base_pipeline_variant
        .expect("IOR queue stores its exact Base variant");
    let (_, _, shader_key) = cache
        .pipeline_and_shader_key_for_variant(variant_id)
        .expect("queued IOR variant keeps a shader key");
    assert!(
        !shader_key
            .features
            .contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR)
    );
    assert!(
        !shader_key
            .features
            .contains(ShaderFeatureBits::RECEIVE_SHADOWS),
        "the IOR readiness gate must match the viewer fixture's no-shadow-receiver key"
    );
    assert!(cache.base_pipeline_requires_forward_receiver(variant_id));
    assert!(
        cache.pbr_ior_forward_base_pipeline_ready().is_ok(),
        "the exact IOR readiness query remains nonblocking"
    );
}

#[test]
fn runtime_environment_only_pbr_base_prewarm_waits_for_its_queued_async_variant() {
    let backend = RenderBackend::new_offscreen()
        .expect("offscreen backend required for environment-only PBR Base prewarm gate");
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    let (blocker_started, wait_for_blocker_start) = std::sync::mpsc::sync_channel(0);
    let (release_blocker, wait_for_blocker_release) = std::sync::mpsc::sync_channel(0);
    let blocker_id = MeshPipelineVariantId::new(u32::MAX);
    assert_eq!(
        cache
            .async_base_pipeline_compiler
            .as_mut()
            .expect("the test backend should create its async pipeline worker")
            .try_queue(blocker_id, move || {
                blocker_started
                    .send(())
                    .expect("test should still observe the blocked worker");
                wait_for_blocker_release
                    .recv()
                    .expect("test should release the blocked worker");
                panic!("test-only predecessor must not install a pipeline")
            }),
        PipelineAsyncQueueResult::Queued
    );
    wait_for_blocker_start
        .recv()
        .expect("the predecessor should block the async compiler before queueing the target");
    let _blocker_release_guard = AsyncWorkerReleaseGuard {
        release: Some(release_blocker.clone()),
    };
    let mut pipeline_key = default_pipeline_key();
    let (_, shader_revision, shader_dependency_revision, _) = streamer
        .ensure_shader_source(&AssetReference::from_locator(fallback_shader_uri()))
        .expect("builtin PBR shader should stream before queueing its exact variant");
    pipeline_key.shader_revision = shader_revision;
    pipeline_key.shader_dependency_revision = shader_dependency_revision;
    pipeline_key.receive_shadows = false;
    cache
        .pipeline_variant_registry
        .enable_environment_only_pbr_base_profile();
    let variant_id = cache.resolve_variant(
        MeshPassPipelineKind::Base,
        &pipeline_key,
        ShaderQualityTier::default(),
    );
    cache.set_async_pipeline_compile_enabled(true);
    assert!(matches!(
        cache.ensure_pipeline_admission_for_variant(&device, &streamer, variant_id),
        PipelineAdmission::Deferred(unavailable)
            if unavailable.reason() == PipelineAdmissionReason::SourceValidationQueued
    ));
    cache.finish_pending_shader_source_validations();
    assert!(matches!(
        cache.ensure_pipeline_admission_for_variant(&device, &streamer, variant_id),
        PipelineAdmission::Deferred(unavailable)
            if unavailable.reason() == PipelineAdmissionReason::CompileQueued
    ));
    assert_eq!(cache.async_pipeline_compile_pending_count(), 2);
    cache
        .async_base_pipeline_compiler
        .as_mut()
        .expect("the async compiler should remain available")
        .set_target_sync_wait_observer(release_blocker);

    let prewarm = cache
        .prewarm_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect("prewarm must wait for its queued Base variant instead of reporting failure");

    assert!(prewarm.pipeline_ready());
    assert!(prewarm.cache_hit());
    assert!(!prewarm.created_pipeline());
    assert_eq!(cache.async_pipeline_compile_pending_count(), 0);
    let creation_metrics = cache.shader_variant_miss_report();
    assert_eq!(creation_metrics.render_pipeline_creation_count, 1);
    assert_eq!(creation_metrics.shader_module_creation_count, 1);
    assert_eq!(creation_metrics.async_base_pipeline_queue_wait_count, 1);
    assert!(creation_metrics.async_base_pipeline_queue_wait_microseconds > 0);
}

#[test]
fn runtime_environment_only_pbr_base_prewarm_reports_creation_after_async_target_failure() {
    let backend = RenderBackend::new_offscreen()
        .expect("offscreen backend required for environment-only PBR Base prewarm gate");
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    let mut pipeline_key = default_pipeline_key();
    let (_, shader_revision, shader_dependency_revision, _) = streamer
        .ensure_shader_source(&AssetReference::from_locator(fallback_shader_uri()))
        .expect("builtin PBR shader should stream before queueing its exact variant");
    pipeline_key.shader_revision = shader_revision;
    pipeline_key.shader_dependency_revision = shader_dependency_revision;
    pipeline_key.receive_shadows = false;
    cache
        .pipeline_variant_registry
        .enable_environment_only_pbr_base_profile();
    let variant_id = cache.resolve_variant(
        MeshPassPipelineKind::Base,
        &pipeline_key,
        ShaderQualityTier::default(),
    );
    assert_eq!(
        cache
            .async_base_pipeline_compiler
            .as_mut()
            .expect("the test backend should create its async pipeline worker")
            .try_queue(variant_id, || panic!("test-only async target failure")),
        PipelineAsyncQueueResult::Queued
    );

    let prewarm = cache
        .prewarm_environment_only_pbr_base_pipeline(&device, &mut streamer)
        .expect("prewarm should synchronously recover from its failed async target");

    assert!(prewarm.pipeline_ready());
    assert!(!prewarm.cache_hit());
    assert!(prewarm.created_pipeline());
    assert_eq!(cache.async_pipeline_compile_pending_count(), 0);
}

#[test]
fn runtime_base_mesh_pipeline_keeps_builtin_fallback_on_standard_template_after_shader_stream() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let pipeline_key = default_pipeline_key();
    let fallback_reference = AssetReference::from_locator(fallback_shader_uri());
    let (shader_id, _, _, _) = streamer
        .ensure_shader_source(&fallback_reference)
        .expect("builtin fallback shader should stream");
    assert_eq!(shader_id, pipeline_key.shader_id);
    assert!(
        streamer.shader_is_surface(&pipeline_key.shader_id),
        "regression setup should reproduce the streamed builtin shader asset"
    );

    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    let variant_id = cache.resolve_variant(
        MeshPassPipelineKind::Base,
        &pipeline_key,
        ShaderQualityTier::Medium,
    );
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

    assert!(
        ensure_base_pipeline_after_shader_validation(&mut cache, &device, &streamer, variant_id,)
            .is_ready(),
        "runtime Base mesh fallback pipeline should keep using the standard material template"
    );

    let error = pollster::block_on(error_scope.pop());
    assert!(
        error.is_none(),
        "streamed builtin fallback shader should not replace the standard material template: {error:?}"
    );
}

#[test]
fn runtime_oit_mesh_pipeline_creates_depth_only_fragment_store_variant_on_wgpu() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    if device.limits().max_storage_buffers_per_shader_stage < 3 {
        return;
    }
    let texture_layout = test_texture_bind_group_layout(&device);
    let streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    let mut pipeline_key = default_pipeline_key();
    pipeline_key.alpha_blend = true;
    let variant_id = cache.resolve_variant(
        MeshPassPipelineKind::Base,
        &pipeline_key,
        ShaderQualityTier::Medium,
    );
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

    assert!(
        ensure_oit_pipeline_after_shader_validation(&mut cache, &device, &streamer, variant_id,)
            .is_ready(),
        "transparent Base mesh should create the dedicated OIT fragment-store pipeline"
    );

    let error = pollster::block_on(error_scope.pop());
    assert!(
        error.is_none(),
        "OIT mesh fragment-store variant should pass WGPU validation: {error:?}"
    );
}

#[test]
fn runtime_project_plugin_registry_shader_keys_use_staged_prewarm_without_compile_miss() {
    let root = std::env::temp_dir().join(format!(
        "zircon_runtime_project_plugin_registry_staged_cache_hit_test_{}",
        std::process::id()
    ));
    let runtime_root = root.join("runtime");
    let staged_root = root.join("staged");
    let _ = fs::remove_dir_all(&root);

    let registry_cases = [
        RegistryShaderCase {
            locator: "res://project/shaders/project_shader",
            revision: 126_198_881_308_539_824,
        },
        RegistryShaderCase {
            locator: "package://native_dynamic_fixture/shaders/shader",
            revision: 14_843_875_089_575_827_114,
        },
    ];
    let manifest = registry_shader_prewarm_manifest(&registry_cases);
    let prewarm_report = prewarm_shader_variants_to_disk(&manifest, &staged_root);
    assert_eq!(prewarm_report.requested_count, registry_cases.len());
    assert_eq!(prewarm_report.written_count, registry_cases.len());
    assert_eq!(prewarm_report.failed_count, 0);
    assert_eq!(
        prewarm_report.source_provenance.source_count,
        registry_cases.len()
    );
    for case in registry_cases {
        assert_registry_shader_written(&prewarm_report, case);
    }

    let Ok(backend) = RenderBackend::new_offscreen() else {
        let _ = fs::remove_dir_all(root);
        return;
    };
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    cache.shader_variant_disk_cache =
        ShaderVariantCacheDisk::with_fallback_roots(&runtime_root, [&staged_root]);

    cache.reset_shader_variant_miss_report();
    for case in registry_cases {
        let pipeline_key = registry_shader_pipeline_key(case);
        let variant_id = cache.resolve_variant(
            MeshPassPipelineKind::Base,
            &pipeline_key,
            ShaderQualityTier::Medium,
        );
        let (_, _, variant_key) = cache
            .pipeline_and_shader_key_for_variant(variant_id)
            .expect("registry shader variant key");
        assert_eq!(variant_key.material_shader, case.resource_id());
        assert_eq!(variant_key.material_revision, case.revision);

        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        assert!(
            ensure_base_pipeline_after_shader_validation(
                &mut cache, &device, &streamer, variant_id,
            )
            .is_ready(),
            "runtime Base mesh pipeline should be created from staged registry shader cache for {}",
            case.locator
        );
        let error = pollster::block_on(error_scope.pop());
        assert!(
            error.is_none(),
            "staged registry shader cache should create the runtime Base WGPU pipeline for {}: {error:?}",
            case.locator
        );
    }

    let miss_report = cache.shader_variant_miss_report();
    assert_eq!(miss_report.request_count, registry_cases.len());
    assert_eq!(miss_report.disk_hit_count, registry_cases.len());
    assert_eq!(miss_report.compile_miss_count, 0);
    assert_eq!(miss_report.disk_write_count, 0);
    assert_eq!(miss_report.disk_error_count, 0);
    let forward_dimension = miss_report
        .dimension_summary
        .pass_types
        .get("forward")
        .expect("forward registry shader runtime dimension");
    assert_eq!(forward_dimension.request_count, registry_cases.len());
    assert_eq!(forward_dimension.disk_hit_count, registry_cases.len());
    assert_eq!(forward_dimension.compile_miss_count, 0);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_custom_geometry_descriptor_pipeline_uses_staged_prewarm_without_compile_miss() {
    let root = std::env::temp_dir().join(format!(
        "zircon_runtime_custom_geometry_staged_cache_hit_test_{}",
        std::process::id()
    ));
    let runtime_root = root.join("runtime");
    let staged_root = root.join("staged");
    let _ = fs::remove_dir_all(&root);

    let geometry_source = virtual_geometry_source_descriptor();
    let manifest = builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor(
        ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
        SHADING_MODEL_ID_STANDARD_PBR,
        None,
        &geometry_source,
        &[ShaderQualityTier::Medium],
    );
    let prewarm_report = prewarm_shader_variants_to_disk(&manifest, &staged_root);
    assert_eq!(prewarm_report.requested_count, manifest.variants.len());
    assert_eq!(prewarm_report.written_count, manifest.variants.len());
    assert_eq!(prewarm_report.failed_count, 0);

    let Ok(backend) = RenderBackend::new_offscreen() else {
        let _ = fs::remove_dir_all(root);
        return;
    };
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    cache.register_geometry_source_descriptor(geometry_source.clone());
    cache.shader_variant_disk_cache =
        ShaderVariantCacheDisk::with_fallback_roots(&runtime_root, [&staged_root]);

    cache.reset_shader_variant_miss_report();
    let pipeline_key = default_pipeline_key();
    let variant_id = cache.resolve_variant_for_geometry(
        MeshPassPipelineKind::Base,
        &pipeline_key,
        geometry_source.id,
        ShaderQualityTier::Medium,
    );
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

    assert!(
        ensure_base_pipeline_after_shader_validation(&mut cache, &device, &streamer, variant_id,)
            .is_ready(),
        "runtime Base mesh pipeline should be created for plugin-range geometry source id from staged cache"
    );

    let error = pollster::block_on(error_scope.pop());
    assert!(
        error.is_none(),
        "staged custom geometry source prewarm should create the runtime Base mesh WGPU pipeline: {error:?}"
    );
    let miss_report = cache.shader_variant_miss_report();
    assert_eq!(miss_report.request_count, 1);
    assert_eq!(miss_report.disk_hit_count, 1);
    assert_eq!(miss_report.compile_miss_count, 0);
    assert_eq!(miss_report.disk_write_count, 0);
    assert_eq!(miss_report.disk_error_count, 0);
    let geometry_dimension = miss_report
        .dimension_summary
        .geometry_source_ids
        .get(&geometry_source.id.value().to_string())
        .expect("custom geometry source runtime dimension");
    assert_eq!(geometry_dimension.request_count, 1);
    assert_eq!(geometry_dimension.disk_hit_count, 1);
    assert_eq!(geometry_dimension.compile_miss_count, 0);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_custom_geometry_descriptor_non_base_pipelines_use_staged_prewarm_without_compile_miss() {
    let root = std::env::temp_dir().join(format!(
        "zircon_runtime_custom_geometry_non_base_staged_cache_hit_test_{}",
        std::process::id()
    ));
    let runtime_root = root.join("runtime");
    let staged_root = root.join("staged");
    let _ = fs::remove_dir_all(&root);

    let geometry_source = virtual_geometry_source_descriptor();
    let manifest = builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor(
        ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
        SHADING_MODEL_ID_STANDARD_PBR,
        None,
        &geometry_source,
        &[ShaderQualityTier::Medium],
    );
    let prewarm_report = prewarm_shader_variants_to_disk(&manifest, &staged_root);
    assert_eq!(prewarm_report.requested_count, manifest.variants.len());
    assert_eq!(prewarm_report.written_count, manifest.variants.len());
    assert_eq!(prewarm_report.failed_count, 0);

    let Ok(backend) = RenderBackend::new_offscreen() else {
        let _ = fs::remove_dir_all(root);
        return;
    };
    let (system_textures, _) = backend
        .acquire_system_texture_lease()
        .expect("offscreen test backend must publish system textures");
    let RenderBackend { device, queue, .. } = backend;
    let texture_layout = test_texture_bind_group_layout(&device);
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::new(ProjectAssetManager::default()),
        &device,
        &queue,
        &texture_layout,
    );
    let scene_layout = test_scene_bind_group_layout(&device);
    let material_layout = test_standard_material_bind_group_layout(&device);
    let gpu_scene = test_gpu_scene(&device);
    let mut cache = MeshPipelineCache::new(
        &device,
        &queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    cache.register_geometry_source_descriptor(geometry_source.clone());
    cache.shader_variant_disk_cache =
        ShaderVariantCacheDisk::with_fallback_roots(&runtime_root, [&staged_root]);

    cache.reset_shader_variant_miss_report();
    let pipeline_key = default_pipeline_key();
    let fallback_reference = AssetReference::from_locator(fallback_shader_uri());
    let (shader_id, _, _, _) = streamer
        .ensure_shader_source(&fallback_reference)
        .expect("builtin fallback shader should stream before non-base pipeline validation");
    assert_eq!(shader_id, pipeline_key.shader_id);
    let pass_cases = [
        (MeshPassPipelineKind::GBuffer, "GBuffer"),
        (MeshPassPipelineKind::DepthPrepass, "DepthPrepass"),
        (MeshPassPipelineKind::ShadowDepth, "Shadow"),
        (MeshPassPipelineKind::Velocity, "Velocity"),
        (MeshPassPipelineKind::TaaReactiveMask, "TAA reactive mask"),
    ];

    for (kind, label) in pass_cases {
        let variant_id = cache.resolve_variant_for_geometry(
            kind,
            &pipeline_key,
            geometry_source.id,
            ShaderQualityTier::Medium,
        );
        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

        assert!(
            ensure_custom_geometry_pass_pipeline(&mut cache, &device, &streamer, kind, variant_id,),
            "{label} pipeline should be created for plugin-range geometry source id from staged cache"
        );

        let error = pollster::block_on(error_scope.pop());
        assert!(
            error.is_none(),
            "{label} staged custom geometry source prewarm should create the WGPU pipeline: {error:?}"
        );
    }

    let expected_pass_count = pass_cases.len();
    let miss_report = cache.shader_variant_miss_report();
    assert_eq!(miss_report.request_count, expected_pass_count);
    assert_eq!(miss_report.disk_hit_count, expected_pass_count);
    assert_eq!(miss_report.compile_miss_count, 0);
    assert_eq!(miss_report.disk_write_count, 0);
    assert_eq!(miss_report.disk_error_count, 0);
    let geometry_dimension = miss_report
        .dimension_summary
        .geometry_source_ids
        .get(&geometry_source.id.value().to_string())
        .expect("custom geometry source runtime dimension");
    assert_eq!(geometry_dimension.request_count, expected_pass_count);
    assert_eq!(geometry_dimension.disk_hit_count, expected_pass_count);
    assert_eq!(geometry_dimension.compile_miss_count, 0);

    let _ = fs::remove_dir_all(root);
}

#[derive(Clone, Copy)]
struct RegistryShaderCase {
    locator: &'static str,
    revision: u64,
}

impl RegistryShaderCase {
    fn resource_id(self) -> ResourceId {
        ResourceId::from_stable_label(self.locator)
    }
}

fn registry_shader_prewarm_manifest(cases: &[RegistryShaderCase]) -> ShaderVariantPrewarmManifest {
    let fallback_manifest = builtin_fallback_shader_prewarm_manifest();
    let fallback_forward = fallback_manifest
        .variants
        .iter()
        .find(|request| request.key.pass_type == ShaderPassType::Forward)
        .expect("builtin forward shader prewarm request")
        .clone();
    let fallback_source = fallback_manifest
        .source_for(&fallback_forward)
        .expect("builtin forward shader prewarm source");
    let mut sources = Vec::with_capacity(cases.len());
    let variants = cases
        .iter()
        .map(|case| {
            let source = fallback_source.with_source_label(case.locator);
            let mut request = fallback_forward.clone();
            // Registry export must reach the actual cache key, not only the report label.
            request.key.material_shader = case.resource_id();
            request.key.material_revision = case.revision;
            request.source_id = source.id.clone();
            sources.push(source);
            request
        })
        .collect();
    ShaderVariantPrewarmManifest::new(sources, variants)
}

fn registry_shader_pipeline_key(case: RegistryShaderCase) -> PipelineKey {
    let mut pipeline_key = default_pipeline_key();
    pipeline_key.shader_id = case.resource_id();
    pipeline_key.shader_revision = case.revision;
    pipeline_key
}

fn assert_registry_shader_written(
    report: &crate::core::framework::render::ShaderVariantPrewarmReport,
    case: RegistryShaderCase,
) {
    let written = report
        .written_variants
        .iter()
        .find(|variant| variant.source_label == case.locator)
        .unwrap_or_else(|| panic!("registry source {} should be written", case.locator));
    let material_id = case.resource_id().to_string();
    assert!(
        written.canonical_string.contains(material_id.as_str()),
        "written cache key should include registry material id for {}; canonical={}",
        case.locator,
        written.canonical_string
    );
    assert!(
        written
            .canonical_string
            .contains(&format!("|revision={}", case.revision)),
        "written cache key should include registry revision for {}; canonical={}",
        case.locator,
        written.canonical_string
    );
    let provenance = report
        .source_provenance
        .sources
        .values()
        .find(|entry| entry.source_label == case.locator)
        .unwrap_or_else(|| panic!("registry provenance for {}", case.locator));
    assert_eq!(provenance.requested_count, 1);
    assert_eq!(provenance.written_count, 1);
    assert_eq!(provenance.failed_count, 0);
}

fn ensure_custom_geometry_pass_pipeline(
    cache: &mut MeshPipelineCache,
    device: &wgpu::Device,
    streamer: &ResourceStreamer,
    kind: MeshPassPipelineKind,
    variant_id: MeshPipelineVariantId,
) -> bool {
    let first_ready = match kind {
        MeshPassPipelineKind::GBuffer => cache
            .ensure_gbuffer_pipeline_admission_for_variant(device, streamer, variant_id)
            .is_ready(),
        MeshPassPipelineKind::DepthPrepass => cache
            .ensure_depth_prepass_pipeline_admission_for_variant(device, streamer, variant_id)
            .is_ready(),
        MeshPassPipelineKind::HitProxy => cache
            .ensure_hit_proxy_pipeline_admission_for_variant(device, streamer, variant_id)
            .is_ready(),
        MeshPassPipelineKind::ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask => cache
            .ensure_shadow_pipeline_admission_for_variant(device, streamer, kind, variant_id)
            .is_ready(),
        MeshPassPipelineKind::Velocity => cache
            .ensure_velocity_pipeline_admission_for_variant(device, streamer, variant_id)
            .is_ready(),
        MeshPassPipelineKind::TaaReactiveMask | MeshPassPipelineKind::TaaReactiveMaterialMask => {
            cache
                .ensure_taa_reactive_pipeline_admission_for_variant(
                    device, streamer, kind, variant_id,
                )
                .is_ready()
        }
        MeshPassPipelineKind::Base => false,
    };
    if first_ready {
        return true;
    }
    cache.finish_pending_shader_source_validations();
    match kind {
        MeshPassPipelineKind::GBuffer => cache
            .ensure_gbuffer_pipeline_admission_for_variant(device, streamer, variant_id)
            .is_ready(),
        MeshPassPipelineKind::DepthPrepass => cache
            .ensure_depth_prepass_pipeline_admission_for_variant(device, streamer, variant_id)
            .is_ready(),
        MeshPassPipelineKind::HitProxy => cache
            .ensure_hit_proxy_pipeline_admission_for_variant(device, streamer, variant_id)
            .is_ready(),
        MeshPassPipelineKind::ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask => cache
            .ensure_shadow_pipeline_admission_for_variant(device, streamer, kind, variant_id)
            .is_ready(),
        MeshPassPipelineKind::Velocity => cache
            .ensure_velocity_pipeline_admission_for_variant(device, streamer, variant_id)
            .is_ready(),
        MeshPassPipelineKind::TaaReactiveMask | MeshPassPipelineKind::TaaReactiveMaterialMask => {
            cache
                .ensure_taa_reactive_pipeline_admission_for_variant(
                    device, streamer, kind, variant_id,
                )
                .is_ready()
        }
        MeshPassPipelineKind::Base => false,
    }
}

fn ensure_base_pipeline_after_shader_validation(
    cache: &mut MeshPipelineCache,
    device: &wgpu::Device,
    streamer: &ResourceStreamer,
    variant_id: MeshPipelineVariantId,
) -> PipelineAdmission<()> {
    let admission = cache.ensure_pipeline_admission_for_variant(device, streamer, variant_id);
    if !matches!(
        admission.unavailable_details().map(|value| value.reason()),
        Some(
            PipelineAdmissionReason::SourceValidationQueued
                | PipelineAdmissionReason::SourceValidationPending
        )
    ) {
        return admission;
    }
    cache.finish_pending_shader_source_validations();
    cache.ensure_pipeline_admission_for_variant(device, streamer, variant_id)
}

fn ensure_oit_pipeline_after_shader_validation(
    cache: &mut MeshPipelineCache,
    device: &wgpu::Device,
    streamer: &ResourceStreamer,
    variant_id: MeshPipelineVariantId,
) -> PipelineAdmission<()> {
    let admission =
        cache.ensure_oit_pipeline_admission_for_base_variant(device, streamer, variant_id);
    if !matches!(
        admission.unavailable_details().map(|value| value.reason()),
        Some(
            PipelineAdmissionReason::SourceValidationQueued
                | PipelineAdmissionReason::SourceValidationPending
        )
    ) {
        return admission;
    }
    cache.finish_pending_shader_source_validations();
    cache.ensure_oit_pipeline_admission_for_base_variant(device, streamer, variant_id)
}

fn virtual_geometry_source_descriptor() -> GeometrySourceDescriptor {
    GeometrySourceDescriptor {
        id: GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START),
        token: "custom:virtual_geometry".to_string(),
        wgsl_include: "zr_geometry_virtual_geometry.wgsl".to_string(),
        vertex_attributes: vec![
            GeometrySourceVertexAttribute::Position,
            GeometrySourceVertexAttribute::Normal,
            GeometrySourceVertexAttribute::Tangent,
            GeometrySourceVertexAttribute::Uv0,
        ],
        required_bindings: vec![
            GeometrySourceBindingRequirement::new(
                GeometrySourceBindingKind::VirtualGeometryPages,
                "virtual_geometry.pages",
            ),
            GeometrySourceBindingRequirement::new(
                GeometrySourceBindingKind::VirtualGeometryClusters,
                "virtual_geometry.clusters",
            ),
        ],
        shader_defines: vec![RenderShaderDefinitionValue::bool(
            "ZR_GEOMETRY_SOURCE_VIRTUAL_GEOMETRY",
            true,
        )],
    }
}

fn test_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-test-runtime-staged-cache-texture-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

fn test_scene_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let scene_layout_entries = scene_bind_group_layout_entries();
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-test-runtime-staged-cache-scene-layout"),
        entries: &scene_layout_entries,
    })
}

fn test_standard_material_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-test-runtime-staged-cache-material-layout"),
        entries: &[
            material_uniform_entry(0),
            material_texture_entry(1),
            material_sampler_entry(2),
            material_texture_entry(3),
            material_sampler_entry(4),
            material_texture_entry(5),
            material_sampler_entry(6),
            material_texture_entry(7),
            material_sampler_entry(8),
            material_texture_entry(9),
            material_sampler_entry(10),
            material_texture_entry(11),
            material_sampler_entry(12),
        ],
    })
}

fn material_texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        },
        count: None,
    }
}

fn material_sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn material_uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(GPU_MATERIAL_UNIFORM_MIN_SIZE as u64),
        },
        count: None,
    }
}

fn test_gpu_scene(device: &wgpu::Device) -> GpuScene {
    GpuScene::new(
        device,
        Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-runtime-staged-cache-joint-palette"),
            size: 256 * 64 + 16,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        })),
        wgpu::BufferSize::new(256 * 64 + 16).expect("test joint palette size"),
    )
}
