use super::{PostProcessDepthSamplingMode, VELOCITY_CAMERA_SHADER};

fn validate_shader_source(name: &str, shader_source: &str) {
    let module = naga::front::wgsl::parse_str(shader_source)
        .unwrap_or_else(|error| panic!("{name}: {}", error.emit_to_string(shader_source)));
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .unwrap_or_else(|error| panic!("{name}: {error}"));
}

#[test]
fn velocity_camera_shader_parses_and_reconstructs_previous_uv() {
    validate_shader_source("velocity_camera.wgsl", VELOCITY_CAMERA_SHADER);
    assert!(VELOCITY_CAMERA_SHADER.contains("texture_depth_2d"));
    assert!(VELOCITY_CAMERA_SHADER.contains("fn load_velocity_scene_depth"));
    assert!(VELOCITY_CAMERA_SHADER.contains("fn clip_to_uv"));
    assert!(VELOCITY_CAMERA_SHADER.contains("fn velocity_camera_velocity"));
    assert!(VELOCITY_CAMERA_SHADER.contains("current_world_from_clip"));
    assert!(VELOCITY_CAMERA_SHADER.contains("previous_clip_from_world"));
}

#[test]
fn velocity_camera_fallback_shader_parses_without_depth_texture_sampling() {
    let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
        .velocity_camera_shader_source(VELOCITY_CAMERA_SHADER);

    validate_shader_source("velocity_camera.viewport_fallback.wgsl", &shader_source);
    assert!(!shader_source.contains("texture_depth_2d"));
    assert!(!shader_source.contains("textureLoad(scene_depth_tex"));
}
