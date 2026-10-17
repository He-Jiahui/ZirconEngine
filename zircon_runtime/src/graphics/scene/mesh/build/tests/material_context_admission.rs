use crate::graphics::scene::resources::MaterialDrawGenerationSelection;

use super::{generation_selection_for_current, MaterialGenerationRequirementCacheStats};

#[test]
fn generation_requirement_cache_miss_count_is_the_observed_remainder() {
    let stats = MaterialGenerationRequirementCacheStats {
        hit_material_count: 2,
        observed_requirement_count: 11,
        hit_requirement_count: 7,
    };

    assert_eq!(stats.miss_requirement_count(), 4);
}

#[test]
fn current_context_admission_uses_atomic_previous_or_error_fallback() {
    assert_eq!(
        generation_selection_for_current(true, true),
        MaterialDrawGenerationSelection::Published
    );
    assert_eq!(
        generation_selection_for_current(false, true),
        MaterialDrawGenerationSelection::PreviousPublished
    );
    assert_eq!(
        generation_selection_for_current(false, false),
        MaterialDrawGenerationSelection::ErrorProxy
    );
}

#[test]
fn context_admission_selects_complete_previous_or_error_material_proxies() {
    let source = include_str!("../material_context_admission.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material context admission test boundary");

    assert!(source.contains("MaterialPipelinePublicationAdmission::Deferred"));
    assert!(source.contains("MaterialPipelinePublicationAdmission::Failed"));
    assert!(source.contains("MaterialDrawGenerationSelection::PreviousPublished"));
    assert!(source.contains("MaterialDrawGenerationSelection::ErrorProxy"));
    assert!(source.contains("collect_previous_context_pipeline_requirements"));
    assert!(source.contains("ensure_material_pipeline_requirements_for_generation"));
    // BUG: [CR-R02-runtime_wave12_graphics_mesh_draw_build-0002] 当前生产准入读取已冻结的 census 代际，已无 draw_generation()；前六个断言均成立后本行固定失败，守卫须追踪实际收集入口。
    let context_selection = source
        .split("fn select_material_generations_for_context(")
        .nth(1)
        .and_then(|body| {
            body.split("fn retain_material_pipeline_requirement_misses(")
                .next()
        })
        .expect("context selection body");
    let frozen_rows = context_selection
        .find("current_census.into_requirements()")
        .expect("freeze the observed material generation and exact requirements");
    let initial_admission = context_selection
        .find("admit_frozen_rows(")
        .expect("admit the frozen current rows");
    assert!(frozen_rows < initial_admission);
    assert!(context_selection.contains("&current_rows"));
    let frozen_admission = source
        .split("fn admit_frozen_rows(")
        .nth(1)
        .and_then(|body| body.split("type FrozenMaterialRequirementRow").next())
        .expect("frozen requirement admission body");
    assert!(frozen_admission.contains("rows.iter()"));
    let generation_call = frozen_admission
        .split("ensure_material_pipeline_requirements_for_generation(")
        .nth(1)
        .and_then(|body| body.split(");").next())
        .expect("generation-qualified admission of each frozen row");
    assert!(generation_call.contains("*material_id"));
    assert!(generation_call.contains("*generation"));
    assert!(generation_call.contains("requirements"));
    assert!(source.contains("\"material\", \"context_admission\""));
}

#[test]
fn context_admission_profiles_fused_census_and_generation_cache_scale() {
    let source = include_str!("../material_context_admission.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material context admission test boundary");

    assert!(!source.contains("context_admission_material_count"));
    assert!(source.contains("material_context_admission_tracked_material_count"));
    assert!(source.contains("material_context_admission_scanned_draw_count"));
    assert!(source.contains("material_context_admission_candidate_count"));
    assert!(source.contains("material_context_admission_generation_cache_hit_count"));
    assert!(source.contains("material_context_admission_generation_cache_miss_count"));
    assert!(source.contains("material_context_admission_observed_requirement_count"));
    assert!(source.contains("material_context_admission_generation_cache_hit_requirement_count"));
    assert!(source.contains("material_context_admission_generation_cache_miss_requirement_count"));
}

