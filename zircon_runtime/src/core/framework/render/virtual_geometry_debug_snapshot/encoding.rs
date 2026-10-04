// GPU 词布局以 u32::MAX 编码 Option::None；这里只解哨兵，窄类型字段仍需调用方验证范围。
pub(super) fn decode_optional_u32(value: u32) -> Option<u32> {
    (value != u32::MAX).then_some(value)
}
