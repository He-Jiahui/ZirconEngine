use super::*;

#[test]
fn render_post_color_space_intermediate_hdr_uses_baseline_renderable_format() {
    assert_eq!(
        INTERMEDIATE_HDR_FORMAT_DEFAULT,
        RenderPostProcessTextureFormat::Rgba16Float
    );
    assert_eq!(INTERMEDIATE_HDR_FORMAT_DEFAULT.bytes_per_pixel(), 8);
    assert!(INTERMEDIATE_HDR_FORMAT_DEFAULT.is_hdr_color());
}

#[test]
fn render_post_color_lut_contract_keeps_power_of_two_sizes() {
    assert_eq!(COLOR_LUT_SIZE_DEFAULT, 32);
    assert_eq!(COLOR_LUT_SIZE_HIGH_QUALITY, 64);
    assert_eq!(
        COLOR_LUT_FORMAT,
        RenderPostProcessTextureFormat::Rgba16Float
    );
}

#[test]
fn render_post_output_transfer_defaults_to_srgb() {
    assert_eq!(OUTPUT_TRANSFER_DEFAULT, RenderOutputTransfer::SrgbNonlinear);
    assert_eq!(OUTPUT_TRANSFER_DEFAULT.label(), "srgb-nonlinear");
}
