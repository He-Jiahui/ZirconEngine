use std::fs;
use std::sync::Arc;

use crate::asset::{ProjectAssetManager, ShaderAsset, ShaderSourceLanguage};
use crate::core::framework::render::{
    CapturedFrame, GBufferChannelMask, RenderMaterialLightingModel, ShaderAssetKind,
    ShadingModelDescriptor, ShadingModelId, SHADING_MODEL_PLUGIN_ID_START,
};
use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceRecord};
use crate::dynamic_api::prewarm_shader_variants_with_wgpu_pipeline_validation;

use super::assertions::{
    assert_registry_material_pass_first_frame_shader_cache_hit_for_shading_model,
    assert_registry_material_pass_prewarm_dimensions_written_for_shading_model,
    assert_registry_material_pass_prewarm_written_for_shading_model,
    assert_registry_material_pass_velocity_frame_shader_cache_hit_for_shading_model,
};
use super::case::registry_shader_cases;
use super::fixture::{
    submit_registry_material_passes_with_plugin_shading_model,
    submit_registry_material_passes_with_plugin_shading_model_capture,
    RegistryMaterialPassPluginShadingModel,
};
use super::manifest::{
    registry_material_pass_product_prewarm_manifest_with_plugin_shading_models,
    registry_material_pass_runtime_surface_source,
};
use super::shader_cache_test_roots;

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
    );
}
"#;

const CUSTOM_TOON_DEFERRED_INCLUDE: &str = r#"
fn shade_deferred_toon(position: vec4<f32>, coord: vec2<i32>, albedo: vec4<f32>, material: vec4<f32>, normal: vec3<f32>) -> vec4<f32> {
    let band = clamp(normal.z * 0.5 + 0.5 + material.r * 0.0 + f32(coord.x) * 0.0 + position.x * 0.0, 0.0, 1.0);
    return vec4<f32>(
        albedo.r * 0.02,
        max(0.65, albedo.g * band),
        albedo.b * 0.02,
        albedo.a,
    );
}
"#;

#[test]
fn render_product_custom_shading_model_registry_material_passes_use_staged_prewarm_without_compile_miss(
) {
    let cache_roots = shader_cache_test_roots(
        "zircon_product_custom_shading_model_registry_material_passes_staged_prewarm",
    );
    let _ = fs::remove_dir_all(&cache_roots.root);
    fs::create_dir_all(&cache_roots.root).expect("shader cache test root");

    let case = registry_shader_cases()[0];
    let descriptor = custom_toon_shading_model_descriptor();
    let prewarm_asset_manager = Arc::new(ProjectAssetManager::default());
    register_custom_toon_shading_model_includes(&prewarm_asset_manager);
    let manifest = registry_material_pass_product_prewarm_manifest_with_plugin_shading_models(
        &prewarm_asset_manager,
        &[case],
        &[descriptor.clone()],
    )
    .expect("custom shading-model registry material-pass prewarm manifest");
    let registry_shader_source = registry_material_pass_runtime_surface_source();

    let prewarm_report =
        prewarm_shader_variants_with_wgpu_pipeline_validation(&manifest, &cache_roots.staged_root);
    assert_eq!(prewarm_report.requested_count, manifest.variants.len());
    assert_eq!(
        prewarm_report.written_count,
        manifest.variants.len(),
        "custom toon prewarm should write every requested pass variant; report={prewarm_report:#?}"
    );
    assert_eq!(prewarm_report.failed_count, 0);
    assert!(prewarm_report.failures.is_empty());
    assert!(prewarm_report.wgpu_pipeline_validation.enabled);
    assert_eq!(
        prewarm_report.wgpu_pipeline_validation.validated_count,
        manifest.variants.len()
    );
    assert_registry_material_pass_prewarm_dimensions_written_for_shading_model(
        &prewarm_report,
        descriptor.id,
        "prewarm custom toon shading model",
    );
    assert_registry_material_pass_prewarm_written_for_shading_model(
        &manifest,
        &prewarm_report,
        case,
        descriptor.id,
    );

    let launch = submit_registry_material_passes_with_plugin_shading_model(
        case,
        registry_shader_source.as_str(),
        7_201,
        &cache_roots.runtime_root,
        &cache_roots.staged_root,
        custom_toon_plugin_shading_model(),
    );
    assert_registry_material_pass_first_frame_shader_cache_hit_for_shading_model(
        &launch.first_frame,
        case,
        &prewarm_report,
        descriptor.id,
        "custom toon shading model",
    );
    assert_registry_material_pass_velocity_frame_shader_cache_hit_for_shading_model(
        &launch.velocity_frame,
        case,
        &prewarm_report,
        descriptor.id,
        "custom toon shading model",
    );

    let _ = fs::remove_dir_all(&cache_roots.root);
}

