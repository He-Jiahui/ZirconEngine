use crate::asset::{ProjectAssetManager, ShaderAsset, ShaderSourceLanguage};
use crate::core::framework::render::{
    builtin_geometry_source_descriptor, GBufferChannelMask, ShaderAssetKind, ShaderPassType,
    ShadingModelDescriptor, ShadingModelId, GEOMETRY_SOURCE_ID_STATIC_MESH,
    SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceRecord};
use crate::graphics::shader::{
    assemble_deferred_gbuffer_shader_template, assemble_material_shader_template,
    DeferredGBufferShaderTemplateRequest, MaterialShaderTemplateRequest,
};

use super::*;

#[test]
fn include_resolution_stops_after_the_duplicate_witness() {
    let source = include_str!("../include_sources.rs");
    let start = source
        .find("fn resolve_include_source(")
        .expect("include resolver");
    let end = start
        + source[start..]
            .find("fn record_matches_include_token(")
            .expect("next include helper");
    let resolver = &source[start..end];

    assert!(!resolver.contains(concat!("collect::<", "Vec<_>>()")));
    assert_eq!(resolver.matches("matches.next()").count(), 2);
}

#[test]
fn project_shader_records_export_plugin_shading_model_include_sources() {
    let asset_manager = ProjectAssetManager::default();
    register_shader(
        &asset_manager,
        "package://toon/shaders/zr_shading_toon.wgsl",
        "fn shade_forward() {}\n",
    );
    register_shader(
        &asset_manager,
        "package://toon/shaders/zr_gbuffer_encode_toon.wgsl",
        "fn encode_gbuffer() {}\n",
    );
    register_shader(
        &asset_manager,
        "package://toon/shaders/zr_shade_deferred_toon.wgsl",
        "fn shade_deferred_toon() {}\n",
    );

    let set = ShadingModelIncludeSourceSet::from_project_asset_manager(
        &asset_manager,
        &[toon_shading_model_descriptor()],
    )
    .expect("plugin include sources should resolve from ready shader assets");

    assert_eq!(set.forward()[0].token, "zr_shading_toon.wgsl");
    assert!(set.forward()[0].source.contains("fn shade_forward"));
    assert_eq!(set.gbuffer()[0].token, "zr_gbuffer_encode_toon.wgsl");
    assert!(set.gbuffer()[0].source.contains("fn encode_gbuffer"));
    assert_eq!(set.deferred()[0].token, "zr_shade_deferred_toon.wgsl");
    assert!(set.deferred()[0].source.contains("fn shade_deferred_toon"));
}

#[test]
fn exported_include_source_set_feeds_forward_and_gbuffer_template_requests() {
    let asset_manager = ProjectAssetManager::default();
    register_shader(
        &asset_manager,
        "package://toon/shaders/zr_shading_toon.wgsl",
        "fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32> { return surface.base_color.rgb + vec3<f32>(ctx.frag_coord.x * 0.0); }\n",
    );
    register_shader(
        &asset_manager,
        "package://toon/shaders/zr_gbuffer_encode_toon.wgsl",
        "fn encode_gbuffer(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> ZrDeferredGBufferOutput { return ZrDeferredGBufferOutput(surface.base_color, vec4<f32>(0.0), vec4<f32>(ctx.shadow_params.z), vec4<f32>(surface.emissive, 1.0)); }\n",
    );
    register_shader(
        &asset_manager,
        "package://toon/shaders/zr_shade_deferred_toon.wgsl",
        "fn shade_deferred_toon() {}\n",
    );
    let descriptor = toon_shading_model_descriptor();
    let source_set = ShadingModelIncludeSourceSet::from_project_asset_manager(
        &asset_manager,
        &[descriptor.clone()],
    )
    .expect("plugin include source set");
    let geometry_source = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
        .expect("static geometry descriptor");

    let forward = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            geometry_source.clone(),
            ShaderPassType::Forward,
            material_surface_source(),
            "user_surface",
        )
        .with_shading_model_descriptor(descriptor.clone())
        .with_shading_model_forward_include_sources(&source_set),
    )
    .expect("forward template should consume exported include source set");
    let gbuffer = assemble_deferred_gbuffer_shader_template(
        DeferredGBufferShaderTemplateRequest::new(
            geometry_source,
            material_surface_source(),
            "user_surface",
        )
        .with_shading_model_descriptor(descriptor)
        .with_shading_model_gbuffer_include_sources(&source_set),
    )
    .expect("deferred GBuffer template should consume exported include source set");

    assert!(forward
        .wgsl_source
        .contains("// include: zr_shading_toon.wgsl"));
    assert!(gbuffer
        .wgsl_source
        .contains("// include: zr_gbuffer_encode_toon.wgsl"));
}

