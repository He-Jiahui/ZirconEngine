use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
/// 字形栅格服务返回的错误类别；diagnostic_code 供日志归类，非穷尽标记要求调用方为后续变体保留处理分支。
pub enum TextGlyphRasterError {
    InvalidGlyphId,
    InvalidPhysicalPpem,
    InvalidSubpixelPhase,
    MissingFontFace,
    MissingGlyph,
    InvalidBitmap,
    StaleFontGeneration,
    BudgetDeferred,
    BackendUnavailable,
}

impl TextGlyphRasterError {
    pub const fn diagnostic_code(self) -> &'static str {
        match self {
            Self::InvalidGlyphId => "ZR-TEXT-RASTER-001",
            Self::InvalidPhysicalPpem => "ZR-TEXT-RASTER-002",
            Self::InvalidSubpixelPhase => "ZR-TEXT-RASTER-003",
            Self::MissingFontFace => "ZR-TEXT-RASTER-004",
            Self::MissingGlyph => "ZR-TEXT-RASTER-005",
            Self::InvalidBitmap => "ZR-TEXT-RASTER-006",
            Self::StaleFontGeneration => "ZR-TEXT-RASTER-007",
            Self::BudgetDeferred => "ZR-TEXT-RASTER-008",
            Self::BackendUnavailable => "ZR-TEXT-RASTER-009",
        }
    }
}

impl Display for TextGlyphRasterError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidGlyphId => "text glyph id is outside the raster backend range",
            Self::InvalidPhysicalPpem => "text glyph physical ppem must be positive",
            Self::InvalidSubpixelPhase => {
                "text glyph subpixel phase is outside its quantized range"
            }
            Self::MissingFontFace => "text glyph font face is unavailable",
            Self::MissingGlyph => "text glyph has no raster image",
            Self::InvalidBitmap => "text glyph raster bitmap is invalid",
            Self::StaleFontGeneration => "text glyph font generation changed during rasterization",
            Self::BudgetDeferred => "text glyph rasterization was deferred by budget",
            Self::BackendUnavailable => "text glyph raster backend is unavailable",
        })
    }
}

impl Error for TextGlyphRasterError {}
