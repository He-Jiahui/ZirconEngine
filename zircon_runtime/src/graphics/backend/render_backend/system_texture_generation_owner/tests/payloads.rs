use crate::core::framework::render::{ENVIRONMENT_BRDF_LUT_HEIGHT, ENVIRONMENT_BRDF_LUT_WIDTH};

use super::super::resources::{EFFECT_LUT_3D_SIZE, EFFECT_LUT_WIDTH, SYSTEM_TEXTURE_UPLOAD_BYTES};
use super::{
    black_alpha_one_rgba8_bytes, black_cube_rgba16float_bytes, black_rgba16float_bytes,
    black_rgba8_bytes, effect_lut_3d_rgba8_bytes, effect_lut_rgba8_bytes,
    irradiance_volume_black_rgba8_bytes, normal_rgba8_bytes, white_rgba8_bytes,
};

#[test]
fn system_texture_payload_lengths_match_the_reported_upload_budget() {
    let owned_payload_bytes = [
        black_cube_rgba16float_bytes().len(),
        black_rgba8_bytes().len(),
        black_alpha_one_rgba8_bytes().len(),
        white_rgba8_bytes().len(),
        normal_rgba8_bytes().len(),
        black_rgba16float_bytes().len(),
        irradiance_volume_black_rgba8_bytes().len(),
        effect_lut_rgba8_bytes().len(),
        effect_lut_3d_rgba8_bytes().len(),
    ];
    let brdf_lut_bytes = (ENVIRONMENT_BRDF_LUT_WIDTH * ENVIRONMENT_BRDF_LUT_HEIGHT * 4) as usize;

    assert_eq!(owned_payload_bytes, [48, 4, 4, 4, 4, 8, 24, 256, 32]);
    assert_eq!(
        owned_payload_bytes.into_iter().sum::<usize>() + brdf_lut_bytes,
        SYSTEM_TEXTURE_UPLOAD_BYTES as usize,
    );
}

#[test]
fn generated_effect_lut_is_s_curve_with_stable_texture_stride() {
    let bytes = effect_lut_rgba8_bytes();

    assert_eq!(bytes.len(), (EFFECT_LUT_WIDTH * 4) as usize);
    assert_eq!(&bytes[0..4], &[0, 0, 0, 255]);
    assert_eq!(&bytes[bytes.len() - 4..], &[255, 255, 255, 255]);
    let midpoint = (EFFECT_LUT_WIDTH / 2 * 4) as usize;
    assert!(bytes[midpoint] > 127);
    assert_eq!(bytes[midpoint], bytes[midpoint + 1]);
    assert_eq!(bytes[midpoint], bytes[midpoint + 2]);
    assert_eq!(bytes[midpoint + 3], 255);
}

#[test]
fn generated_effect_lut_3d_is_identity_cube() {
    let bytes = effect_lut_3d_rgba8_bytes();

    assert_eq!(bytes.len(), (EFFECT_LUT_3D_SIZE.pow(3) * 4) as usize);
    assert_eq!(&bytes[0..4], &[0, 0, 0, 255]);
    assert_eq!(&bytes[bytes.len() - 4..], &[255, 255, 255, 255]);
}
