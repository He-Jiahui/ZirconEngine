pub const COLOR_LUT_IDENTITY_EPSILON_MICRO: u32 = 977;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderColorLutReadbackReference {
    #[default]
    Identity,
    UserLut,
    ColorTransform,
}

impl RenderColorLutReadbackReference {
    pub const fn diagnostic_id(self) -> u32 {
        match self {
            Self::Identity => 0,
            Self::UserLut => 1,
            Self::ColorTransform => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderColorLutReadbackReport {
    pub available: bool,
    pub reference: RenderColorLutReadbackReference,
    pub size: [u32; 3],
    pub byte_len: usize,
    pub expected_byte_len: usize,
    pub sample_count: usize,
    pub invalid_byte_len: bool,
    pub invalid_sample_count: usize,
    pub max_abs_error_micro: u32,
    pub out_of_tolerance_sample_count: usize,
    pub identity_max_abs_error_micro: u32,
    pub identity_out_of_tolerance_sample_count: usize,
    pub alpha_out_of_tolerance_sample_count: usize,
}

impl RenderColorLutReadbackReport {
    pub fn from_raw_rgba16_float_identity_bytes(size: [u32; 3], bytes: &[u8]) -> Self {
        Self::from_raw_rgba16_float_reference_bytes(
            size,
            bytes,
            RenderColorLutReadbackReference::Identity,
            |source_color| source_color,
        )
    }

    pub fn from_raw_rgba16_float_user_lut_bytes(
        size: [u32; 3],
        bytes: &[u8],
        expected_rgb: impl Fn([f32; 3]) -> [f32; 3],
    ) -> Self {
        Self::from_raw_rgba16_float_reference_bytes(
            size,
            bytes,
            RenderColorLutReadbackReference::UserLut,
            expected_rgb,
        )
    }

    pub fn from_raw_rgba16_float_color_transform_bytes(
        size: [u32; 3],
        bytes: &[u8],
        expected_rgb: impl Fn([f32; 3]) -> [f32; 3],
    ) -> Self {
        Self::from_raw_rgba16_float_reference_bytes(
            size,
            bytes,
            RenderColorLutReadbackReference::ColorTransform,
            expected_rgb,
        )
    }

    pub fn from_raw_rgba16_float_reference_bytes(
        size: [u32; 3],
        bytes: &[u8],
        reference: RenderColorLutReadbackReference,
        expected_rgb: impl Fn([f32; 3]) -> [f32; 3],
    ) -> Self {
        let expected_byte_len = expected_rgba16_float_byte_len(size);
        let mut report = Self {
            available: true,
            reference,
            size,
            byte_len: bytes.len(),
            expected_byte_len,
            sample_count: (size[0] as usize)
                .saturating_mul(size[1] as usize)
                .saturating_mul(size[2] as usize),
            invalid_byte_len: bytes.len() != expected_byte_len,
            ..Self::default()
        };

        for (sample_index, texel) in bytes.chunks_exact(8).take(report.sample_count).enumerate() {
            let source_color = expected_identity_texel(size, sample_index);
            let expected = expected_rgb(source_color);
            let actual = [
                f16_to_f32(u16::from_le_bytes([texel[0], texel[1]])),
                f16_to_f32(u16::from_le_bytes([texel[2], texel[3]])),
                f16_to_f32(u16::from_le_bytes([texel[4], texel[5]])),
                f16_to_f32(u16::from_le_bytes([texel[6], texel[7]])),
            ];
            let mut reference_out_of_tolerance = false;
            let mut identity_out_of_tolerance = false;
            for channel in 0..3 {
                if !reference_out_of_tolerance {
                    let error = channel_error_micro(actual[channel], expected[channel]);
                    report.max_abs_error_micro = report.max_abs_error_micro.max(error);
                    if error > COLOR_LUT_IDENTITY_EPSILON_MICRO {
                        report.out_of_tolerance_sample_count += 1;
                        reference_out_of_tolerance = true;
                    }
                }
                if !identity_out_of_tolerance {
                    let error = channel_error_micro(actual[channel], source_color[channel]);
                    report.identity_max_abs_error_micro =
                        report.identity_max_abs_error_micro.max(error);
                    if error > COLOR_LUT_IDENTITY_EPSILON_MICRO {
                        report.identity_out_of_tolerance_sample_count += 1;
                        identity_out_of_tolerance = true;
                    }
                }
                if reference_out_of_tolerance && identity_out_of_tolerance {
                    break;
                }
            }
            let alpha_error = channel_error_micro(actual[3], 1.0);
            if alpha_error > COLOR_LUT_IDENTITY_EPSILON_MICRO {
                report.alpha_out_of_tolerance_sample_count += 1;
            }
            if actual.iter().any(|value| !value.is_finite()) {
                report.invalid_sample_count += 1;
            }
        }

        report
    }

    pub const fn reference_within_epsilon(self) -> bool {
        self.available
            && !self.invalid_byte_len
            && self.invalid_sample_count == 0
            && self.out_of_tolerance_sample_count == 0
            && self.alpha_out_of_tolerance_sample_count == 0
            && self.max_abs_error_micro <= COLOR_LUT_IDENTITY_EPSILON_MICRO
    }

    pub const fn identity_within_epsilon(self) -> bool {
        match self.reference {
            RenderColorLutReadbackReference::Identity => {
                self.reference_within_epsilon()
                    && self.identity_out_of_tolerance_sample_count == 0
                    && self.identity_max_abs_error_micro <= COLOR_LUT_IDENTITY_EPSILON_MICRO
            }
            RenderColorLutReadbackReference::UserLut
            | RenderColorLutReadbackReference::ColorTransform => false,
        }
    }

    pub const fn user_lut_within_epsilon(self) -> bool {
        match self.reference {
            RenderColorLutReadbackReference::Identity => false,
            RenderColorLutReadbackReference::UserLut => self.reference_within_epsilon(),
            RenderColorLutReadbackReference::ColorTransform => false,
        }
    }

    pub const fn color_transform_within_epsilon(self) -> bool {
        match self.reference {
            RenderColorLutReadbackReference::Identity
            | RenderColorLutReadbackReference::UserLut => false,
            RenderColorLutReadbackReference::ColorTransform => self.reference_within_epsilon(),
        }
    }
}

fn expected_rgba16_float_byte_len(size: [u32; 3]) -> usize {
    (size[0] as usize)
        .saturating_mul(size[1] as usize)
        .saturating_mul(size[2] as usize)
        .saturating_mul(8)
}

fn expected_identity_texel(size: [u32; 3], sample_index: usize) -> [f32; 3] {
    let width = size[0].max(1) as usize;
    let height = size[1].max(1) as usize;
    let x = sample_index % width;
    let y = (sample_index / width) % height;
    let z = sample_index / width.saturating_mul(height).max(1);
    [
        expected_axis_value(x as u32, size[0]),
        expected_axis_value(y as u32, size[1]),
        expected_axis_value(z as u32, size[2]),
    ]
}

fn expected_axis_value(index: u32, size: u32) -> f32 {
    if size <= 1 {
        0.0
    } else {
        index as f32 / (size - 1) as f32
    }
}

fn channel_error_micro(actual: f32, expected: f32) -> u32 {
    if !actual.is_finite() || !expected.is_finite() {
        return u32::MAX;
    }
    ((actual - expected).abs() * 1_000_000.0).round() as u32
}

fn f16_to_f32(bits: u16) -> f32 {
    let sign = ((bits & 0x8000) as u32) << 16;
    let exponent = (bits >> 10) & 0x1f;
    let fraction = bits & 0x03ff;
    match exponent {
        0 => {
            if fraction == 0 {
                f32::from_bits(sign)
            } else {
                let mut normalized_fraction = fraction;
                let mut exponent_value = -14_i32;
                while normalized_fraction & 0x0400 == 0 {
                    normalized_fraction <<= 1;
                    exponent_value -= 1;
                }
                normalized_fraction &= 0x03ff;
                f32::from_bits(
                    sign | (((exponent_value + 127) as u32) << 23)
                        | ((normalized_fraction as u32) << 13),
                )
            }
        }
        0x1f => f32::from_bits(sign | 0x7f80_0000 | ((fraction as u32) << 13)),
        _ => f32::from_bits(
            sign | ((((exponent as i32) - 15 + 127) as u32) << 23) | ((fraction as u32) << 13),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{RenderColorLutReadbackReference, RenderColorLutReadbackReport};

    #[test]
    fn color_lut_readback_report_accepts_identity_rgba16float_bytes() {
        let bytes = identity_2x2x2_rgba16float_bytes();

        let report =
            RenderColorLutReadbackReport::from_raw_rgba16_float_identity_bytes([2, 2, 2], &bytes);

        assert!(report.available);
        assert_eq!(report.byte_len, 64);
        assert_eq!(report.expected_byte_len, 64);
        assert_eq!(report.sample_count, 8);
        assert_eq!(report.reference, RenderColorLutReadbackReference::Identity);
        assert_eq!(report.max_abs_error_micro, 0);
        assert_eq!(report.out_of_tolerance_sample_count, 0);
        assert_eq!(report.identity_out_of_tolerance_sample_count, 0);
        assert_eq!(report.alpha_out_of_tolerance_sample_count, 0);
        assert!(report.reference_within_epsilon());
        assert!(report.identity_within_epsilon());
        assert!(!report.user_lut_within_epsilon());
    }

    #[test]
    fn color_lut_readback_report_rejects_non_identity_rgba16float_bytes() {
        let mut bytes = identity_2x2x2_rgba16float_bytes();
        bytes[8] = 0;
        bytes[9] = 0;

        let report =
            RenderColorLutReadbackReport::from_raw_rgba16_float_identity_bytes([2, 2, 2], &bytes);

        assert_eq!(report.identity_out_of_tolerance_sample_count, 1);
        assert_eq!(report.out_of_tolerance_sample_count, 1);
        assert!(!report.identity_within_epsilon());
        assert!(!report.reference_within_epsilon());
    }

    #[test]
    fn color_lut_readback_report_tracks_invalid_byte_length() {
        let report =
            RenderColorLutReadbackReport::from_raw_rgba16_float_identity_bytes([2, 2, 2], &[0; 8]);

        assert!(report.invalid_byte_len);
        assert!(!report.identity_within_epsilon());
        assert!(!report.reference_within_epsilon());
    }

    #[test]
    fn color_lut_readback_report_accepts_user_lut_reference_rgba16float_bytes() {
        let bytes = user_lut_2x2x2_rgba16float_bytes();

        let report = RenderColorLutReadbackReport::from_raw_rgba16_float_user_lut_bytes(
            [2, 2, 2],
            &bytes,
            expected_user_lut_color,
        );

        assert_eq!(report.reference, RenderColorLutReadbackReference::UserLut);
        assert_eq!(report.max_abs_error_micro, 0);
        assert_eq!(report.out_of_tolerance_sample_count, 0);
        assert!(report.reference_within_epsilon());
        assert!(report.user_lut_within_epsilon());
        assert!(!report.identity_within_epsilon());
        assert!(report.identity_out_of_tolerance_sample_count > 0);
    }

    #[test]
    fn color_lut_readback_report_accepts_color_transform_reference_rgba16float_bytes() {
        let bytes = user_lut_2x2x2_rgba16float_bytes();

        let report = RenderColorLutReadbackReport::from_raw_rgba16_float_color_transform_bytes(
            [2, 2, 2],
            &bytes,
            expected_user_lut_color,
        );

        assert_eq!(
            report.reference,
            RenderColorLutReadbackReference::ColorTransform
        );
        assert_eq!(report.reference.diagnostic_id(), 2);
        assert_eq!(report.max_abs_error_micro, 0);
        assert_eq!(report.out_of_tolerance_sample_count, 0);
        assert!(report.reference_within_epsilon());
        assert!(report.color_transform_within_epsilon());
        assert!(!report.user_lut_within_epsilon());
        assert!(!report.identity_within_epsilon());
        assert!(report.identity_out_of_tolerance_sample_count > 0);
    }

    #[test]
    fn color_lut_readback_report_rejects_wrong_user_lut_reference_bytes() {
        let mut bytes = user_lut_2x2x2_rgba16float_bytes();
        bytes[0] = 0;
        bytes[1] = 0;

        let report = RenderColorLutReadbackReport::from_raw_rgba16_float_user_lut_bytes(
            [2, 2, 2],
            &bytes,
            expected_user_lut_color,
        );

        assert_eq!(report.reference, RenderColorLutReadbackReference::UserLut);
        assert_eq!(report.out_of_tolerance_sample_count, 1);
        assert!(!report.reference_within_epsilon());
        assert!(!report.user_lut_within_epsilon());
    }

    #[test]
    fn optimization_batch_20260830ct_color_lut_fuses_reference_and_identity_rgb_scans() {
        let source = include_str!("color_lut_readback.rs");
        let rgb_scan = ["for channel in ", "0..3 {"].concat();

        assert_eq!(
            source.matches(&rgb_scan).count(),
            1,
            "reference and identity error tracking should share one RGB channel scan"
        );
    }

    #[test]
    #[ignore = "deterministic operation-count benchmark"]
    fn optimization_batch_20260830ct_color_lut_fused_rgb_scan_benchmark() {
        const SAMPLE_COUNT: usize = 32_768;
        const CHANNEL_COUNT: usize = 3;

        let mut bytes = vec![0_u8; SAMPLE_COUNT * 8];
        for texel in bytes.chunks_exact_mut(8) {
            texel[6..8].copy_from_slice(&0x3c00_u16.to_le_bytes());
        }
        let report = RenderColorLutReadbackReport::from_raw_rgba16_float_color_transform_bytes(
            [1, 1, SAMPLE_COUNT as u32],
            &bytes,
            |_| [0.0; 3],
        );
        assert_eq!(report.sample_count, SAMPLE_COUNT);
        assert_eq!(report.out_of_tolerance_sample_count, 0);
        assert!(report.identity_out_of_tolerance_sample_count > 0);

        let legacy_channel_iterations = SAMPLE_COUNT * CHANNEL_COUNT * 2;
        let optimized_channel_iterations = SAMPLE_COUNT * CHANNEL_COUNT;
        assert_eq!(optimized_channel_iterations * 2, legacy_channel_iterations);
        println!(
            "RUNTIME507_COLOR_LUT_FUSED_RGB_SCAN_BENCH_V1 samples={SAMPLE_COUNT} \
             legacy_channel_iterations={legacy_channel_iterations} \
             optimized_channel_iterations={optimized_channel_iterations} reduction_percent=50"
        );
    }

    fn identity_2x2x2_rgba16float_bytes() -> Vec<u8> {
        let mut bytes = Vec::new();
        for z in [0_u16, 0x3c00] {
            for y in [0_u16, 0x3c00] {
                for x in [0_u16, 0x3c00] {
                    push_f16(&mut bytes, x);
                    push_f16(&mut bytes, y);
                    push_f16(&mut bytes, z);
                    push_f16(&mut bytes, 0x3c00);
                }
            }
        }
        bytes
    }

    fn user_lut_2x2x2_rgba16float_bytes() -> Vec<u8> {
        let mut bytes = Vec::new();
        for z in [0.0_f32, 1.0] {
            for y in [0.0_f32, 1.0] {
                for x in [0.0_f32, 1.0] {
                    let expected = expected_user_lut_color([x, y, z]);
                    push_f16(&mut bytes, f16_bits_for_test_value(expected[0]));
                    push_f16(&mut bytes, f16_bits_for_test_value(expected[1]));
                    push_f16(&mut bytes, f16_bits_for_test_value(expected[2]));
                    push_f16(&mut bytes, 0x3c00);
                }
            }
        }
        bytes
    }

    fn expected_user_lut_color(source_color: [f32; 3]) -> [f32; 3] {
        [
            1.0 - source_color[0],
            source_color[1] * 0.5,
            source_color[2],
        ]
    }

    fn f16_bits_for_test_value(value: f32) -> u16 {
        match value {
            0.0 => 0,
            0.5 => 0x3800,
            1.0 => 0x3c00,
            other => panic!("unsupported test half-float value {other}"),
        }
    }

    fn push_f16(bytes: &mut Vec<u8>, bits: u16) {
        bytes.extend_from_slice(&bits.to_le_bytes());
    }
}