#[test]
fn context_admission_consumes_fused_generation_ledger_misses() {
    let source = include_str!("../material_context_admission.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material context admission test boundary");
    let selection = source
        .split("fn select_material_generations_for_context(")
        .nth(1)
        .expect("context material selection");

    assert!(selection.contains("current_census: MaterialPipelineRequirementCensus"));
    assert!(selection.contains("retain_material_pipeline_requirement_misses"));
    assert!(!selection.contains("collect_published_context_pipeline_requirements"));
    assert!(!selection.contains("context_admission_material_count"));
}

#[test]
fn context_selection_precedes_every_gpu_and_command_cache_projection() {
    let build = include_str!("../../../renderer/mesh/build_mesh_draws/build/build.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("mesh build test boundary");
    let selection = build
        .find("select_material_generations_for_context(")
        .expect("context material selection");
    let virtual_geometry = build
        .find("build_virtual_geometry_indirect_draw_plan(")
        .expect("virtual geometry plan");
    let gpu_scene = build
        .find("sync_gpu_scene_pending_draws(")
        .expect("GPUScene projection");
    let command_cache = build
        .find("extract_pending_static_mesh_command_cache_hits(")
        .expect("command cache projection");

    assert!(selection < virtual_geometry);
    assert!(selection < gpu_scene);
    assert!(selection < command_cache);
}

#[test]
fn error_proxy_requirements_are_synchronously_admitted_before_selection_returns() {
    let source = include_str!("../material_context_admission.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material context admission test boundary");
    let collect = source
        .find("collect_error_proxy_context_pipeline_requirements(")
        .expect("error proxy requirement census");
    let admit = source
        .find("ensure_error_proxy_pipeline_requirements(")
        .expect("synchronous error proxy admission");
    let finish = source
        .rfind("Ok((selection, stats))")
        .expect("fallible context selection result");

    assert!(collect < admit);
    assert!(admit < finish);
}

fn with_native_material_admission_fixture(
    run: impl FnOnce(
        &wgpu::Device,
        &mut crate::graphics::scene::resources::ResourceStreamer,
        &mut crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache,
        &crate::asset::pipeline::manager::ProjectAssetManager,
        &crate::graphics::backend::RenderBackend,
        &wgpu::BindGroupLayout,
    ),
) {
    use crate::asset::pipeline::manager::ProjectAssetManager;
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::scene::gpu_scene::GpuScene;
    use crate::graphics::scene::resources::{ResourceStreamer, GPU_MATERIAL_UNIFORM_MIN_SIZE};
    use crate::graphics::scene::scene_renderer::environment::scene_bind_group_layout_entries;
    use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
    use std::sync::Arc;

    let backend =
        RenderBackend::new_offscreen().expect("material admission requires a real GPU backend");
    let (system_textures, _system_lease) = backend
        .acquire_system_texture_lease()
        .expect("system textures");
    let device = &backend.device;
    let queue = &backend.queue;
    let texture_entry = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let sampler_entry = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    };
    let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("material-admission-test-textures"),
        entries: &[texture_entry(0), sampler_entry(1)],
    });
    let manager = Arc::new(ProjectAssetManager::default());
    let mut streamer =
        ResourceStreamer::new_for_test(Arc::clone(&manager), device, queue, &texture_layout);
    let scene_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("material-admission-test-scene"),
        entries: &scene_bind_group_layout_entries(),
    });
    let mut entries = vec![wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(GPU_MATERIAL_UNIFORM_MIN_SIZE as u64),
        },
        count: None,
    }];
    for binding in 1..13 {
        entries.push(if binding % 2 == 1 {
            texture_entry(binding)
        } else {
            sampler_entry(binding)
        });
    }
    let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("material-admission-test-material"),
        entries: &entries,
    });
    let palette_size = 256 * 64 + 16;
    let gpu_scene = GpuScene::new(
        device,
        Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("material-admission-test-joints"),
            size: palette_size,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        })),
        wgpu::BufferSize::new(palette_size).expect("joint palette extent"),
    );
    let mut cache = MeshPipelineCache::new(
        device,
        queue,
        &system_textures,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        &scene_layout,
        &material_layout,
        gpu_scene.scene_bind_group_layout(),
    );
    cache.set_async_pipeline_compile_enabled(false);
    run(
        device,
        &mut streamer,
        &mut cache,
        manager.as_ref(),
        &backend,
        &texture_layout,
    );
}

