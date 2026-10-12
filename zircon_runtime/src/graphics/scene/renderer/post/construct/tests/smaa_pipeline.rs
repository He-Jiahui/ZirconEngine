use super::super::super::super::shader_sources::SMAA_SHADER;
use super::SMAA_STAGE_FORMAT;

#[test]
fn smaa_shader_parses() {
    let module = naga::front::wgsl::parse_str(SMAA_SHADER)
        .unwrap_or_else(|error| panic!("{}", error.emit_to_string(SMAA_SHADER)));
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn smaa_internal_stage_format_is_sdr_weight_texture() {
    assert_eq!(SMAA_STAGE_FORMAT, wgpu::TextureFormat::Rgba8Unorm);
}
