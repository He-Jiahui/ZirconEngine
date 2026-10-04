use super::{FULLSCREEN_TRIANGLE_SHADER, MOTION_VECTOR_TILE_MAX_FRAGMENT_SHADER};

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
fn motion_vector_tile_max_shader_parses_and_selects_dominant_tile_vector() {
    let assembled =
        format!("{FULLSCREEN_TRIANGLE_SHADER}\n{MOTION_VECTOR_TILE_MAX_FRAGMENT_SHADER}");
    validate_shader_source("motion_vector_tile_max.wgsl", &assembled);
    assert!(MOTION_VECTOR_TILE_MAX_FRAGMENT_SHADER
        .contains("@group(1) @binding(0) var motion_vector_source_tex"));
    assert!(MOTION_VECTOR_TILE_MAX_FRAGMENT_SHADER
        .contains("@group(2) @binding(0) var<uniform> motion_vector_tile_max_parameters"));
    assert!(!MOTION_VECTOR_TILE_MAX_FRAGMENT_SHADER.contains("@vertex"));
    assert!(FULLSCREEN_TRIANGLE_SHADER.contains("fn zr_fullscreen_triangle_vs"));
    assert!(MOTION_VECTOR_TILE_MAX_FRAGMENT_SHADER.contains("textureDimensions"));
    assert!(MOTION_VECTOR_TILE_MAX_FRAGMENT_SHADER.contains("fn choose_motion_vector_tile_max"));
    assert!(MOTION_VECTOR_TILE_MAX_FRAGMENT_SHADER.contains("fn motion_vector_tile_max"));
    assert!(MOTION_VECTOR_TILE_MAX_FRAGMENT_SHADER.contains("textureLoad(motion_vector_source_tex"));
    assert!(MOTION_VECTOR_TILE_MAX_FRAGMENT_SHADER
        .contains("motion_vector_tile_max_parameters.tile_span.xy"));
    assert!(MOTION_VECTOR_TILE_MAX_FRAGMENT_SHADER.contains("tile_coord * tile_span"));
}