fn native_admission_requirements(
    key: crate::graphics::scene::resources::PipelineKey,
) -> crate::graphics::scene::scene_renderer::mesh::MaterialPipelineRequirementSet {
    use crate::core::framework::render::{ShaderQualityTier, GEOMETRY_SOURCE_ID_STATIC_MESH};
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassPipelineKind;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::PipelineCreationTarget;
    use crate::graphics::scene::scene_renderer::mesh::{
        MaterialPipelineRequirement, MaterialPipelineRequirementSet,
    };
    let mut requirements = MaterialPipelineRequirementSet::default();
    requirements.insert(MaterialPipelineRequirement::new(
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
        key,
        GEOMETRY_SOURCE_ID_STATIC_MESH,
        ShaderQualityTier::default(),
    ));
    requirements
}

#[test]
fn cold_current_requirements_are_rechecked_after_existing_error_proxy_validation() {
    use super::{
        MaterialContextAdmissionStats, MaterialDrawSelection, MaterialPipelinePublicationAdmission,
    };
    use crate::asset::AssetReference;
    use crate::core::resource::ResourceId;
    use crate::graphics::pipeline::PipelineAdmissionReason;
    use crate::graphics::scene::resources::{default_pipeline_key, fallback_shader_uri};

    with_native_material_admission_fixture(
        |device, streamer, cache, _manager, _backend, _layout| {
            let mut key = default_pipeline_key();
            let (_, revision, identity, _) = streamer
                .ensure_shader_source(&AssetReference::from_locator(fallback_shader_uri()))
                .expect("the real built-in material source");
            key.shader_revision = revision;
            key.shader_dependency_identity = Some(identity);
            let material_id = ResourceId::from_stable_label("test://cold-current-admission");
            let generation = 7;
            let rows = vec![(material_id, generation, native_admission_requirements(key))];
            let first = cache.ensure_material_pipeline_requirements_for_generation(
                device,
                streamer,
                material_id,
                generation,
                &rows[0].2,
            );
            assert!(
                matches!(first, MaterialPipelinePublicationAdmission::Deferred { unavailable, .. }
            if unavailable.reason() == PipelineAdmissionReason::SourceValidationQueued)
            );
            let mut selection = MaterialDrawSelection::default();
            selection.select(material_id, MaterialDrawGenerationSelection::ErrorProxy);
            let mut stats = MaterialContextAdmissionStats {
                error_proxy_selected_count: 1,
                ..Default::default()
            };
            super::record_material_pipeline_admission(&mut stats, first);
            let initial_stats = stats;
            cache
                .ensure_error_proxy_pipeline_requirements(device, streamer, &rows[0].2)
                .expect("the existing mandatory fallback validation completes the cold source");
            super::retry_current_requirements_after_error_proxy_validation(
                device,
                streamer,
                cache,
                &rows,
                &[(material_id, first)],
                &mut selection,
                &mut stats,
            );
            assert_eq!(
                selection.selection_for(&material_id),
                MaterialDrawGenerationSelection::Published
            );
            assert_eq!(stats.current_ready_count, 1);
            assert_eq!(stats.error_proxy_selected_count, 0);
            assert_eq!(stats.requirement_count - initial_stats.requirement_count, 1);
            assert_eq!(
                stats.ready_requirement_count - initial_stats.ready_requirement_count,
                1
            );
            assert_eq!(stats.deferred_count, initial_stats.deferred_count);
            assert_eq!(stats.failed_count, initial_stats.failed_count);
            assert_eq!(rows[0].1, generation);
            assert_eq!(rows[0].2.len(), 1);
        },
    );
}

