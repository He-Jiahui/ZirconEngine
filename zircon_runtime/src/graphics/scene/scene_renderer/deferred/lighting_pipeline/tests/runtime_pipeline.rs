use std::sync::Arc;
use std::time::Duration;

use crate::asset::{ProjectAssetManager, ShaderAsset, ShaderSourceLanguage};
use crate::core::framework::render::ShaderAssetKind;
use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceRecord};
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::scene_renderer::environment::scene_bind_group_layout_entries;
use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;

use super::super::super::lighting_bind_group_layout::create_lighting_bind_group_layout;
use super::super::DeferredLightingPipelineCache;
use super::{toon_shading_model_descriptor, CUSTOM_TOON_DEFERRED_INCLUDE};

const CUSTOM_TOON_FORWARD_INCLUDE: &str = r#"
fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32> {
    return surface.base_color.rgb + vec3<f32>(ctx.frag_coord.x * 0.0);
}
"#;

const CUSTOM_TOON_GBUFFER_INCLUDE: &str = r#"
fn encode_gbuffer(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> ZrDeferredGBufferOutput {
    let receive_shadows = ctx.shadow_params.z > 0.5;
    return ZrDeferredGBufferOutput(
        surface.base_color,
        vec4<f32>(surface.normal_ws * 0.5 + vec3<f32>(0.5), surface.base_color.a),
        vec4<f32>(
            0.25,
            0.75,
            clamp(surface.occlusion, 0.0, 1.0),
            zr_deferred_encode_material_flags(16u, receive_shadows),
        ),
        vec4<f32>(surface.emissive, 1.0),
    );
}
"#;

#[test]
fn custom_shading_model_deferred_lighting_pipelines_create_with_project_include_source() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let asset_manager = ProjectAssetManager::default();
    let descriptor = toon_shading_model_descriptor();
    register_shader(
        &asset_manager,
        "package://toon/shaders/zr_shading_toon.wgsl",
        CUSTOM_TOON_FORWARD_INCLUDE,
    );
    register_shader(
        &asset_manager,
        "package://toon/shaders/zr_gbuffer_encode_toon.wgsl",
        CUSTOM_TOON_GBUFFER_INCLUDE,
    );
    register_shader(
        &asset_manager,
        "package://toon/shaders/zr_shade_deferred_toon.wgsl",
        CUSTOM_TOON_DEFERRED_INCLUDE,
    );

    let scene_layout = scene_bind_group_layout(&backend.device);
    let lighting_layout = create_lighting_bind_group_layout(
        &backend.device,
        SceneRendererDeferredLightingProfile::FullScene,
    );
    let gpu_scene_palette_buffer =
        Arc::new(backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-custom-deferred-lighting-skinned-palette"),
            size: 64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        }));
    let gpu_scene = GpuScene::new(
        &backend.device,
        gpu_scene_palette_buffer,
        wgpu::BufferSize::new(64).unwrap(),
    );

    let error_scope = backend
        .device
        .push_error_scope(wgpu::ErrorFilter::Validation);
    let (pipelines, _) = DeferredLightingPipelineCache::new(
        &backend.device,
        &asset_manager,
        &scene_layout,
        &lighting_layout,
        gpu_scene.scene_bind_group_layout(),
        wgpu::TextureFormat::Rgba8Unorm,
        &[descriptor],
        false,
        SceneRendererDeferredLightingProfile::FullScene,
    )
    .expect("custom deferred lighting pipeline should be created from project WGSL include source");
    let _standard_pipeline = pipelines.pipeline(&backend.device, &lighting_layout, false);
    let _subsurface_pipeline = pipelines.pipeline(&backend.device, &lighting_layout, true);
    let error = pollster::block_on(error_scope.pop());
    assert!(
        error.is_none(),
        "custom deferred lighting pipelines should pass WGPU validation: {error:?}"
    );
}

#[test]
fn environment_only_pbr_pipeline_defers_startup_pso_and_creates_on_demand_without_direct_light_layout_entries(
) {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let asset_manager = ProjectAssetManager::default();
    let scene_layout = scene_bind_group_layout(&backend.device);
    let lighting_layout = create_lighting_bind_group_layout(
        &backend.device,
        SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
    );
    let gpu_scene_palette_buffer =
        Arc::new(backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-environment-only-pbr-skinned-palette"),
            size: 64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        }));
    let gpu_scene = GpuScene::new(
        &backend.device,
        gpu_scene_palette_buffer,
        wgpu::BufferSize::new(64).unwrap(),
    );

    let error_scope = backend
        .device
        .push_error_scope(wgpu::ErrorFilter::Validation);
    let (pipelines, startup) = DeferredLightingPipelineCache::new(
        &backend.device,
        &asset_manager,
        &scene_layout,
        &lighting_layout,
        gpu_scene.scene_bind_group_layout(),
        wgpu::TextureFormat::Rgba8Unorm,
        &[],
        false,
        SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
    )
    .expect("environment-only PBR pipeline should assemble without direct-light sources");
    assert_eq!(startup.pipeline_foundation(), Duration::ZERO);
    assert_eq!(startup.standard_pipeline(), Duration::ZERO);
    let _pipeline = pipelines.pipeline(&backend.device, &lighting_layout, false);
    let error = pollster::block_on(error_scope.pop());
    assert!(
        error.is_none(),
        "environment-only PBR layout and pipeline should pass WGPU validation: {error:?}"
    );
}

fn register_shader(asset_manager: &ProjectAssetManager, locator_text: &str, source: &str) {
    let locator = ResourceLocator::parse(locator_text).expect("valid shader locator");
    let id = ResourceId::from_locator(&locator);
    let record = ResourceRecord::new(id, ResourceKind::Shader, locator.clone())
        .with_source_hash(format!("{locator_text}-hash"));
    asset_manager
        .resource_manager()
        .register_ready(
            record,
            ShaderAsset {
                uri: locator,
                kind: ShaderAssetKind::Include,
                source_language: ShaderSourceLanguage::Wgsl,
                source: source.to_string(),
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
        .expect("register shader resource");
}

fn scene_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-test-custom-deferred-lighting-scene-layout"),
        entries: &scene_bind_group_layout_entries(),
    })
}
