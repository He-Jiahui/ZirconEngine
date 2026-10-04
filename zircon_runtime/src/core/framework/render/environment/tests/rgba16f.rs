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
