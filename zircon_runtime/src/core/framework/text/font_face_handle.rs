use super::TextFontCollectionHandle;

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