#[test]
fn render_product_custom_shading_model_deferred_lighting_readback_uses_project_include() {
    let cache_roots =
        shader_cache_test_roots("zircon_product_custom_shading_model_deferred_lighting_readback");
    let _ = fs::remove_dir_all(&cache_roots.root);
    fs::create_dir_all(&cache_roots.root).expect("shader cache test root");

    let case = registry_shader_cases()[0];
    let descriptor = custom_toon_shading_model_descriptor();
    let prewarm_asset_manager = Arc::new(ProjectAssetManager::default());
    register_custom_toon_shading_model_includes(&prewarm_asset_manager);
    let manifest = registry_material_pass_product_prewarm_manifest_with_plugin_shading_models(
        &prewarm_asset_manager,
        &[case],
        &[descriptor.clone()],
    )
    .expect("custom shading-model deferred lighting readback prewarm manifest");
    let registry_shader_source = registry_material_pass_runtime_surface_source();

    let prewarm_report =
        prewarm_shader_variants_with_wgpu_pipeline_validation(&manifest, &cache_roots.staged_root);
    assert_eq!(prewarm_report.requested_count, manifest.variants.len());
    assert_eq!(
        prewarm_report.written_count,
        manifest.variants.len(),
        "custom toon readback prewarm should write every requested pass variant; report={prewarm_report:#?}"
    );
    assert_eq!(prewarm_report.failed_count, 0);
    assert!(prewarm_report.failures.is_empty());
    assert!(prewarm_report.wgpu_pipeline_validation.enabled);
    assert_eq!(
        prewarm_report.wgpu_pipeline_validation.validated_count,
        manifest.variants.len()
    );
    assert_registry_material_pass_prewarm_dimensions_written_for_shading_model(
        &prewarm_report,
        descriptor.id,
        "prewarm custom toon shading model deferred lighting readback",
    );
    assert_registry_material_pass_prewarm_written_for_shading_model(
        &manifest,
        &prewarm_report,
        case,
        descriptor.id,
    );

    let launch = submit_registry_material_passes_with_plugin_shading_model_capture(
        case,
        registry_shader_source.as_str(),
        7_301,
        &cache_roots.runtime_root,
        &cache_roots.staged_root,
        custom_toon_plugin_shading_model(),
    );
    assert_registry_material_pass_first_frame_shader_cache_hit_for_shading_model(
        &launch.first_frame,
        case,
        &prewarm_report,
        descriptor.id,
        "custom toon shading model deferred lighting readback",
    );
    assert_registry_material_pass_velocity_frame_shader_cache_hit_for_shading_model(
        &launch.velocity_frame,
        case,
        &prewarm_report,
        descriptor.id,
        "custom toon shading model deferred lighting readback",
    );
    assert_custom_toon_deferred_lighting_readback(
        launch
            .first_capture
            .as_ref()
            .expect("custom toon first-frame capture"),
    );
    assert_custom_toon_deferred_lighting_readback(
        launch
            .velocity_capture
            .as_ref()
            .expect("custom toon velocity-frame capture"),
    );

    let _ = fs::remove_dir_all(&cache_roots.root);
}

