#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::text::atlas) struct GlyphAtlasBitmapPageShadow {
    pub(super) generation: u64,
    pub(super) bytes: Vec<u8>,
}
