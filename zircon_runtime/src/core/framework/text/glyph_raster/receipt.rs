use std::sync::Arc;

use super::{TextGlyphBitmapFormat, TextGlyphRasterRequest};
use crate::core::framework::text::{TextFontCollectionHandle, TextFontFaceHandle};

#[derive(Clone, Debug, PartialEq)]
/// 一次栅格结果及其字体来源快照；共享位图与请求、格式、尺寸和 bearing 一起交给 atlas/UI 消费者。
/// font_generation 与 source_identity 用于让消费者发现缓存结果是否仍对应当前字体来源。
pub struct TextGlyphRasterReceipt {
    pub font_collection: TextFontCollectionHandle,
    pub font_face: TextFontFaceHandle,
    pub font_instance: Option<TextFontFaceHandle>,
    pub font_generation: u64,
    pub source_identity: [u8; 16],
    pub request: TextGlyphRasterRequest,
    pub format: TextGlyphBitmapFormat,
    pub size: [u32; 2],
    pub bearing: [f32; 2],
    pub bitmap: Arc<[u8]>,
}
