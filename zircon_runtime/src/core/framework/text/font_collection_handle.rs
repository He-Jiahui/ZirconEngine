/// 字体句柄所属的集合身份；解析字形时必须与当前集合快照配对。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextFontCollectionHandle(u64);

impl TextFontCollectionHandle {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}
