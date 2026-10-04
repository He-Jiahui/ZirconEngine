use super::*;

#[test]
fn byte_and_usize_helpers_match_abi_bounds() {
    let payload = byte_slice("abc");
    assert_eq!(payload.len, 3);
    assert!(!payload.data.is_null());
    assert_eq!(usize_to_u32(7), 7);
    assert_eq!(usize_to_u32(usize::MAX), u32::MAX - 1);
}
