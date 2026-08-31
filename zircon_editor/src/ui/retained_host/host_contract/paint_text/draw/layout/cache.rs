use std::sync::{Arc, Mutex, OnceLock};

use indexmap::{Equivalent, IndexMap};

use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::HostTextSmoothing;
use super::super::super::font::{host_font_snapshot_for_face, HostTextFontFace};
use super::super::super::layout_policy::HostTextLayoutPolicy;
use super::super::super::sync::lock_recovering_poison;
use super::PaintTextLayout;

const TEXT_LAYOUT_CACHE_CAPACITY: usize = 2_048;

#[derive(Debug, Eq, Hash, PartialEq)]
struct PaintTextLayoutCacheKey {
    text: String,
    properties: PaintTextLayoutCacheProperties,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct PaintTextLayoutCacheProperties {
    rect_x_bits: u32,
    rect_y_bits: u32,
    rect_width_bits: u32,
    rect_height_bits: u32,
    font_size_bits: u32,
    line_height_bits: u32,
    font_face: HostTextFontFace,
    font_cache_key: u64,
    runtime_font_generation: u64,
    smoothing: HostTextSmoothing,
    word_wrap: bool,
}

#[derive(Clone, Copy, Debug, Hash)]
struct PaintTextLayoutCacheLookup<'a> {
    text: &'a str,
    properties: PaintTextLayoutCacheProperties,
}

impl Equivalent<PaintTextLayoutCacheKey> for PaintTextLayoutCacheLookup<'_> {
    fn equivalent(&self, key: &PaintTextLayoutCacheKey) -> bool {
        self.text == key.text.as_str() && self.properties == key.properties
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
    build: impl FnOnce() -> PaintTextLayout,
) -> Arc<PaintTextLayout> {
    static CACHE: OnceLock<Mutex<IndexMap<PaintTextLayoutCacheKey, Arc<PaintTextLayout>>>> =
        OnceLock::new();

    let properties = PaintTextLayoutCacheProperties {
        rect_x_bits: rect.x.to_bits(),
        rect_y_bits: rect.y.to_bits(),
        rect_width_bits: rect.width.to_bits(),
        rect_height_bits: rect.height.to_bits(),
        font_size_bits: font_size.to_bits(),
        line_height_bits: line_height.to_bits(),
        font_face,
        font_cache_key: host_font_snapshot_for_face(font_face).cache_key(),
        runtime_font_generation: zircon_runtime::ui::surface::current_resolved_text_font_generation(
        ),
        smoothing,
        word_wrap: layout_policy == HostTextLayoutPolicy::WordWrap,
    };
    let lookup = PaintTextLayoutCacheLookup { text, properties };
    let cache = CACHE.get_or_init(|| Mutex::new(IndexMap::new()));
    if let Some(layout) = lock_recovering_poison(cache).get(&lookup).cloned() {
        return layout;
    }

    let layout = Arc::new(build());
    let mut cache = lock_recovering_poison(cache);
    if let Some(existing) = cache.get(&lookup).cloned() {
        return existing;
    }
    if cache.len() >= TEXT_LAYOUT_CACHE_CAPACITY {
        let _ = cache.swap_remove_index(0);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn cache_properties(rect_x_bits: u32) -> PaintTextLayoutCacheProperties {
        PaintTextLayoutCacheProperties {
            rect_x_bits,
            rect_y_bits: 0,
            rect_width_bits: 100.0_f32.to_bits(),
            rect_height_bits: 20.0_f32.to_bits(),
            font_size_bits: 13.0_f32.to_bits(),
            line_height_bits: 16.0_f32.to_bits(),
            font_face: HostTextFontFace::Ui,
            font_cache_key: 1,
            runtime_font_generation: 1,
            smoothing: HostTextSmoothing::Grayscale,
            word_wrap: false,
        }
    }

    fn layout(text: &str) -> Arc<PaintTextLayout> {
        Arc::new(PaintTextLayout {
            display_text: text.to_string(),
            font_face: HostTextFontFace::Ui,
            glyphs: Vec::new(),
            artifact_raster_fonts: Vec::new(),
        })
    }

    #[test]
    fn borrowed_text_lookup_reuses_the_inserted_layout() {
        let mut cache = IndexMap::new();
        let properties = cache_properties(0);
        let expected = layout("Preview");
        cache.insert(
            PaintTextLayoutCacheKey {
                text: "Preview".to_string(),
                properties,
            },
            Arc::clone(&expected),
        );

        let actual = cache
            .get(&PaintTextLayoutCacheLookup {
                text: "Preview",
                properties,
            })
            .expect("cached layout");

        assert!(Arc::ptr_eq(actual, &expected));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn eviction_removes_one_layout_instead_of_clearing_the_cache() {
        let mut cache = IndexMap::new();
        cache.insert(
            PaintTextLayoutCacheKey {
                text: "First".to_string(),
                properties: cache_properties(0),
            },
            layout("First"),
        );
        cache.insert(
            PaintTextLayoutCacheKey {
                text: "Second".to_string(),
                properties: cache_properties(1),
            },
            layout("Second"),
        );

        let _ = cache.swap_remove_index(0);

        assert_eq!(cache.len(), 1);
    }
}
