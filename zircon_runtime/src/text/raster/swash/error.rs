//! 保留后端输入错误、缺字形与非法位图的不同失败语义。
//! 服务适配层把这些错误映射到共享文字错误，异步原生图集路径则记录失败并由后续帧重新请求。

use super::bitmap::GlyphBitmapError;
use super::request::SwashRasterSource;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SwashRasterError {
    InvalidFontFace {
        face_index: usize,
    },
    InvalidPxSize,
    InvalidOffset,
    InvalidVariationCoordinate,
    MissingGlyphImage {
        glyph_id: u16,
        source: SwashRasterSource,
    },
    InvalidGlyphBitmap(GlyphBitmapError),
}