pub(super) fn custom_toon_plugin_shading_model() -> RegistryMaterialPassPluginShadingModel {
    RegistryMaterialPassPluginShadingModel {
        descriptor: custom_toon_shading_model_descriptor(),
        material_lighting_model: RenderMaterialLightingModel::Custom {
            name: "toon".to_string(),
        },
        register_shader_includes: register_custom_toon_shading_model_includes,
    }
}

fn custom_toon_shading_model_descriptor() -> ShadingModelDescriptor {
    ShadingModelDescriptor::new(
        custom_toon_shading_model_id(),
        "custom:toon",
        "zr_shading_toon.wgsl",
        "zr_gbuffer_encode_toon.wgsl",
        "zr_shade_deferred_toon.wgsl",
        GBufferChannelMask::standard_lit(),
    )
}

fn custom_toon_shading_model_id() -> ShadingModelId {
    ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START)
}

fn register_custom_toon_shading_model_includes(asset_manager: &ProjectAssetManager) {
    register_shader(
        asset_manager,
        "package://toon/shaders/zr_shading_toon.wgsl",
        CUSTOM_TOON_FORWARD_INCLUDE,
    );
    register_shader(
        asset_manager,
        "package://toon/shaders/zr_gbuffer_encode_toon.wgsl",
        CUSTOM_TOON_GBUFFER_INCLUDE,
    );
    register_shader(
        asset_manager,
        "package://toon/shaders/zr_shade_deferred_toon.wgsl",
        CUSTOM_TOON_DEFERRED_INCLUDE,
    );
}

pub(super) fn assert_custom_toon_deferred_lighting_readback(frame: &CapturedFrame) {
    let dominant_green_pixels = frame
        .rgba
        .chunks_exact(4)
        .filter(|pixel| {
            let red = i16::from(pixel[0]);
            let green = i16::from(pixel[1]);
            let blue = i16::from(pixel[2]);
            green > 48 && green > red + 24 && green > blue + 24
        })
        .count();
    assert!(
        dominant_green_pixels >= 32,
        "custom toon deferred lighting include should tint the product frame green; dominant_green_pixels={dominant_green_pixels}; {}",
        frame_rgb_summary(frame)
    );
}

fn frame_rgb_summary(frame: &CapturedFrame) -> String {
    let mut non_black_pixels = 0usize;
    let mut dominant_red_pixels = 0usize;
    let mut dominant_green_pixels = 0usize;
    let mut dominant_blue_pixels = 0usize;
    let mut max_pixel = [0u8; 4];
    for pixel in frame.rgba.chunks_exact(4) {
        let red = i16::from(pixel[0]);
        let green = i16::from(pixel[1]);
        let blue = i16::from(pixel[2]);
        if red > 4 || green > 4 || blue > 4 {
            non_black_pixels += 1;
        }
        if red > green + 24 && red > blue + 24 {
            dominant_red_pixels += 1;
        }
        if green > red + 24 && green > blue + 24 {
            dominant_green_pixels += 1;
        }
        if blue > red + 24 && blue > green + 24 {
            dominant_blue_pixels += 1;
        }
        if u16::from(pixel[0]) + u16::from(pixel[1]) + u16::from(pixel[2])
            > u16::from(max_pixel[0]) + u16::from(max_pixel[1]) + u16::from(max_pixel[2])
        {
            max_pixel.copy_from_slice(pixel);
        }
    }
    let center_pixel = frame_center_pixel(frame);
    format!(
        "frame={}x{} non_black_pixels={non_black_pixels} dominant_rgb=({dominant_red_pixels},{dominant_green_pixels},{dominant_blue_pixels}) max_pixel={max_pixel:?} center_pixel={center_pixel:?}",
        frame.width, frame.height
    )
}

fn frame_center_pixel(frame: &CapturedFrame) -> Option<[u8; 4]> {
    let x = usize::try_from(frame.width / 2).ok()?;
    let y = usize::try_from(frame.height / 2).ok()?;
    let width = usize::try_from(frame.width).ok()?;
    let offset = ((y * width) + x) * 4;
    frame
        .rgba
        .get(offset..offset + 4)
        .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
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
