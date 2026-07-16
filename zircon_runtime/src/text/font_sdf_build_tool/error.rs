//! Typed failures produced by offline font distance-field tooling.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FontSdfBakeError {
    #[error("decode font source: {0}")]
    DecodeFont(String),
    #[error("extract font face {face_index}: {message}")]
    ExtractFace { face_index: u32, message: String },
    #[error("parse standalone font face: {0}")]
    ParseFace(String),
    #[error("font-SDF glyph selection is empty")]
    EmptySelection,
    #[error("font-SDF selection contains invalid Unicode scalar U+{0:04X}")]
    InvalidCodepoint(u32),
    #[error("font-SDF selection contains no mapped glyphs")]
    NoMappedGlyphs,
    #[error("font-SDF generation produced no visible glyphs; skipped {skipped_count}")]
    NoGeneratedGlyphs { skipped_count: usize },
    #[error("font-SDF glyph {glyph_id} size {width}x{height} exceeds page size {page_size}")]
    GlyphExceedsPage {
        glyph_id: u32,
        width: u32,
        height: u32,
        page_size: u32,
    },
    #[error("font-SDF atlas size arithmetic overflowed")]
    AtlasSizeOverflow,
    #[error("build `.zsdf` artifact: {0}")]
    Artifact(String),
}