#[test]
fn unavailable_current_shader_retains_the_complete_error_proxy_after_retry() {
    use super::{
        MaterialContextAdmissionStats, MaterialDrawSelection, MaterialPipelinePublicationAdmission,
    };
    use crate::core::resource::ResourceId;
    use crate::graphics::scene::resources::default_pipeline_key;

    with_native_material_admission_fixture(
        |device, streamer, cache, manager, _backend, _layout| {
            let material_id = ResourceId::from_stable_label("test://unavailable-current-admission");
            let mut key = default_pipeline_key();
            let registered_shader_id = register_invalid_admission_shader(manager);
            let reference = crate::asset::AssetReference::from_locator(
                crate::core::resource::ResourceLocator::parse(
                    "res://shaders/material-admission-invalid.wgsl",
                )
                .expect("actual invalid shader locator"),
            );
            let (shader_id, revision, identity, report) = streamer
                .ensure_shader_source(&reference)
                .expect("load the registered invalid source without falling back");
            assert_eq!(shader_id, registered_shader_id);
            assert!(report.is_none());
            key.shader_id = shader_id;
            key.shader_revision = revision;
            key.shader_dependency_identity = Some(identity);
            let rows = vec![(material_id, 9, native_admission_requirements(key))];
            cache
                .ensure_error_proxy_pipeline_requirements(device, streamer, &rows[0].2)
                .expect_err("registered invalid WGSL must fail real source validation");
            let first = cache.ensure_material_pipeline_requirements_for_generation(
                device,
                streamer,
                material_id,
                9,
                &rows[0].2,
            );
            assert!(!matches!(
                first,
                MaterialPipelinePublicationAdmission::Ready { .. }
            ));
            assert!(matches!(
                first,
                MaterialPipelinePublicationAdmission::Failed { .. }
            ));
            assert!(!super::current_admission_can_retry(first));
            let mut selection = MaterialDrawSelection::default();
            selection.select(material_id, MaterialDrawGenerationSelection::ErrorProxy);
            let mut stats = MaterialContextAdmissionStats {
                error_proxy_selected_count: 1,
                ..Default::default()
            };
            super::record_material_pipeline_admission(&mut stats, first);
            let initial_stats = stats;
            cache
                .ensure_error_proxy_pipeline_requirements(
                    device,
                    streamer,
                    &native_admission_requirements(default_pipeline_key()),
                )
                .expect("the independent error proxy must remain available");
            super::retry_current_requirements_after_error_proxy_validation(
                device,
                streamer,
                cache,
                &rows,
                &[(material_id, first)],
                &mut selection,
                &mut stats,
            );
            assert_eq!(
                selection.selection_for(&material_id),
                MaterialDrawGenerationSelection::ErrorProxy
            );
            assert!(selection.proxy(streamer, &material_id).runtime().is_none());
            assert_eq!(stats.current_ready_count, 0);
            assert_eq!(stats.error_proxy_selected_count, 1);
            assert_eq!(stats.requirement_count, initial_stats.requirement_count);
            assert_eq!(
                stats.ready_requirement_count,
                initial_stats.ready_requirement_count
            );
            assert_eq!(stats.deferred_count, initial_stats.deferred_count);
            assert_eq!(stats.failed_count, initial_stats.failed_count);
            assert_eq!(rows[0].1, 9);
        },
    );
}

fn register_invalid_admission_shader(
    manager: &crate::asset::pipeline::manager::ProjectAssetManager,
) -> crate::core::resource::ResourceId {
    use crate::asset::{ShaderAsset, ShaderSourceLanguage};
    use crate::core::framework::render::ShaderAssetKind;
    use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceRecord};
    let locator = ResourceLocator::parse("res://shaders/material-admission-invalid.wgsl")
        .expect("registered invalid shader locator");
    let id = ResourceId::from_locator(&locator);
    manager
        .resource_manager()
        .register_ready(
            ResourceRecord::new(id, ResourceKind::Shader, locator.clone())
                .with_source_hash("material-admission-invalid-source-v1"),
            ShaderAsset {
                uri: locator,
                kind: ShaderAssetKind::Include,
                source_language: ShaderSourceLanguage::Wgsl,
                source: "this is not valid WGSL;".to_string(),
                wgsl_source: String::new(),
                import_path: None,
                entry_points: Vec::new(),
                dependencies: Vec::new(),
                source_files: Vec::new(),
                imports: Vec::new(),
                shader_defs: Vec::new(),
                property_schema: Vec::new(),
                options: Vec::new(),
                texture_slots: Vec::new(),
                shading_model: None,
                render_state: Default::default(),
                queue: None,
                disabled_passes: Vec::new(),
                resources: Vec::new(),
                material_property_layout: Default::default(),
                material_option_table: Default::default(),
                generated_material_wgsl: String::new(),
                editor: Default::default(),
                pipeline_layout: Default::default(),
                validation_diagnostics: Vec::new(),
            },
        )
        .expect("register actual invalid WGSL source");
    id
}

