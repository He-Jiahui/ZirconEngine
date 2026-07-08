use crate::core::math::Real;

pub const RGBA16F_TEXEL_SIZE_BYTES: usize = 8;

pub fn encode_rgba16f_texels(texels: &[[Real; 4]]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(texels.len() * RGBA16F_TEXEL_SIZE_BYTES);
    append_rgba16f_texels(&mut bytes, texels);
    bytes
}

pub fn append_rgba16f_texels(bytes: &mut Vec<u8>, texels: &[[Real; 4]]) {
    for texel in texels {
        for channel in *texel {
            push_f16_le_bytes(bytes, channel);
        }
    }
}

pub fn append_rgb_as_rgba16f_texels(bytes: &mut Vec<u8>, texels: &[[Real; 3]], alpha: Real) {
    for texel in texels {
        push_f16_le_bytes(bytes, texel[0]);
        push_f16_le_bytes(bytes, texel[1]);
        push_f16_le_bytes(bytes, texel[2]);
        push_f16_le_bytes(bytes, alpha);
    }
}

pub fn decode_rgba16f_texels(bytes: &[u8]) -> Vec<[Real; 4]> {
    bytes
        .chunks_exact(RGBA16F_TEXEL_SIZE_BYTES)
        .map(|chunk| {
            [
                f16_bits_to_f32(read_u16_at(chunk, 0)),
                f16_bits_to_f32(read_u16_at(chunk, 2)),
                f16_bits_to_f32(read_u16_at(chunk, 4)),
                f16_bits_to_f32(read_u16_at(chunk, 6)),
            ]
        })
        .collect()
}

pub fn decode_rgb_from_rgba16f_texels(bytes: &[u8]) -> Vec<[Real; 3]> {
    bytes
        .chunks_exact(RGBA16F_TEXEL_SIZE_BYTES)
        .map(|chunk| {
            [
                f16_bits_to_f32(read_u16_at(chunk, 0)),
                f16_bits_to_f32(read_u16_at(chunk, 2)),
                f16_bits_to_f32(read_u16_at(chunk, 4)),
            ]
        })
        .collect()
}

fn push_f16_le_bytes(bytes: &mut Vec<u8>, value: Real) {
    bytes.extend_from_slice(&f32_to_f16_bits(value).to_le_bytes());
}

fn f32_to_f16_bits(value: Real) -> u16 {
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

fn f16_bits_to_f32(bits: u16) -> Real {
    let sign = ((bits & 0x8000) as u32) << 16;
    let exponent = ((bits >> 10) & 0x1f) as u32;
    let fraction = (bits & 0x03ff) as u32;
    let f32_bits = if exponent == 0 {
        if fraction == 0 {
            sign
        } else {
            let mut normalized_fraction = fraction;
            let mut normalized_exponent = -14;
            while (normalized_fraction & 0x0400) == 0 {
                normalized_fraction <<= 1;
                normalized_exponent -= 1;
            }
            normalized_fraction &= 0x03ff;
            sign | (((normalized_exponent + 127) as u32) << 23) | (normalized_fraction << 13)
        }
    } else if exponent == 0x1f {
        sign | 0x7f80_0000 | (fraction << 13)
    } else {
        sign | ((exponent + 112) << 23) | (fraction << 13)
    };
    f32::from_bits(f32_bits)
}

fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}
