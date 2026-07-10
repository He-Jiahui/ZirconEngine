use std::borrow::Cow;
use std::sync::Arc;

use crate::asset::{
    AssetReference, ProjectAssetManager, ShaderAsset, ShaderEntryPointAsset, ShaderSourceLanguage,
};
use crate::core::framework::render::{
    builtin_geometry_source_descriptor, GBufferChannelMask, ShaderAssetKind,
    ShadingModelDescriptor, ShadingModelId, GEOMETRY_SOURCE_ID_STATIC_MESH,
    SHADING_MODEL_PLUGIN_ID_START,
};
use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceRecord};
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::resources::{default_pipeline_key, ResourceStreamer};

use super::super::{
    mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor_with_streamer,
    mesh_pipeline_shader_source_for_geometry_descriptor, MESH_SHADER_TEMPLATE_REVISION,
};

const CUSTOM_TOON_FORWARD_INCLUDE: &str = r#"
const ZR_SHADING_TOON_DEBUG_ID: u32 = 16u;

fn zr_toon_band(normal_ws: vec3<f32>) -> f32 {
    return select(0.35, 0.85, normal_ws.z >= 0.0);
}

fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32> {
    let toon_band = zr_toon_band(surface.normal_ws);
    return surface.base_color.rgb * toon_band + surface.emissive + vec3<f32>(ctx.frag_coord.x * 0.0);
}
"#;

const CUSTOM_TOON_GBUFFER_INCLUDE: &str = r#"
const ZR_GBUFFER_TOON_DEBUG_ID: u32 = 16u;

fn encode_gbuffer(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> ZrDeferredGBufferOutput {
    let receive_shadows = ctx.shadow_params.z > 0.5;
    return ZrDeferredGBufferOutput(
        surface.base_color,
        vec4<f32>(surface.normal_ws * 0.5 + vec3<f32>(0.5), surface.base_color.a),
        vec4<f32>(
            0.25,
            0.75,
            clamp(surface.occlusion, 0.0, 1.0),
            zr_deferred_encode_material_flags(ZR_GBUFFER_TOON_DEBUG_ID, receive_shadows),
        ),
        vec4<f32>(surface.emissive, 1.0),
    );
}
"#;

const CUSTOM_TOON_DEFERRED_INCLUDE: &str = r#"
fn shade_deferred_toon(position: vec4<f32>, coord: vec2<i32>, albedo: vec4<f32>, material: vec4<f32>, normal: vec3<f32>) -> vec4<f32> {
    let band = clamp(normal.z * 0.5 + 0.5 + material.r * 0.0 + f32(coord.x) * 0.0 + position.x * 0.0, 0.0, 1.0);
    return vec4<f32>(albedo.rgb * band, albedo.a);
}
"#;

const FULL_PASS_WGSL: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(position, 1.0);
    out.color = vec4<f32>(0.0, 1.0, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

#[test]
fn runtime_custom_shading_model_sources_compile_as_wgpu_modules() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let texture_layout = texture_bind_group_layout(&backend.device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
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
    let streamer = ResourceStreamer::new_for_test_with_plugin_shading_models(
        Arc::clone(&asset_manager),
        &backend.device,
        &backend.queue,
        &texture_layout,
        [descriptor.clone()],
    )
    .expect("resource streamer with plugin shading model");
    let geometry_source = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
        .expect("static geometry source descriptor");
    let mut key = default_pipeline_key();
    key.shading_model_id = descriptor.id;

    let forward_source =
        mesh_pipeline_shader_source_for_geometry_descriptor(&streamer, &key, &geometry_source)
            .expect("forward runtime custom shading model source");
    assert!(forward_source
        .wgsl_source
        .contains("ZR_SHADING_TOON_DEBUG_ID"));
    validate_wgpu_shader_module(
        &backend.device,
        "zircon-test-runtime-custom-shading-forward",
        &forward_source.wgsl_source,
    );

    let gbuffer_source =
        mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor_with_streamer(
            &streamer,
            &key,
            &geometry_source,
        )
        .expect("gbuffer runtime custom shading model source");
    assert!(gbuffer_source
        .wgsl_source
        .contains("ZR_GBUFFER_TOON_DEBUG_ID"));
    validate_wgpu_shader_module(
        &backend.device,
        "zircon-test-runtime-custom-shading-gbuffer",
        &gbuffer_source.wgsl_source,
    );
}

#[test]
fn builtin_fallback_shader_loaded_as_surface_still_uses_standard_material_template() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let texture_layout = texture_bind_group_layout(&backend.device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::clone(&asset_manager),
        &backend.device,
        &backend.queue,
        &texture_layout,
    );
    let fallback_shader =
        ResourceLocator::parse("builtin://shader/pbr.wgsl").expect("builtin fallback shader");
    let (shader_id, _, _) = streamer
        .ensure_shader_source(&AssetReference::from_locator(fallback_shader))
        .expect("builtin fallback shader loads");
    let key = default_pipeline_key();
    assert_eq!(shader_id, key.shader_id);
    assert!(key.uses_fallback_shader());
    assert!(streamer.shader_is_surface(&key.shader_id));
    let geometry_source = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
        .expect("static geometry source descriptor");

    let forward_source =
        mesh_pipeline_shader_source_for_geometry_descriptor(&streamer, &key, &geometry_source)
            .expect("fallback shader should assemble as standard forward template");
    assert!(forward_source
        .wgsl_source
        .contains("fn zr_material_surface("));
    assert!(forward_source
        .wgsl_source
        .contains("standard_material_properties.data8.z"));
    assert!(!forward_source
        .wgsl_source
        .contains("struct MaterialPropertyUniform"));
    validate_wgpu_shader_module(
        &backend.device,
        "zircon-test-fallback-forward-standard-template",
        &forward_source.wgsl_source,
    );

    let gbuffer_source =
        mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor_with_streamer(
            &streamer,
            &key,
            &geometry_source,
        )
        .expect("fallback shader should assemble as standard GBuffer template");
    assert!(gbuffer_source
        .wgsl_source
        .contains("// include: zr_gbuffer_encode_standard_pbr.wgsl"));
    assert!(gbuffer_source
        .wgsl_source
        .contains("standard_material_properties.data8.z"));
    assert!(!gbuffer_source
        .wgsl_source
        .contains("struct MaterialPropertyUniform"));
    validate_wgpu_shader_module(
        &backend.device,
        "zircon-test-fallback-gbuffer-standard-template",
        &gbuffer_source.wgsl_source,
    );
}

#[test]
fn runtime_surface_shader_with_full_pass_entry_points_uses_raw_wgsl_source() {
    let backend = RenderBackend::new_offscreen().expect("offscreen backend");
    let texture_layout = texture_bind_group_layout(&backend.device);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let shader_locator = register_surface_full_pass_shader(
        &asset_manager,
        "res://project/shaders/full-pass.wgsl",
        FULL_PASS_WGSL,
    );
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::clone(&asset_manager),
        &backend.device,
        &backend.queue,
        &texture_layout,
    );
    let (shader_id, shader_revision, _) = streamer
        .ensure_shader_source(&AssetReference::from_locator(shader_locator))
        .expect("project full-pass shader streams");
    let geometry_source = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
        .expect("static geometry source descriptor");
    let mut key = default_pipeline_key();
    key.shader_id = shader_id;
    key.shader_revision = shader_revision;

    assert!(streamer.shader_is_surface(&key.shader_id));
    assert!(!streamer.shader_uses_material_surface_source(&key.shader_id));

    let source =
        mesh_pipeline_shader_source_for_geometry_descriptor(&streamer, &key, &geometry_source)
            .expect("full-pass project shader should remain raw WGSL");
    assert_eq!(source.wgsl_source, FULL_PASS_WGSL);
    assert_eq!(source.template_revision, MESH_SHADER_TEMPLATE_REVISION);
    assert_eq!(source.cache_content_hashes.len(), 1);
    assert!(source.wgsl_source.contains("fn vs_main("));
    assert!(source.wgsl_source.contains("fn fs_main("));
    assert!(!source.wgsl_source.contains("fn zr_material_surface("));
    validate_wgpu_shader_module(
        &backend.device,
        "zircon-test-full-pass-project-raw-wgsl",
        &source.wgsl_source,
    );
}

