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

/// 帧提交后对 GPU LUT 读回的 CPU 诊断结果，用于验证烘焙结果与身份映射或指定参考变换。
/// 构造时提供的参考函数必须接受按 3D LUT 顺序生成的源颜色；此报告不参与当帧渲染决策。
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
#[path = "tests/color_lut_readback.rs"]
mod tests;
