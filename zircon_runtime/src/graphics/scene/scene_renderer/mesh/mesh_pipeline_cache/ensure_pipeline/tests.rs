use std::fs;
use std::sync::Arc;

use crate::asset::{pipeline::manager::ProjectAssetManager, AssetReference};
use crate::core::framework::render::{
    GeometrySourceBindingKind, GeometrySourceBindingRequirement, GeometrySourceDescriptor,
    GeometrySourceId, GeometrySourceVertexAttribute, RenderShaderDefinitionValue,
    ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantPrewarmManifest,
    GEOMETRY_SOURCE_PLUGIN_ID_START, SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::core::resource::ResourceId;
use crate::dynamic_api::{
    builtin_fallback_shader_prewarm_manifest,
    builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::resources::{
    default_pipeline_key, fallback_shader_uri, PipelineKey, ResourceStreamer,
    GPU_MATERIAL_UNIFORM_MIN_SIZE,
};
use crate::graphics::scene::scene_renderer::environment::scene_bind_group_layout_entries;
use crate::graphics::shader::{prewarm_shader_variants_to_disk, ShaderVariantCacheDisk};

use super::super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline_standard_material_template_source;
use super::super::MeshPipelineCache;
use super::mesh_shader_module_cache_key;

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
        cache
            .ensure_pipeline_for_variant(&device, &streamer, variant_id)
            .is_some(),
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
fn runtime_base_mesh_pipeline_keeps_builtin_fallback_on_standard_template_after_shader_stream() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
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
    let (shader_id, _, _) = streamer
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
        cache
            .ensure_pipeline_for_variant(&device, &streamer, variant_id)
            .is_some(),
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
        cache
            .ensure_oit_pipeline_for_base_variant(&device, &streamer, variant_id)
            .is_some(),
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
            cache
                .ensure_pipeline_for_variant(&device, &streamer, variant_id)
                .is_some(),
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
        cache
            .ensure_pipeline_for_variant(&device, &streamer, variant_id)
            .is_some(),
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
    let (shader_id, _, _) = streamer
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
            ensure_custom_geometry_pass_pipeline(
                &mut cache,
                &device,
                &streamer,
                kind,
                variant_id,
            ),
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
    let fallback_forward = builtin_fallback_shader_prewarm_manifest()
        .variants
        .into_iter()
        .find(|request| request.key.pass_type == ShaderPassType::Forward)
        .expect("builtin forward shader prewarm request");
    let variants = cases
        .iter()
        .map(|case| {
            let mut request = fallback_forward.clone();
            // Registry export must reach the actual cache key, not only the report label.
            request.key.material_shader = case.resource_id();
            request.key.material_revision = case.revision;
            request.source_label = case.locator.to_string();
            request
        })
        .collect();
    ShaderVariantPrewarmManifest::new(variants)
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
    match kind {
        MeshPassPipelineKind::GBuffer => cache
            .ensure_gbuffer_pipeline_for_variant(device, streamer, variant_id)
            .is_some(),
        MeshPassPipelineKind::DepthPrepass => cache
            .ensure_depth_prepass_pipeline_for_variant(device, streamer, variant_id)
            .is_some(),
        MeshPassPipelineKind::ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask => cache
            .ensure_shadow_pipeline_for_variant(device, streamer, variant_id)
            .is_some(),
        MeshPassPipelineKind::Velocity => cache
            .ensure_velocity_pipeline_for_variant(device, streamer, variant_id)
            .is_some(),
        MeshPassPipelineKind::TaaReactiveMask | MeshPassPipelineKind::TaaReactiveMaterialMask => {
            cache
                .ensure_taa_reactive_mask_pipeline_for_variant(device, streamer, variant_id)
                .is_some()
        }
        MeshPassPipelineKind::Base => false,
    }
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
