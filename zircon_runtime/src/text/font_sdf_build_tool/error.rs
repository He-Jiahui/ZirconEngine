//! 构建工具的对外错误边界，区分字体解码、字型面提取、字符选择、页面容量与产物构建失败；CLI 可报告失败阶段而无需理解 runtime 内部错误。

//! Typed failures produced by offline font distance-field tooling.

use thiserror::Error;

#[derive(Debug, Error)]
/// 公开构建和检查 API 的失败契约；缺失字符可在 bake 阶段跳过，但最终无可见字形会成为 NoGeneratedGlyphs。
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
