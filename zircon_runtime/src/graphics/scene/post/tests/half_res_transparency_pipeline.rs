use super::super::super::super::shader_sources::HALF_RES_TRANSPARENCY_SHADER;

#[test]
fn half_resolution_transparency_shader_parses() {
    let module = naga::front::wgsl::parse_str(HALF_RES_TRANSPARENCY_SHADER)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(HALF_RES_TRANSPARENCY_SHADER)));
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .unwrap_or_else(|error| panic!("{error}"));
}
