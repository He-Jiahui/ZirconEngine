use super::super::super::super::shader_sources::OUTPUT_TRANSFER_SHADER;

#[test]
fn output_transfer_shader_parses() {
    let module = naga::front::wgsl::parse_str(OUTPUT_TRANSFER_SHADER)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(OUTPUT_TRANSFER_SHADER)));
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .unwrap_or_else(|error| panic!("{error}"));
}