fn prepare_admission_fixture_material(
    manager: &crate::asset::pipeline::manager::ProjectAssetManager,
    streamer: &mut crate::graphics::scene::resources::ResourceStreamer,
    backend: &crate::graphics::backend::RenderBackend,
    texture_layout: &wgpu::BindGroupLayout,
    source_hash: &str,
    tint: [f32; 4],
    shader_uri: &str,
) -> (crate::core::resource::ResourceId, u64, u64) {
    use crate::asset::{AlphaMode, AssetReference, MaterialAsset};
    use crate::core::resource::{
        MaterialMarker, ResourceHandle, ResourceId, ResourceKind, ResourceLocator, ResourceRecord,
    };
    let locator = ResourceLocator::parse("res://materials/cold-context-admission.mat")
        .expect("fixture material locator");
    let id = ResourceId::from_locator(&locator);
    manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(id, ResourceKind::Material, locator).with_source_hash(source_hash),
            MaterialAsset {
                name: Some("Cold admission fixture".to_string()),
                shader: AssetReference::from_locator(
                    ResourceLocator::parse(shader_uri).expect("fixture shader locator"),
                ),
                parent: None,
                options: Default::default(),
                queue: None,
                base_color: tint,
                base_color_texture: None,
                normal_texture: None,
                metallic: 0.0,
                roughness: 0.58,
                metallic_roughness_texture: None,
                occlusion_texture: None,
                emissive: [0.0; 3],
                emissive_texture: None,
                alpha_mode: AlphaMode::Opaque,
                double_sided: false,
                property_values: [
                    (
                        "lighting_model".to_string(),
                        toml::Value::String("custom:subsurface".to_string()),
                    ),
                    ("subsurface_profile".to_string(), toml::Value::Integer(7)),
                ]
                .into_iter()
                .collect(),
                texture_slots: Default::default(),
                validation_diagnostics: Vec::new(),
            },
        )
        .expect("register actual material with an explicit content identity");
    let revision = manager
        .resource_manager()
        .registry()
        .get(id)
        .expect("registered material revision")
        .revision;
    streamer
        .ensure_material(
            backend,
            &backend.device,
            &backend.queue,
            texture_layout,
            ResourceHandle::<MaterialMarker>::new(id),
        )
        .expect("prepare actual material candidate");
    let generation = streamer
        .staged_material_draw_generation(&id)
        .expect("preparation retains the staged generation");
    (id, generation, revision)
}

fn pending_for_admission_material(
    id: crate::core::resource::ResourceId,
    proxy: crate::graphics::scene::resources::PublishedMaterialDrawProxy<'_>,
) -> super::super::pending_mesh_draw::PendingMeshDraw {
    use super::super::pending_material_draw::PendingMaterialDraw;
    use super::super::pending_mesh_draw::{PendingMeshDraw, PendingMeshGeometry};
    use crate::asset::{MeshVertex, ModelPrimitiveAsset};
    use crate::core::framework::render::{CastShadowsMode, RenderMeshStaticState, RendererCommon};
    use crate::core::framework::scene::Mobility;
    use crate::core::math::{RenderMat4, Vec2, Vec3, Vec4};
    use crate::graphics::scene::resources::{
        default_pipeline_key, MaterialDisabledPasses, PublishedMaterialTextureBinding,
    };
    use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
        MaterialTextureBinding, MaterialTextureSet, MeshCommandSortInput,
    };
    use std::sync::Arc;
    let primitive = Arc::new(ModelPrimitiveAsset {
        vertices: vec![
            MeshVertex::new(Vec3::new(-0.5, -0.5, 0.0), Vec3::Z, Vec2::ZERO),
            MeshVertex::new(Vec3::new(0.5, -0.5, 0.0), Vec3::Z, Vec2::X),
            MeshVertex::new(Vec3::new(0.0, 0.5, 0.0), Vec3::Z, Vec2::Y),
        ],
        indices: vec![0, 1, 2],
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    });
    let binding = |value| match value {
        PublishedMaterialTextureBinding::Texture(resource) => {
            MaterialTextureBinding::texture(resource)
        }
        PublishedMaterialTextureBinding::OutputTarget(resource) => {
            MaterialTextureBinding::output_target(resource)
        }
    };
    let textures = proxy.textures();
    let runtime = proxy.runtime();
    PendingMeshDraw {
        local_bounds: primitive.render_mesh_descriptor().bounds,
        draw_index_count: u32::try_from(primitive.indices.len()).expect("triangle index extent"),
        mesh: PendingMeshGeometry::Dynamic(primitive),
        source_entity: 1,
        stable_instance_key: 1,
        source_draw_ordinal: 0,
        transform_revision: 1,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::default(),
        material: PendingMaterialDraw {
            resource_id: id,
            draw_generation: proxy.draw_generation(),
            textures: MaterialTextureSet::new(
                binding(textures.base_color),
                binding(textures.normal),
                binding(textures.metallic_roughness),
                binding(textures.occlusion),
                binding(textures.emissive),
                binding(textures.clearcoat_normal),
            ),
            uniform: proxy.uniform(),
            uniform_override_payload: None,
            standard_uniform: proxy.standard_uniform(),
            pipeline_key: runtime
                .map(|value| value.pipeline_key.clone())
                .unwrap_or_else(default_pipeline_key),
            common: Arc::new(RendererCommon::default()),
            renderer_cast_shadows: CastShadowsMode::Off,
            disabled_passes: runtime
                .map(|value| value.disabled_passes)
                .unwrap_or_else(MaterialDisabledPasses::default),
            taa_reactive_mask_strength: 0.0,
            half_resolution_transparency: false,
            draw_tint: Vec4::ONE,
        },
        morph_payload: None,
        source_morph_weights: None,
        morph_payload_slot: None,
        mesh_lod: None,
        model_matrix: RenderMat4::IDENTITY.to_cols_array_2d(),
        normal_transform_flags: 0,
        skinned: false,
        skinned_palette_signature: None,
        skinned_joint_palette: None,
        previous_skinned_joint_palette: None,
        skinned_gpu_source: None,
        resolved_skinned_gpu_source: None,
        previous_skinned_gpu_source: None,
        command_sort_input: MeshCommandSortInput::new(0.0, 1),
        first_index: 0,
        indirect_draw_ref: None,
    }
}

