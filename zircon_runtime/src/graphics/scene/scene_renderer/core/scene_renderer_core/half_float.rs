pub(in crate::graphics::scene::scene_renderer::core) fn push_f16_le_bytes(
    bytes: &mut Vec<u8>,
    value: f32,
) {
    bytes.extend_from_slice(&f32_to_f16_bits(value).to_le_bytes());
}

pub(in crate::graphics::scene::scene_renderer::core) fn f32_to_f16_bits(value: f32) -> u16 {
    let value = value.clamp(0.0, f32::MAX);
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    let mantissa = bits & 0x7f_ffff;

    if exponent <= 0 {
        if exponent < -10 {
            return sign;
        }
        let mantissa = mantissa | 0x80_0000;
        let shift = (14 - exponent) as u32;
        let mut half = (mantissa >> shift) as u16;
        if ((mantissa >> shift.saturating_sub(1)) & 1) != 0 {
            half = half.saturating_add(1);
        }
        return sign | half;
    }

    if exponent >= 31 {
        return sign | 0x7c00;
    }

    let mut half = sign | ((exponent as u16) << 10) | ((mantissa >> 13) as u16);
    if (mantissa & 0x1000) != 0 {
        half = half.saturating_add(1);
    }
    half
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_to_f16_bits_encodes_common_positive_values() {
        assert_eq!(f32_to_f16_bits(0.0), 0x0000);
        assert_eq!(f32_to_f16_bits(0.25), 0x3400);
        assert_eq!(f32_to_f16_bits(0.5), 0x3800);
        assert_eq!(f32_to_f16_bits(1.0), 0x3c00);
    }
}
