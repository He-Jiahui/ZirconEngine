pub(super) fn decode_optional_u32(value: u32) -> Option<u32> {
    (value != u32::MAX).then_some(value)
}