fn select_fixture_material_context(
    device: &wgpu::Device,
    streamer: &crate::graphics::scene::resources::ResourceStreamer,
    cache: &mut crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache,
    id: crate::core::resource::ResourceId,
    expected_published_count: usize,
) -> (
    super::MaterialDrawSelection,
    super::MaterialContextAdmissionStats,
) {
    use super::super::material_pipeline_requirements::{
        collect_material_pipeline_requirements, MaterialPipelineFeatureSet,
        PublishedMaterialPipelineRequirementCollector,
    };
    use crate::core::framework::render::ShaderQualityTier;
    let features = MaterialPipelineFeatureSet::direct(false);
    let quality = ShaderQualityTier::default();
    let pending = vec![pending_for_admission_material(
        id,
        streamer.published_material_draw_proxy(&id),
    )];
    let mut collector = PublishedMaterialPipelineRequirementCollector::default();
    for draw in &pending {
        collector.observe_published_draw(draw, features, quality, false);
    }
    let census = collector.finish(pending.len());
    assert_eq!(census.len(), expected_published_count);
    let staged_requirements =
        collect_material_pipeline_requirements(&pending, streamer, features, quality, false)
            .into_requirements()
            .map(|(_, _, requirements)| requirements.len())
            .sum::<usize>();
    assert!(
        staged_requirements > 0,
        "the real candidate has observed nonempty geometry requirements"
    );
    super::select_material_generations_for_context(
        device, streamer, cache, &pending, census, features, quality, false,
    )
    .expect("select a context using real material and fallback pipeline admission")
}

