use super::super::{GlyphAtlasPageKey, GlyphAtlasRect};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapPageShadowPatch {
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) page_generation: u64,
    pub(crate) target_rect: GlyphAtlasRect,
    pub(crate) bytes_per_row: u32,
    pub(crate) bytes: Vec<u8>,
}
