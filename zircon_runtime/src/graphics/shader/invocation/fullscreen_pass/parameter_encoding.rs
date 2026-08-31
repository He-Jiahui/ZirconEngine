use super::super::compiler::ShaderParameterValue;

pub(super) fn fullscreen_parameter_words(value: &ShaderParameterValue) -> [u32; 4] {
    // A fixed vec4-sized slot keeps generated fullscreen parameter ABI stable.
    match value {
        ShaderParameterValue::Bool { value } => [u32::from(*value), 0, 0, 0],
        ShaderParameterValue::F32 { value } => [value.to_bits(), 0, 0, 0],
        ShaderParameterValue::I32 { value } => [*value as u32, 0, 0, 0],
        ShaderParameterValue::U32 { value } => [*value, 0, 0, 0],
        ShaderParameterValue::Vec2 { value } => [value[0].to_bits(), value[1].to_bits(), 0, 0],
        ShaderParameterValue::Vec3 { value } => [
            value[0].to_bits(),
            value[1].to_bits(),
            value[2].to_bits(),
            0,
        ],
        ShaderParameterValue::Vec4 { value } => [
            value[0].to_bits(),
            value[1].to_bits(),
            value[2].to_bits(),
            value[3].to_bits(),
        ],
    }
}
