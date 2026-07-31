#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct GlyphAtlasBitmapPageShadow {
    pub(super) generation: u64,
    pub(super) bytes: Vec<u8>,
}
