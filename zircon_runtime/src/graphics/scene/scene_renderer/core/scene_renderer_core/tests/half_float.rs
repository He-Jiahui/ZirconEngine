use super::*;

#[test]
fn f16_bits_to_f32_decodes_signed_normals_and_special_values() {
    assert_eq!(f16_bits_to_f32(0x0000), 0.0);
    assert_eq!(f16_bits_to_f32(0x3c00), 1.0);
    assert_eq!(f16_bits_to_f32(0xbc00), -1.0);
    assert!(f16_bits_to_f32(0x7c00).is_infinite());
    assert!(f16_bits_to_f32(0x7e00).is_nan());
}
