use std::sync::{Arc, Mutex, OnceLock};

use indexmap::{Equivalent, IndexMap};
use zircon_runtime::ui::surface::current_resolved_text_font_generation;

use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::HostTextSmoothing;
use super::super::super::font::{font_request_for_face, HostTextFontFace, HostTextFontRequest};
use super::super::super::layout_policy::HostTextLayoutPolicy;
use super::super::super::sync::lock_recovering_poison;
use super::PaintTextLayout;

const TEXT_LAYOUT_CACHE_CAPACITY: usize = 2_048;

#[derive(Debug, Eq, Hash, PartialEq)]
struct PaintTextLayoutCacheKey {
    text: String,
    properties: PaintTextLayoutCacheProperties,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PaintTextLayoutCacheProperties {
    rect_width_bits: u32,
    rect_height_bits: u32,
    font_size_bits: u32,
    line_height_bits: u32,
    font_collection_generation: u64,
    font_request: HostTextFontRequest,
    smoothing: HostTextSmoothing,
    word_wrap: bool,
    display_only: bool,
}

#[derive(Clone, Copy, Debug, Hash)]
struct PaintTextLayoutCacheLookup<'a> {
    text: &'a str,
    properties: &'a PaintTextLayoutCacheProperties,
}

impl Equivalent<PaintTextLayoutCacheKey> for PaintTextLayoutCacheLookup<'_> {
    fn equivalent(&self, key: &PaintTextLayoutCacheKey) -> bool {
        self.text == key.text.as_str() && self.properties == &key.properties
    }
}

pub(super) fn cached_paint_text_layout(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
    font_face: HostTextFontFace,
    smoothing: HostTextSmoothing,
    layout_policy: HostTextLayoutPolicy,
    display_only: bool,
    build: impl FnOnce(&FrameRect) -> PaintTextLayout,
) -> Arc<PaintTextLayout> {
    static CACHE: OnceLock<Mutex<IndexMap<PaintTextLayoutCacheKey, Arc<PaintTextLayout>>>> =
        OnceLock::new();

    let properties = PaintTextLayoutCacheProperties {
        rect_width_bits: rect.width.to_bits(),
        rect_height_bits: rect.height.to_bits(),
        font_size_bits: font_size.to_bits(),
        line_height_bits: line_height.to_bits(),
        font_collection_generation: current_resolved_text_font_generation(),
        font_request: font_request_for_face(font_face),
        smoothing,
        word_wrap: layout_policy == HostTextLayoutPolicy::WordWrap,
        display_only,
    };
    let lookup = PaintTextLayoutCacheLookup {
        text,
        properties: &properties,
    };
    let cache = CACHE.get_or_init(|| Mutex::new(IndexMap::new()));
    {
        let mut cache = lock_recovering_poison(cache);
        if let Some(layout) = touch_layout(&mut cache, &lookup) {
            return layout;
        }
    }

    let relative_rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: rect.width,
        height: rect.height,
    };
    let layout = Arc::new(build(&relative_rect));
    let mut cache = lock_recovering_poison(cache);
    if let Some(existing) = touch_layout(
        &mut cache,
        &PaintTextLayoutCacheLookup {
            text,
            properties: &properties,
        },
    ) {
        return existing;
    }
    if cache.len() >= TEXT_LAYOUT_CACHE_CAPACITY {
        let _ = cache.shift_remove_index(0);
    }
    cache.insert(
        PaintTextLayoutCacheKey {
            text: text.to_string(),
            properties,
        },
        Arc::clone(&layout),
    );
    layout
}

fn touch_layout(
    cache: &mut IndexMap<PaintTextLayoutCacheKey, Arc<PaintTextLayout>>,
    lookup: &PaintTextLayoutCacheLookup<'_>,
) -> Option<Arc<PaintTextLayout>> {
    let (index, _, value) = cache.get_full(lookup)?;
    let value = Arc::clone(value);
    let last = cache.len() - 1;
    cache.move_index(index, last);
    Some(value)
}

#[cfg(test)]
#[path = "tests/cache.rs"]
mod tests;
