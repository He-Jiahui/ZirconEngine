use std::sync::Arc;

use super::super::{GlyphAtlasPageKey, GlyphAtlasRect};

#[derive(Clone, Debug, PartialEq, Eq)]
/// 一次已接受上传的紧凑区域副本，供持久阴影重放。
/// 页键和世代共同标识目标；复用页键后不能把旧世代的像素写入新页面。
pub(crate) struct GlyphAtlasBitmapPageShadowPatch {
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) page_generation: u64,
    pub(crate) target_rect: GlyphAtlasRect,
    pub(crate) bytes_per_row: u32,
    pub(crate) bytes: Arc<[u8]>,
}
