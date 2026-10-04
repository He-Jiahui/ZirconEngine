use super::{PostProcessDepthSamplingMode, DEPTH_OF_FIELD_PREPARE_SHADER};

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
fn depth_of_field_prepare_shader_parses_and_encodes_coc_outputs() {
    validate_shader_source("depth_of_field_prepare.wgsl", DEPTH_OF_FIELD_PREPARE_SHADER);
    assert!(DEPTH_OF_FIELD_PREPARE_SHADER.contains("texture_depth_2d"));
    assert!(DEPTH_OF_FIELD_PREPARE_SHADER.contains("@group(0) @binding(2) var scene_color_tex"));
    assert!(DEPTH_OF_FIELD_PREPARE_SHADER.contains("fn load_scene_color"));
    assert!(DEPTH_OF_FIELD_PREPARE_SHADER.contains("fn signed_circle_of_confusion_radius"));
    assert!(DEPTH_OF_FIELD_PREPARE_SHADER.contains("fn circle_of_confusion_layers"));
    assert!(DEPTH_OF_FIELD_PREPARE_SHADER.contains("fn bokeh_prefilter_weight"));
    assert!(DEPTH_OF_FIELD_PREPARE_SHADER.contains("fn bokeh_prefilter_sample"));
    assert!(DEPTH_OF_FIELD_PREPARE_SHADER.contains("fn prefiltered_bokeh_seed"));
    assert!(DEPTH_OF_FIELD_PREPARE_SHADER.contains("clamp_prepare_coord(coord_i32"));
    assert!(
        DEPTH_OF_FIELD_PREPARE_SHADER.contains("load_scene_color(sample_coord) * sample_weight")
    );
    assert!(DEPTH_OF_FIELD_PREPARE_SHADER.contains("output.coc = vec4<f32>("));
    assert!(DEPTH_OF_FIELD_PREPARE_SHADER.contains("far_coc,\n        near_coc"));
    assert!(
        DEPTH_OF_FIELD_PREPARE_SHADER.contains("let bokeh_seed = prefiltered_bokeh_seed(coord)")
    );
    assert!(DEPTH_OF_FIELD_PREPARE_SHADER.contains("output.bokeh = bokeh_seed"));
}

#[test]
fn depth_of_field_prepare_fallback_shader_parses_without_depth_texture_sampling() {
    let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
        .depth_of_field_prepare_shader_source(DEPTH_OF_FIELD_PREPARE_SHADER);

    validate_shader_source(
        "depth_of_field_prepare.viewport_fallback.wgsl",
        &shader_source,
    );
    assert!(!shader_source.contains("texture_depth_2d"));
    assert!(!shader_source.contains("textureLoad(scene_depth_tex"));
    assert!(shader_source.contains("@group(0) @binding(0) var scene_depth_tex: texture_2d<f32>;"));
}
