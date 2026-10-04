use super::super::shader_sources::POST_PROCESS_SHADER;
use super::PostProcessDepthSamplingMode;

const DEPTH_OF_FIELD_PREPARE_SHADER: &str =
    include_str!("../../shaders/depth_of_field_prepare.wgsl");
const VELOCITY_CAMERA_SHADER: &str =
    include_str!("../../../temporal/velocity/shaders/velocity_camera.wgsl");
const TAA_RESOLVE_SHADER: &str = include_str!("../../../temporal/taa/shaders/taa_resolve.wgsl");

#[test]
fn gl_backends_use_viewport_depth_fallback() {
    assert_eq!(
        PostProcessDepthSamplingMode::for_backend_name("wgpu(gl)"),
        PostProcessDepthSamplingMode::ViewportDepthFallback
    );
    assert_eq!(
        PostProcessDepthSamplingMode::for_backend_name("wgpu(webgl)"),
        PostProcessDepthSamplingMode::ViewportDepthFallback
    );
    assert_eq!(
        PostProcessDepthSamplingMode::for_backend_name("wgpu(ANGLE)"),
        PostProcessDepthSamplingMode::ViewportDepthFallback
    );
}

#[test]
fn non_gl_backends_keep_raw_depth_texture_sampling() {
    assert_eq!(
        PostProcessDepthSamplingMode::for_backend_name("wgpu(vulkan)"),
        PostProcessDepthSamplingMode::RawDepthTexture
    );
    assert_eq!(
        PostProcessDepthSamplingMode::for_backend_name("wgpu(dx12)"),
        PostProcessDepthSamplingMode::RawDepthTexture
    );
    assert_eq!(
        PostProcessDepthSamplingMode::for_backend_name("wgpu(metal)"),
        PostProcessDepthSamplingMode::RawDepthTexture
    );
}

#[test]
fn raw_depth_shader_uses_derivative_free_integer_loads() {
    let shader_source = PostProcessDepthSamplingMode::RawDepthTexture
        .post_process_shader_source(POST_PROCESS_SHADER);

    naga::front::wgsl::parse_str(&shader_source).expect("raw-depth post-process shader must parse");
    assert!(shader_source
        .contains("return clamp(textureLoad(scene_depth_tex, physical_coord, 0), 0.0, 1.0);"));
    assert!(!shader_source.contains("textureSample(scene_depth_tex"));
}

#[test]
fn viewport_depth_fallback_shader_removes_raw_depth_texture_sampling() {
    let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
        .post_process_shader_source(POST_PROCESS_SHADER);

    naga::front::wgsl::parse_str(&shader_source).expect("fallback post-process shader must parse");
    assert!(!shader_source.contains("texture_depth_2d"));
    assert!(!shader_source.contains("textureSample(scene_depth_tex"));
    assert!(shader_source.contains("@group(0) @binding(11) var scene_depth_tex: texture_2d<f32>;"));
    assert!(shader_source.contains(
        "return clamp((vec2<f32>(clamped) + vec2<f32>(0.5, 0.5)).y / f32(viewport_size.y), 0.0, 1.0);"
    ));
}

#[test]
fn viewport_depth_fallback_rewrites_depth_of_field_prepare_shader() {
    let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
        .depth_of_field_prepare_shader_source(DEPTH_OF_FIELD_PREPARE_SHADER);

    naga::front::wgsl::parse_str(&shader_source).expect("fallback DoF prepare shader must parse");
    assert!(!shader_source.contains("texture_depth_2d"));
    assert!(!shader_source.contains("textureLoad(scene_depth_tex"));
    assert!(shader_source.contains("@group(0) @binding(0) var scene_depth_tex: texture_2d<f32>;"));
    assert!(shader_source.contains("return clamp((vec2<f32>(clamped)"));
}

#[test]
fn viewport_depth_fallback_rewrites_velocity_camera_shader() {
    let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
        .velocity_camera_shader_source(VELOCITY_CAMERA_SHADER);

    naga::front::wgsl::parse_str(&shader_source)
        .expect("fallback camera velocity shader must parse");
    assert!(!shader_source.contains("texture_depth_2d"));
    assert!(!shader_source.contains("textureLoad(scene_depth_tex"));
    assert!(shader_source.contains("@group(0) @binding(0) var scene_depth_tex: texture_2d<f32>;"));
    assert!(shader_source.contains("return clamp((vec2<f32>(clamped)"));
}

#[test]
fn viewport_depth_fallback_rewrites_taa_resolve_shader() {
    let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
        .taa_resolve_shader_source(TAA_RESOLVE_SHADER);

    naga::front::wgsl::parse_str(&shader_source).expect("fallback TAA shader must parse");
    assert!(!shader_source.contains("texture_depth_2d"));
    assert!(!shader_source.contains("textureLoad(scene_depth_tex"));
    assert!(shader_source.contains("@group(0) @binding(1) var scene_depth_tex: texture_2d<f32>;"));
    assert!(shader_source.contains("return clamp((vec2<f32>(clamped)"));
}
