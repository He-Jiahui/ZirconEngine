/// 将命中代理 GPU readback 的半精度通道还原为 CPU 数值；调用方按 little-endian 两字节读取。
pub(in crate::graphics::scene::scene_renderer::core) fn f16_bits_to_f32(bits: u16) -> f32 {
    let sign = u32::from(bits & 0x8000) << 16;
    let exponent = u32::from((bits >> 10) & 0x1f);
    let mantissa = u32::from(bits & 0x03ff);
    let value = match exponent {
        0 if mantissa == 0 => sign,
        0 => {
            let leading = mantissa.leading_zeros().saturating_sub(21);
            let normalized_mantissa = (mantissa << (leading + 1)) & 0x03ff;
            let normalized_exponent = 127_u32.saturating_sub(14 + leading);
            sign | (normalized_exponent << 23) | (normalized_mantissa << 13)
        }
        0x1f => sign | 0x7f80_0000 | (mantissa << 13),
        _ => sign | ((exponent + 112) << 23) | (mantissa << 13),
    };
    f32::from_bits(value)
}

#[cfg(test)]
#[path = "tests/half_float.rs"]
mod tests;
