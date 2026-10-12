use super::super::super::super::shader_sources::FXAA_SHADER;

#[test]
fn fxaa_shader_parses() {
    let module = naga::front::wgsl::parse_str(FXAA_SHADER)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(FXAA_SHADER)));
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .unwrap_or_else(|error| panic!("{error}"));
}
