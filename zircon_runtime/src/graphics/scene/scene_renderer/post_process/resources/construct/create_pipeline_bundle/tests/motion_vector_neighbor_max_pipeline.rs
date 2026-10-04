use super::MOTION_VECTOR_NEIGHBOR_MAX_SHADER;

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
fn motion_vector_neighbor_max_shader_parses_and_selects_dominant_neighbor() {
    validate_shader_source(
        "motion_vector_neighbor_max.wgsl",
        MOTION_VECTOR_NEIGHBOR_MAX_SHADER,
    );
    assert!(MOTION_VECTOR_NEIGHBOR_MAX_SHADER
        .contains("@group(0) @binding(0) var motion_vector_tile_max_coarse_tex"));
    assert!(MOTION_VECTOR_NEIGHBOR_MAX_SHADER.contains("textureDimensions"));
    assert!(MOTION_VECTOR_NEIGHBOR_MAX_SHADER.contains("fn choose_motion_vector_neighbor_max"));
    assert!(MOTION_VECTOR_NEIGHBOR_MAX_SHADER.contains("fn motion_vector_neighbor_max"));
    assert!(
        MOTION_VECTOR_NEIGHBOR_MAX_SHADER.contains("textureLoad(motion_vector_tile_max_coarse_tex")
    );
    assert!(MOTION_VECTOR_NEIGHBOR_MAX_SHADER.contains("full_res_coord / vec2<u32>(4u, 4u)"));
    assert!(MOTION_VECTOR_NEIGHBOR_MAX_SHADER.contains("coord_i32 + vec2<i32>(1, 1)"));
}