#[test]
fn cold_staged_material_is_view_local_and_rejects_a_replaced_generation() {
    use crate::core::math::Vec4;
    use std::sync::Arc;
    with_native_material_admission_fixture(|device, streamer, cache, manager, backend, layout| {
        let tint = [0.2, 0.7, 0.4, 1.0];
        let (id, generation, revision) = prepare_admission_fixture_material(
            manager,
            streamer,
            backend,
            layout,
            "cold-context-v1",
            tint,
            "builtin://shader/pbr.wgsl",
        );
        assert!(streamer
            .published_material_draw_proxy(&id)
            .runtime()
            .is_none());
        assert_eq!(
            streamer
                .published_material_draw_proxy(&id)
                .draw_generation(),
            None
        );
        let staged = streamer
            .staged_material_candidate(&id)
            .expect("actual staged runtime");
        assert_eq!(staged.base_color, Vec4::from_array(tint));
        assert_eq!(staged.subsurface_profile_index, 7);
        let staged_uniform = streamer.material_uniform(&id);
        let staged_standard = streamer.standard_material_uniform(&id);
        let unadmitted = super::MaterialDrawSelection::default();
        assert!(!unadmitted.has_overrides());
        assert!(unadmitted.proxy(streamer, &id).runtime().is_none());
        assert!(!Arc::ptr_eq(
            &unadmitted.proxy(streamer, &id).uniform(),
            &staged_uniform
        ));

        let (selection, stats) = select_fixture_material_context(device, streamer, cache, id, 0);
        assert_eq!(stats.cold_staged_candidate_count, 1);
        assert_eq!(stats.cold_staged_selected_count, 1);
        assert_eq!(stats.current_ready_count, 0);
        assert_eq!(stats.error_proxy_selected_count, 0);
        assert!(stats.ready_requirement_count > 0);
        assert!(
            selection.has_overrides(),
            "the actual build must recollect before GPU projection"
        );
        let proxy = selection.proxy(streamer, &id);
        assert_eq!(proxy.draw_generation(), Some(generation));
        let runtime = proxy.runtime().expect("admitted cold candidate runtime");
        assert_eq!(runtime.base_color, Vec4::from_array(tint));
        assert_eq!(runtime.subsurface_profile_index, 7);
        assert!(std::ptr::eq(
            runtime,
            streamer
                .staged_material_candidate(&id)
                .expect("same admitted staged runtime")
        ));
        assert_same_admission_texture_set(
            proxy.textures(),
            streamer
                .admitted_cold_staged_material_draw_proxy(&id, generation)
                .textures(),
        );
        assert!(Arc::ptr_eq(&proxy.uniform(), &staged_uniform));
        assert!(Arc::ptr_eq(&proxy.standard_uniform(), &staged_standard));
        assert!(streamer
            .published_material_draw_proxy(&id)
            .runtime()
            .is_none());
        assert_eq!(
            streamer
                .published_material_draw_proxy(&id)
                .draw_generation(),
            None
        );

        let (next_id, next_generation, next_revision) = prepare_admission_fixture_material(
            manager,
            streamer,
            backend,
            layout,
            "cold-context-v2",
            [0.8, 0.1, 0.3, 1.0],
            "builtin://shader/pbr.wgsl",
        );
        assert_eq!(next_id, id);
        assert!(next_revision > revision);
        assert_ne!(next_generation, generation);
        assert!(selection.proxy(streamer, &id).runtime().is_none());
        assert_eq!(selection.proxy(streamer, &id).draw_generation(), None);
        assert!(Arc::ptr_eq(
            &selection.proxy(streamer, &id).uniform(),
            &streamer.published_material_draw_proxy(&id).uniform(),
        ));
        assert!(Arc::ptr_eq(
            &selection.proxy(streamer, &id).standard_uniform(),
            &streamer
                .published_material_draw_proxy(&id)
                .standard_uniform(),
        ));
        assert!(streamer
            .admitted_cold_staged_material_draw_proxy(&id, generation)
            .runtime()
            .is_none());
        assert!(streamer
            .published_material_draw_proxy(&id)
            .runtime()
            .is_none());
    });
}

