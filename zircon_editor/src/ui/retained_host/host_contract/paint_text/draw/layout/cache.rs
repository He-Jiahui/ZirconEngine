use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::HostTextSmoothing;
use super::super::super::font::{font_cache_key_for_face, HostTextFontFace};
use super::super::super::layout_policy::HostTextLayoutPolicy;
use super::super::super::sync::lock_recovering_poison;
use super::PaintTextLayout;

const TEXT_LAYOUT_CACHE_CAPACITY: usize = 2_048;

#[derive(Debug, Eq, Hash, PartialEq)]
struct PaintTextLayoutCacheKey {
    text: String,
    rect_x_bits: u32,
    rect_y_bits: u32,
    rect_width_bits: u32,
    rect_height_bits: u32,
    font_size_bits: u32,
    line_height_bits: u32,
    font_face: HostTextFontFace,
    font_cache_key: u64,
    smoothing: HostTextSmoothing,
    word_wrap: bool,
}

pub(super) fn cached_paint_text_layout(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
    font_face: HostTextFontFace,
    smoothing: HostTextSmoothing,
    layout_policy: HostTextLayoutPolicy,
    build: impl FnOnce() -> PaintTextLayout,
) -> Arc<PaintTextLayout> {
    static CACHE: OnceLock<Mutex<HashMap<PaintTextLayoutCacheKey, Arc<PaintTextLayout>>>> =
        OnceLock::new();

    let key = PaintTextLayoutCacheKey {
        text: text.to_string(),
        rect_x_bits: rect.x.to_bits(),
        rect_y_bits: rect.y.to_bits(),
        rect_width_bits: rect.width.to_bits(),
        rect_height_bits: rect.height.to_bits(),
        font_size_bits: font_size.to_bits(),
        line_height_bits: line_height.to_bits(),
        font_face,
        font_cache_key: font_cache_key_for_face(font_face),
        smoothing,
        word_wrap: layout_policy == HostTextLayoutPolicy::WordWrap,
    };
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(layout) = lock_recovering_poison(cache).get(&key).cloned() {
        return layout;
    }

    let layout = Arc::new(build());
    let mut cache = lock_recovering_poison(cache);
    if let Some(existing) = cache.get(&key).cloned() {
        return existing;
    }
    if cache.len() >= TEXT_LAYOUT_CACHE_CAPACITY {
        cache.clear();
    }
    cache.insert(key, Arc::clone(&layout));
    layout
}
