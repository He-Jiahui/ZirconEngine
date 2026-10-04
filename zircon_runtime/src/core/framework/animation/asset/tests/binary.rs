use super::{
    decode_binary_asset, encode_binary_asset, validate_binary_input_len, AnimationBinaryAssetKind,
    ANIMATION_BINARY_MAX_DECODE_BYTES,
};
use crate::core::framework::animation::AnimationAssetError;

#[test]
fn animation_binary_rejects_oversized_input_before_deserialization() {
    let error = validate_binary_input_len(
        AnimationBinaryAssetKind::Graph,
        ANIMATION_BINARY_MAX_DECODE_BYTES + 1,
    )
    .expect_err("oversized input must be rejected before deserialize");

    assert!(matches!(
        error,
        AnimationAssetError::InputTooLarge {
            kind: "graph",
            actual_bytes,
            limit_bytes,
        } if actual_bytes == limit_bytes + 1
    ));
}

#[test]
fn animation_binary_budgeting_preserves_legacy_trailing_byte_decoding() {
    let mut bytes = encode_binary_asset(AnimationBinaryAssetKind::Graph, &7_u8)
        .expect("fixture serialization succeeds");
    bytes.push(0);

    let decoded = decode_binary_asset::<u8>(AnimationBinaryAssetKind::Graph, &bytes)
        .expect("legacy trailing bytes remain accepted");

    assert_eq!(decoded, 7);
}
