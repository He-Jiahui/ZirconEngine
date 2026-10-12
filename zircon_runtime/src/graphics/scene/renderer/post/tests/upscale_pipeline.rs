use super::super::super::super::shader_sources::UPSCALE_SHADER;

#[test]
fn upscale_shader_parses() {
    let module = naga::front::wgsl::parse_str(UPSCALE_SHADER)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(UPSCALE_SHADER)));
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .unwrap_or_else(|error| panic!("{error}"));
}