#[test]
fn hot_staged_material_keeps_the_complete_published_view_proxy() {
    use crate::core::math::Vec4;
    use std::sync::Arc;
    with_native_material_admission_fixture(|device, streamer, cache, manager, backend, layout| {
        let tint = [0.2, 0.7, 0.4, 1.0];
        let (id, generation, revision) = prepare_admission_fixture_material(
            manager,
            streamer,
            backend,
            layout,
            "hot-context-v1",
            tint,
            "builtin://shader/pbr.wgsl",
        );
        let (_, ready) = select_fixture_material_context(device, streamer, cache, id, 0);
        assert_eq!(ready.cold_staged_selected_count, 1);
        assert!(streamer.publish_staged_material_candidate(id));
        let published = streamer.published_material_draw_proxy(&id);
        let uniform = published.uniform();
        let standard = published.standard_uniform();
        assert_eq!(published.draw_generation(), Some(generation));

        let (_, next_generation, next_revision) = prepare_admission_fixture_material(
            manager,
            streamer,
            backend,
            layout,
            "hot-context-v2",
            [0.8, 0.1, 0.3, 1.0],
            "builtin://shader/pbr.wgsl",
        );
        assert!(next_revision > revision);
        assert_ne!(next_generation, generation);
        assert!(streamer
            .admitted_cold_staged_material_draw_proxy(&id, next_generation)
            .runtime()
            .is_none());
        let (selection, stats) = select_fixture_material_context(device, streamer, cache, id, 1);
        assert_eq!(stats.cold_staged_candidate_count, 0);
        assert_eq!(stats.cold_staged_selected_count, 0);
        let selected = selection.proxy(streamer, &id);
        assert_eq!(selected.draw_generation(), Some(generation));
        assert_eq!(
            selected
                .runtime()
                .expect("retained published runtime")
                .base_color,
            Vec4::from_array(tint)
        );
        assert!(Arc::ptr_eq(&selected.uniform(), &uniform));
        assert!(Arc::ptr_eq(&selected.standard_uniform(), &standard));
        assert_eq!(
            streamer
                .published_material_draw_proxy(&id)
                .draw_generation(),
            Some(generation)
        );
        assert_eq!(
            streamer.staged_material_draw_generation(&id),
            Some(next_generation)
        );
    });
}

fn assert_same_admission_texture_set(
    a: crate::graphics::scene::resources::PublishedMaterialTextureSet,
    b: crate::graphics::scene::resources::PublishedMaterialTextureSet,
) {
    use crate::graphics::scene::resources::PublishedMaterialTextureBinding;
    use std::sync::Arc;
    for (a, b) in [
        (a.base_color, b.base_color),
        (a.normal, b.normal),
        (a.metallic_roughness, b.metallic_roughness),
        (a.occlusion, b.occlusion),
        (a.emissive, b.emissive),
        (a.clearcoat_normal, b.clearcoat_normal),
    ] {
        match (a, b) {
            (
                PublishedMaterialTextureBinding::Texture(a),
                PublishedMaterialTextureBinding::Texture(b),
            ) => assert!(Arc::ptr_eq(&a, &b)),
            (
                PublishedMaterialTextureBinding::OutputTarget(a),
                PublishedMaterialTextureBinding::OutputTarget(b),
            ) => assert!(Arc::ptr_eq(&a, &b)),
            _ => panic!("a material selection must retain the complete texture binding kind"),
        }
    }
}

#[test]
fn invalid_cold_staged_shader_keeps_the_complete_error_proxy() {
    use std::sync::Arc;
    with_native_material_admission_fixture(|device, streamer, cache, manager, backend, layout| {
        let shader_id = register_invalid_admission_shader(manager);
        let (id, generation, _) = prepare_admission_fixture_material(
            manager,
            streamer,
            backend,
            layout,
            "cold-context-invalid-v1",
            [0.2, 0.7, 0.4, 1.0],
            "res://shaders/material-admission-invalid.wgsl",
        );
        assert_eq!(
            streamer
                .staged_material_candidate(&id)
                .expect("invalid shader candidate remains staged before validation")
                .pipeline_key
                .shader_id,
            shader_id,
        );
        let (selection, stats) = select_fixture_material_context(device, streamer, cache, id, 0);
        assert_eq!(stats.cold_staged_candidate_count, 1);
        assert_eq!(stats.cold_staged_selected_count, 0);
        assert_eq!(stats.current_ready_count, 0);
        assert_eq!(stats.error_proxy_selected_count, 1);
        assert!(stats.failed_count + stats.deferred_count > 0);
        assert!(
            stats.ready_requirement_count > 0,
            "the independent mandatory fallback is ready"
        );
        let selected = selection.proxy(streamer, &id);
        let fallback = streamer.published_material_draw_proxy(&id);
        assert_eq!(selected.draw_generation(), None);
        assert!(selected.runtime().is_none());
        assert!(Arc::ptr_eq(&selected.uniform(), &fallback.uniform()));
        assert!(Arc::ptr_eq(
            &selected.standard_uniform(),
            &fallback.standard_uniform()
        ));
        assert_same_admission_texture_set(selected.textures(), fallback.textures());
        assert_eq!(
            streamer.staged_material_draw_generation(&id),
            Some(generation)
        );
        assert!(streamer
            .published_material_draw_proxy(&id)
            .runtime()
            .is_none());
    });
}
