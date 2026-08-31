use std::sync::{Arc, OnceLock};

use crate::core::framework::render::{encode_rgba16f_texels, CANONICAL_ENVIRONMENT_PBR_RECIPE};

use super::resources::{
    BLACK_CUBE_FACE_COUNT, EFFECT_LUT_3D_SIZE, EFFECT_LUT_WIDTH, IRRADIANCE_VOLUME_FALLBACK_DEPTH,
    IRRADIANCE_VOLUME_FALLBACK_HEIGHT,
};

pub(super) const ENVIRONMENT_BRDF_LUT_ARTIFACT_BYTE_LEN: usize = CANONICAL_ENVIRONMENT_PBR_RECIPE
    .brdf_lut_recipe()
    .expected_byte_len();
pub(super) const ENVIRONMENT_BRDF_LUT_ARTIFACT_SHA256: [u8; 32] = [
    0x40, 0x69, 0x56, 0x35, 0x6b, 0x13, 0x6b, 0xd0, 0x79, 0xcd, 0xcc, 0xe8, 0xdc, 0xb8, 0x6f, 0x9e,
    0x20, 0xd5, 0x96, 0x68, 0x1f, 0x74, 0x57, 0xad, 0x38, 0xd9, 0x1a, 0x7e, 0xe4, 0x72, 0x67, 0x4d,
];

const ENVIRONMENT_BRDF_LUT_RG16FLOAT_BYTES: &[u8; ENVIRONMENT_BRDF_LUT_ARTIFACT_BYTE_LEN] =
    include_bytes!("environment_brdf_lut_rg16float.bin");

pub(super) fn builtin_environment_brdf_lut_rg16float_bytes() -> Arc<[u8]> {
    Arc::from(&ENVIRONMENT_BRDF_LUT_RG16FLOAT_BYTES[..])
}

pub(super) fn black_cube_rgba16float_bytes() -> Arc<[u8]> {
    encode_rgba16f_texels(&[[0.0, 0.0, 0.0, 1.0]; BLACK_CUBE_FACE_COUNT as usize]).into()
}

pub(super) fn black_rgba8_bytes() -> Arc<[u8]> {
    Arc::from([0_u8, 0, 0, 0])
}

pub(super) fn black_alpha_one_rgba8_bytes() -> Arc<[u8]> {
    Arc::from([0_u8, 0, 0, u8::MAX])
}

pub(super) fn white_rgba8_bytes() -> Arc<[u8]> {
    Arc::from([u8::MAX; 4])
}

pub(super) fn normal_rgba8_bytes() -> Arc<[u8]> {
    Arc::from([128_u8, 128, u8::MAX, u8::MAX])
}

pub(super) fn black_rgba16float_bytes() -> Arc<[u8]> {
    Arc::from([0_u8; 8])
}

pub(super) fn irradiance_volume_black_rgba8_bytes() -> Arc<[u8]> {
    Arc::from(
        vec![
            0_u8;
            (IRRADIANCE_VOLUME_FALLBACK_HEIGHT * IRRADIANCE_VOLUME_FALLBACK_DEPTH * 4) as usize
        ]
        .into_boxed_slice(),
    )
}

pub(super) fn effect_lut_rgba8_bytes() -> Arc<[u8]> {
    static ENCODED_BYTES: OnceLock<Arc<[u8]>> = OnceLock::new();
    Arc::clone(ENCODED_BYTES.get_or_init(|| {
        let mut bytes = Vec::with_capacity((EFFECT_LUT_WIDTH * 4) as usize);
        for index in 0..EFFECT_LUT_WIDTH {
            let t = index as f32 / (EFFECT_LUT_WIDTH - 1) as f32;
            let shaped = t * t * (3.0 - 2.0 * t);
            let value = (shaped * u8::MAX as f32).round() as u8;
            bytes.extend_from_slice(&[value, value, value, u8::MAX]);
        }
        Arc::from(bytes.into_boxed_slice())
    }))
}

pub(super) fn effect_lut_3d_rgba8_bytes() -> Arc<[u8]> {
    static ENCODED_BYTES: OnceLock<Arc<[u8]>> = OnceLock::new();
    Arc::clone(ENCODED_BYTES.get_or_init(|| {
        let texel_count = EFFECT_LUT_3D_SIZE.pow(3);
        let mut bytes = Vec::with_capacity((texel_count * 4) as usize);
        for blue in 0..EFFECT_LUT_3D_SIZE {
            for green in 0..EFFECT_LUT_3D_SIZE {
                for red in 0..EFFECT_LUT_3D_SIZE {
                    let scale = u8::MAX as f32 / (EFFECT_LUT_3D_SIZE - 1) as f32;
                    bytes.extend_from_slice(&[
                        (red as f32 * scale).round() as u8,
                        (green as f32 * scale).round() as u8,
                        (blue as f32 * scale).round() as u8,
                        u8::MAX,
                    ]);
                }
            }
        }
        Arc::from(bytes.into_boxed_slice())
    }))
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{ENVIRONMENT_BRDF_LUT_HEIGHT, ENVIRONMENT_BRDF_LUT_WIDTH};

    use super::super::resources::{
        EFFECT_LUT_3D_SIZE, EFFECT_LUT_WIDTH, SYSTEM_TEXTURE_UPLOAD_BYTES,
    };
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
        let brdf_lut_bytes =
            (ENVIRONMENT_BRDF_LUT_WIDTH * ENVIRONMENT_BRDF_LUT_HEIGHT * 4) as usize;

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
}
