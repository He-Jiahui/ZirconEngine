use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum SdfOfflineArtifactError {
    #[error("invalid canonical asset GUID `{0}`")]
    InvalidAssetGuid(String),
    #[error("offline SDF atlas page size must be non-zero")]
    InvalidPageSize,
    #[error("offline SDF artifact must contain at least one page")]
    MissingPages,
    #[error("offline SDF page index {0} is duplicated")]
    DuplicatePageIndex(u32),
    #[error("offline SDF pages must be contiguous; expected {expected}, found {actual}")]
    NonContiguousPageIndex { expected: u32, actual: u32 },
    #[error("offline SDF page {page_index} has {actual} bytes; expected {expected}")]
    InvalidPageByteLength {
        page_index: u32,
        expected: usize,
        actual: usize,
    },
    #[error("offline SDF glyph id {0} is duplicated")]
    DuplicateGlyphId(u32),
    #[error("offline SDF glyph {glyph_id} contains invalid Unicode scalar U+{codepoint:04X}")]
    InvalidCodepoint { glyph_id: u32, codepoint: u32 },
    #[error("offline SDF glyph {glyph_id} references missing page {page_index}")]
    MissingGlyphPage { glyph_id: u32, page_index: u32 },
    #[error("offline SDF glyph {glyph_id} rectangle is empty or outside page {page_index}")]
    GlyphRectOutOfBounds { glyph_id: u32, page_index: u32 },
    #[error("offline SDF glyph {glyph_id} contains a non-finite metric")]
    NonFiniteGlyphMetric { glyph_id: u32 },
    #[error("offline SDF encoded length overflow")]
    LengthOverflow,
    #[error("offline SDF input ended before the declared section completed")]
    UnexpectedEof,
    #[error("offline SDF magic does not match")]
    InvalidMagic,
    #[error("offline SDF version {found} is unsupported; expected {supported}")]
    UnsupportedVersion { found: u32, supported: u32 },
    #[error("offline SDF header length {0} is invalid")]
    InvalidHeaderLength(u32),
    #[error("offline SDF header contains non-zero reserved bits")]
    NonZeroReserved,
    #[error("offline SDF mode discriminant {0} is invalid")]
    InvalidMode(u32),
    #[error("offline SDF section lengths do not match their record counts")]
    InvalidSectionLength,
    #[error("offline SDF page payload offsets are not tightly packed")]
    NonContiguousPagePayload,
    #[error("offline SDF input contains trailing bytes")]
    TrailingBytes,
    #[error("offline SDF checksum does not match")]
    ChecksumMismatch,
    #[error("offline SDF identity field `{field}` is stale")]
    IdentityMismatch { field: &'static str },
}