fn toon_shading_model_descriptor() -> ShadingModelDescriptor {
    ShadingModelDescriptor::new(
        ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START),
        "toon",
        "zr_shading_toon.wgsl",
        "zr_gbuffer_encode_toon.wgsl",
        "zr_shade_deferred_toon.wgsl",
        GBufferChannelMask::standard_lit(),
    )
}

fn register_shader(asset_manager: &ProjectAssetManager, locator_text: &str, source: &str) {
    let locator = ResourceLocator::parse(locator_text).expect("valid shader locator");
    let id = ResourceId::from_locator(&locator);
    let record = ResourceRecord::new(id, ResourceKind::Shader, locator.clone())
        .with_source_hash(format!("{locator_text}-hash"));
    asset_manager.resource_manager().register_ready(
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
    );
}

fn register_surface_full_pass_shader(
    asset_manager: &ProjectAssetManager,
    locator_text: &str,
    source: &str,
) -> ResourceLocator {
    let locator = ResourceLocator::parse(locator_text).expect("valid shader locator");
    let id = ResourceId::from_locator(&locator);
    let record = ResourceRecord::new(id, ResourceKind::Shader, locator.clone())
        .with_source_hash(format!("{locator_text}-hash"));
    asset_manager.resource_manager().register_ready(
        record,
        ShaderAsset {
            uri: locator.clone(),
            kind: ShaderAssetKind::Surface,
            source_language: ShaderSourceLanguage::Wgsl,
            source: source.to_string(),
            wgsl_source: source.to_string(),
            import_path: None,
            entry_points: vec![
                ShaderEntryPointAsset {
                    name: "vs_main".to_string(),
                    stage: "vertex".to_string(),
                },
                ShaderEntryPointAsset {
                    name: "fs_main".to_string(),
                    stage: "fragment".to_string(),
                },
            ],
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
    );
    locator
}

fn texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-runtime-custom-shading-test-texture-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

fn validate_wgpu_shader_module(device: &wgpu::Device, label: &'static str, wgsl_source: &str) {
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let _shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(wgsl_source)),
    });
    let error = pollster::block_on(error_scope.pop());
    assert!(
        error.is_none(),
        "{label} should create a WGPU shader module: {error:?}"
    );
}
