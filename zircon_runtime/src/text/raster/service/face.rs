use crate::core::framework::text::{TextFontCollectionHandle, TextFontFaceHandle};
use crate::text::VariationCoords;

pub(crate) struct RuntimeGlyphRasterFace<'a> {
    pub(crate) font_collection: TextFontCollectionHandle,
    pub(crate) font_face: TextFontFaceHandle,
    pub(crate) font_instance: Option<TextFontFaceHandle>,
    pub(crate) font_generation: u64,
    pub(crate) source_identity: [u8; 16],
    pub(crate) bytes: &'a [u8],
    pub(crate) collection_index: usize,
    pub(crate) variations: Option<&'a VariationCoords>,
}
