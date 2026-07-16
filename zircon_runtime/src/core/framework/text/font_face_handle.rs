#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextFontFaceHandle {
    pub index: u32,
    pub generation: u64,
}

impl TextFontFaceHandle {
    pub const fn new(index: u32, generation: u64) -> Self {
        Self { index, generation }
    }
}
