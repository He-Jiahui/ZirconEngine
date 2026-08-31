use crate::core::math::Real;

pub const RGBA16F_TEXEL_SIZE_BYTES: usize = 8;
pub const RG16F_TEXEL_SIZE_BYTES: usize = 4;

pub fn encode_rg16f_texels(texels: &[[Real; 2]]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(texels.len() * RG16F_TEXEL_SIZE_BYTES);
    for texel in texels {
        push_f16_le_bytes(&mut bytes, texel[0]);
        push_f16_le_bytes(&mut bytes, texel[1]);
    }
    bytes
}

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

/// Decodes an exact RGBA16F payload directly into caller-owned storage.
///
/// The length check happens before the first write so malformed payloads leave
/// the destination unchanged. This is useful for bounded subresource decode
/// paths that already reserved their final texel slice.
pub fn decode_rgba16f_texels_into_exact(bytes: &[u8], output: &mut [[Real; 4]]) -> bool {
    let Some(expected_bytes) = output.len().checked_mul(RGBA16F_TEXEL_SIZE_BYTES) else {
        return false;
    };
    if bytes.len() != expected_bytes {
        return false;
    }
    for (texel, chunk) in output
        .iter_mut()
        .zip(bytes.chunks_exact(RGBA16F_TEXEL_SIZE_BYTES))
    {
        *texel = [
            f16_bits_to_f32(read_u16_at(chunk, 0)),
            f16_bits_to_f32(read_u16_at(chunk, 2)),
            f16_bits_to_f32(read_u16_at(chunk, 4)),
            f16_bits_to_f32(read_u16_at(chunk, 6)),
        ];
    }
    true
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

#[cfg(test)]
mod tests {
    use super::{decode_rgba16f_texels_into_exact, encode_rg16f_texels};

    #[test]
    fn rg16f_encoding_emits_two_half_float_channels_per_texel() {
        let bytes = encode_rg16f_texels(&[[1.0, 0.0], [0.5, 0.25]]);

        assert_eq!(bytes.len(), 8);
        assert_eq!(u16::from_le_bytes([bytes[0], bytes[1]]), 0x3c00);
        assert_eq!(u16::from_le_bytes([bytes[2], bytes[3]]), 0x0000);
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), 0x3800);
        assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 0x3400);
    }

    #[test]
    fn exact_rgba16f_decode_rejects_wrong_length_before_writing() {
        let mut output = vec![[9.0; 4]; 1];

        assert!(!decode_rgba16f_texels_into_exact(&[0; 7], &mut output));
        assert_eq!(output, vec![[9.0; 4]]);
    }

    #[test]
    fn exact_rgba16f_decode_writes_reserved_texels() {
        let bytes = [0x00, 0x3c, 0x00, 0x38, 0x00, 0x34, 0x00, 0x3c];
        let mut output = vec![[0.0; 4]; 1];

        assert!(decode_rgba16f_texels_into_exact(&bytes, &mut output));
        assert_eq!(output[0], [1.0, 0.5, 0.25, 1.0]);
    }
}