#[test]
fn project_shader_records_match_include_tokens_without_wgsl_extension() {
    let asset_manager = ProjectAssetManager::default();
    register_shader(
        &asset_manager,
        "res://shaders/zr_shading_toon.wgsl",
        "fn shade_forward() {}\n",
    );
    register_shader(
        &asset_manager,
        "res://shaders/zr_gbuffer_encode_toon.wgsl",
        "fn encode_gbuffer() {}\n",
    );
    register_shader(
        &asset_manager,
        "res://shaders/zr_shade_deferred_toon.wgsl",
        "fn shade_deferred_toon() {}\n",
    );

    let descriptor = ShadingModelDescriptor::new(
        ShadingModelId::new(16),
        "toon",
        "zr_shading_toon",
        "zr_gbuffer_encode_toon",
        "zr_shade_deferred_toon",
        GBufferChannelMask::standard_lit(),
    );

    let set =
        ShadingModelIncludeSourceSet::from_project_asset_manager(&asset_manager, &[descriptor])
            .expect("extension-free include tokens should resolve");

    assert_eq!(set.forward()[0].token, "zr_shading_toon");
    assert_eq!(set.gbuffer()[0].token, "zr_gbuffer_encode_toon");
    assert_eq!(set.deferred()[0].token, "zr_shade_deferred_toon");
}

#[test]
fn project_shader_records_skip_builtin_shading_model_descriptors() {
    let asset_manager = ProjectAssetManager::default();
    let set = ShadingModelIncludeSourceSet::from_project_asset_manager(
        &asset_manager,
        &[ShadingModelDescriptor::new(
            SHADING_MODEL_ID_STANDARD_PBR,
            "pbr",
            "zr_shading_standard_pbr.wgsl",
            "zr_gbuffer_encode_standard_pbr.wgsl",
            "zr_shade_deferred_standard_pbr.wgsl",
            GBufferChannelMask::standard_lit(),
        )],
    )
    .expect("builtin descriptor should not require project WGSL assets");

    assert!(set.forward().is_empty());
    assert!(set.gbuffer().is_empty());
    assert!(set.deferred().is_empty());
}

#[test]
fn plugin_descriptor_may_reuse_runtime_owned_subsurface_includes() {
    let asset_manager = ProjectAssetManager::default();
    let set = ShadingModelIncludeSourceSet::from_project_asset_manager(
        &asset_manager,
        &[ShadingModelDescriptor::new(
            ShadingModelId::new(16),
            "custom:subsurface",
            "zr_shading_standard_pbr.wgsl",
            "zr_gbuffer_encode_subsurface.wgsl",
            "zr_shade_deferred_subsurface.wgsl",
            GBufferChannelMask::standard_lit(),
        )],
    )
    .expect("runtime-owned plugin includes should not require project shader assets");

    assert!(set.forward().is_empty());
    assert!(set.gbuffer().is_empty());
    assert!(set.deferred().is_empty());
}

#[test]
fn project_shader_records_report_missing_plugin_include_source() {
    let asset_manager = ProjectAssetManager::default();

    let error = ShadingModelIncludeSourceSet::from_project_asset_manager(
        &asset_manager,
        &[toon_shading_model_descriptor()],
    )
    .expect_err("missing plugin include should be explicit");

    assert_eq!(
        error,
        ShadingModelIncludeSourceError::MissingInclude {
            token: "zr_shading_toon.wgsl".to_string()
        }
    );
}

#[test]
fn project_shader_records_reject_duplicate_include_token_matches() {
    let asset_manager = ProjectAssetManager::default();
    register_shader(
        &asset_manager,
        "res://toon/a/zr_shading_toon.wgsl",
        "fn shade_forward_a() {}\n",
    );
    register_shader(
        &asset_manager,
        "res://toon/b/zr_shading_toon.wgsl",
        "fn shade_forward_b() {}\n",
    );
    register_shader(
        &asset_manager,
        "res://toon/zr_gbuffer_encode_toon.wgsl",
        "fn encode_gbuffer() {}\n",
    );
    register_shader(
        &asset_manager,
        "res://toon/zr_shade_deferred_toon.wgsl",
        "fn shade_deferred_toon() {}\n",
    );

    let error = ShadingModelIncludeSourceSet::from_project_asset_manager(
        &asset_manager,
        &[toon_shading_model_descriptor()],
    )
    .expect_err("duplicate include token should be rejected");

    assert!(matches!(
        error,
        ShadingModelIncludeSourceError::DuplicateIncludeToken { .. }
    ));
}

fn toon_shading_model_descriptor() -> ShadingModelDescriptor {
    ShadingModelDescriptor::new(
        ShadingModelId::new(16),
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

fn material_surface_source() -> &'static str {
    "fn user_surface(input: ZrVertexOutput) -> ZrSurfaceOutput { return zr_surface_from_base_color(input.color); }\n"
}
