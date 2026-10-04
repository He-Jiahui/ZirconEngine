use super::TextFontCollectionHandle;

/// 中立字形引用的字体槽位，携带集合和代际以拒绝热重载后的陈旧槽位。
///
/// face 与 instance 分别由字体注册表解析，不能仅凭 index 当作后端字体 ID。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextFontFaceHandle {
    pub collection: TextFontCollectionHandle,
    pub index: u32,
    pub generation: u64,
}

impl TextFontFaceHandle {
    pub const fn new(collection: TextFontCollectionHandle, index: u32, generation: u64) -> Self {
        Self {
            collection,
            index,
            generation,
        }
    }
}
