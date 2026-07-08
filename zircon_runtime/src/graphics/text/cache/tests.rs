use crate::core::framework::render::{
    ShapedGlyphRun, TextOrientation, TextShapeRequest, VerticalMode,
};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextOverflow, UiTextRange, UiTextWrap,
};

use super::{
    ShapedRunCache, ShapedRunCacheKey, TextFrameDedup, TextLayoutCache, TextLayoutWidthValidity,
    TextMeasureCache,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct MeasureKey {
    text_hash: u64,
    wrap_width_bits: u32,
}

impl MeasureKey {
    fn new(text_hash: u64, wrap_width: f32) -> Self {
        Self {
            text_hash,
            wrap_width_bits: wrap_width.to_bits(),
        }
    }
}

#[test]
fn text_measure_cache_rejects_hash_collision_without_text_match() {
    let key = MeasureKey::new(7, 120.0);
    let mut cache = TextMeasureCache::with_capacity(4);
    cache.begin_frame(1);

    cache.insert(key.clone(), "Alpha", 42_u32);

    assert!(cache.get(&key, "Beta").is_none());
    assert_eq!(cache.get(&key, "Alpha"), Some(&42));

    let report = cache.report();
    assert_eq!(report.collision_miss_count, 1);
    assert_eq!(report.miss_count, 1);
    assert_eq!(report.hit_count, 1);
}

#[test]
fn text_measure_cache_lru_trims_only_when_capacity_is_exceeded() {
    let key_a = MeasureKey::new(1, 80.0);
    let key_b = MeasureKey::new(2, 80.0);
    let key_c = MeasureKey::new(3, 80.0);
    let mut cache = TextMeasureCache::with_capacity(2);

    cache.begin_frame(1);
    cache.insert(key_a.clone(), "A", 1_u32);
    cache.insert(key_b.clone(), "B", 2_u32);

    cache.begin_frame(2);
    assert_eq!(cache.get(&key_a, "A"), Some(&1));

    cache.begin_frame(3);
    cache.insert(key_c.clone(), "C", 3_u32);

    assert!(cache.contains_exact(&key_a, "A"));
    assert!(!cache.contains_exact(&key_b, "B"));
    assert!(cache.contains_exact(&key_c, "C"));
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.report().evicted_count, 1);
}

#[test]
fn text_measure_cache_frame_end_does_not_clear_unreferenced_entries() {
    let key = MeasureKey::new(99, 240.0);
    let mut cache = TextMeasureCache::with_capacity(4);

    cache.begin_frame(1);
    cache.insert(key.clone(), "folder-open.svg", 96_u32);

    cache.begin_frame(2);
    cache.finish_frame();

    assert!(cache.contains_exact(&key, "folder-open.svg"));
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.report().evicted_count, 0);
}

#[test]
fn text_measure_cache_get_or_insert_measures_only_on_miss() {
    let key = MeasureKey::new(5, 160.0);
    let mut measure_count = 0_u32;
    let mut cache = TextMeasureCache::with_capacity(2);
    cache.begin_frame(1);

    let (first, inserted_first) = cache.get_or_insert_with(key.clone(), "asset", || {
        measure_count += 1;
        32_u32
    });
    assert_eq!(*first, 32);
    assert!(inserted_first);

    let (second, inserted_second) = cache.get_or_insert_with(key, "asset", || {
        measure_count += 1;
        64_u32
    });

    assert_eq!(*second, 32);
    assert!(!inserted_second);
    assert_eq!(measure_count, 1);
    assert_eq!(cache.report().hit_count, 1);
    assert_eq!(cache.report().miss_count, 1);
}

#[test]
fn text_layout_cache_hits_exact_wrap_width_only() {
    let key = MeasureKey::new(11, 80.0);
    let mut cache = TextLayoutCache::with_capacity(4);
    cache.begin_frame(1);

    cache.insert(
        key.clone(),
        "Alpha Beta",
        TextLayoutWidthValidity::exact(80.0),
        1_u32,
    );

    assert_eq!(cache.get(&key, "Alpha Beta", 80.0), Some(&1));
    assert!(cache.get(&key, "Alpha Beta", 96.0).is_none());
    assert_eq!(cache.report().width_miss_count, 1);
}

#[test]
fn text_layout_cache_hits_valid_width_range() {
    let key = MeasureKey::new(12, 100.0);
    let mut cache = TextLayoutCache::with_capacity(4);
    cache.begin_frame(1);

    cache.insert(
        key.clone(),
        "folder-open.svg",
        TextLayoutWidthValidity::range(96.0, 128.0),
        7_u32,
    );

    assert_eq!(cache.get(&key, "folder-open.svg", 96.0), Some(&7));
    assert_eq!(cache.get(&key, "folder-open.svg", 127.5), Some(&7));
    assert!(cache.get(&key, "folder-open.svg", 128.0).is_none());
}

#[test]
fn text_layout_cache_rejects_hash_collision_without_text_match() {
    let key = MeasureKey::new(13, 80.0);
    let mut cache = TextLayoutCache::with_capacity(4);
    cache.begin_frame(1);

    cache.insert(
        key.clone(),
        "editor base.zui",
        TextLayoutWidthValidity::exact(80.0),
        3_u32,
    );

    assert!(cache.get(&key, "other base.zui", 80.0).is_none());
    assert_eq!(cache.report().collision_miss_count, 1);
}

#[test]
fn text_layout_cache_lru_trims_only_when_capacity_is_exceeded() {
    let key_a = MeasureKey::new(21, 80.0);
    let key_b = MeasureKey::new(22, 80.0);
    let key_c = MeasureKey::new(23, 80.0);
    let width = TextLayoutWidthValidity::exact(80.0);
    let mut cache = TextLayoutCache::with_capacity(2);

    cache.begin_frame(1);
    cache.insert(key_a.clone(), "A", width, 1_u32);
    cache.insert(key_b.clone(), "B", width, 2_u32);

    cache.begin_frame(2);
    assert_eq!(cache.get(&key_a, "A", 80.0), Some(&1));

    cache.begin_frame(3);
    cache.insert(key_c.clone(), "C", width, 3_u32);

    assert!(cache.contains_exact(&key_a, "A", width));
    assert!(!cache.contains_exact(&key_b, "B", width));
    assert!(cache.contains_exact(&key_c, "C", width));
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.report().evicted_count, 1);
}

#[test]
fn text_layout_cache_frame_end_does_not_clear_unreferenced_entries() {
    let key = MeasureKey::new(31, 240.0);
    let width = TextLayoutWidthValidity::exact(240.0);
    let mut cache = TextLayoutCache::with_capacity(4);

    cache.begin_frame(1);
    cache.insert(key.clone(), "asset", width, 96_u32);

    cache.begin_frame(2);
    cache.finish_frame();

    assert!(cache.contains_exact(&key, "asset", width));
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.report().evicted_count, 0);
}

#[test]
fn text_layout_cache_get_or_insert_lays_out_only_on_miss() {
    let key = MeasureKey::new(41, 100.0);
    let width = TextLayoutWidthValidity::range(90.0, 120.0);
    let mut layout_count = 0_u32;
    let mut cache = TextLayoutCache::with_capacity(2);
    cache.begin_frame(1);

    let (first, inserted_first) =
        cache.get_or_insert_with(key.clone(), "label", width, 100.0, || {
            layout_count += 1;
            32_u32
        });
    assert_eq!(*first, 32);
    assert!(inserted_first);

    let (second, inserted_second) = cache.get_or_insert_with(key, "label", width, 110.0, || {
        layout_count += 1;
        64_u32
    });

    assert_eq!(*second, 32);
    assert!(!inserted_second);
    assert_eq!(layout_count, 1);
    assert_eq!(cache.report().hit_count, 1);
    assert_eq!(cache.report().miss_count, 1);
}

#[test]
fn text_frame_dedup_shares_value_inside_one_frame() {
    let key = MeasureKey::new(51, 80.0);
    let mut produce_count = 0_u32;
    let mut dedup = TextFrameDedup::default();
    dedup.begin_frame(1);

    let (first, inserted_first) = dedup.get_or_insert_with(key.clone(), "label", || {
        produce_count += 1;
        10_u32
    });
    assert_eq!(*first, 10);
    assert!(inserted_first);

    let (second, inserted_second) = dedup.get_or_insert_with(key, "label", || {
        produce_count += 1;
        20_u32
    });

    assert_eq!(*second, 10);
    assert!(!inserted_second);
    assert_eq!(produce_count, 1);
    assert_eq!(dedup.report().hit_count, 1);
    assert_eq!(dedup.report().miss_count, 1);
}

#[test]
fn text_frame_dedup_resets_between_frames() {
    let key = MeasureKey::new(52, 80.0);
    let mut dedup = TextFrameDedup::default();
    dedup.begin_frame(1);

    dedup.insert(key.clone(), "label", 10_u32);
    assert_eq!(dedup.get(&key, "label"), Some(&10));

    dedup.begin_frame(2);

    assert!(dedup.get(&key, "label").is_none());
    assert_eq!(dedup.len(), 0);
    assert_eq!(dedup.report().frame_index, 2);
}

#[test]
fn text_frame_dedup_rejects_hash_collision_without_text_match() {
    let key = MeasureKey::new(53, 80.0);
    let mut dedup = TextFrameDedup::default();
    dedup.begin_frame(1);

    dedup.insert(key.clone(), "Alpha", 10_u32);

    assert!(dedup.get(&key, "Beta").is_none());
    assert_eq!(dedup.report().collision_miss_count, 1);
}

#[test]
fn text_frame_dedup_updates_exact_entry() {
    let key = MeasureKey::new(54, 80.0);
    let mut dedup = TextFrameDedup::default();
    dedup.begin_frame(1);

    dedup.insert(key.clone(), "label", 10_u32);
    dedup.insert(key.clone(), "label", 20_u32);

    assert_eq!(dedup.get(&key, "label"), Some(&20));
    assert_eq!(dedup.len(), 1);
    assert_eq!(dedup.report().update_count, 1);
}

#[test]
fn shaped_run_cache_key_omits_wrap_alignment_and_overflow() {
    let mut style = UiResolvedStyle {
        font_family: Some("DengXian".to_string()),
        font_size: 13.0,
        wrap: UiTextWrap::None,
        text_align: UiTextAlign::Left,
        text_overflow: UiTextOverflow::Clip,
        ..UiResolvedStyle::default()
    };
    let first = key_for("editor base.zui", &style);

    style.wrap = UiTextWrap::Glyph;
    style.text_align = UiTextAlign::Right;
    style.text_overflow = UiTextOverflow::EllipsisMiddle;
    let second = key_for("editor base.zui", &style);

    assert_eq!(first, second);

    style.font_size = 14.0;
    assert_ne!(first, key_for("editor base.zui", &style));
}

#[test]
fn shaped_run_cache_rejects_text_hash_collision_without_text_match() {
    let style = UiResolvedStyle::default();
    let key = key_for("abc", &style);
    let mut cache = ShapedRunCache::with_capacity(4);
    cache.begin_frame(1);

    cache.insert(key.clone(), "abc", dummy_run("abc", 12.0));

    assert!(cache.get(&key, "xyz").is_none());
    assert!(cache.get(&key, "abc").is_some());

    let report = cache.report();
    assert_eq!(report.collision_miss_count, 1);
    assert_eq!(report.miss_count, 1);
    assert_eq!(report.hit_count, 1);
}

#[test]
fn shaped_run_cache_keeps_colliding_texts_as_distinct_entries() {
    let style = UiResolvedStyle::default();
    let key = key_for("abc", &style);
    let mut cache = ShapedRunCache::with_capacity(4);
    cache.begin_frame(1);

    cache.insert(key.clone(), "abc", dummy_run("abc", 12.0));
    cache.insert(key.clone(), "xyz", dummy_run("xyz", 21.0));

    let abc = cache
        .get(&key, "abc")
        .expect("abc run should remain cached");
    let xyz = cache
        .get(&key, "xyz")
        .expect("xyz run should remain cached");

    assert_eq!(abc.measured_width, 12.0);
    assert_eq!(xyz.measured_width, 21.0);
    assert_eq!(cache.len(), 2);
}

#[test]
fn shaped_run_cache_trims_lru_only_when_capacity_is_exceeded() {
    let style = UiResolvedStyle::default();
    let key_a = key_for("a", &style);
    let key_b = key_for("b", &style);
    let key_c = key_for("c", &style);
    let mut cache = ShapedRunCache::with_capacity(2);

    cache.begin_frame(1);
    cache.insert(key_a.clone(), "a", dummy_run("a", 1.0));
    cache.insert(key_b.clone(), "b", dummy_run("b", 2.0));

    cache.begin_frame(2);
    assert!(cache.get(&key_a, "a").is_some());

    cache.begin_frame(3);
    cache.insert(key_c.clone(), "c", dummy_run("c", 3.0));

    assert!(cache.contains_exact(&key_a, "a"));
    assert!(!cache.contains_exact(&key_b, "b"));
    assert!(cache.contains_exact(&key_c, "c"));
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.report().evicted_count, 1);
}

#[test]
fn shaped_run_cache_frame_end_does_not_clear_unreferenced_entries() {
    let style = UiResolvedStyle::default();
    let key = key_for("folder-open.svg", &style);
    let mut cache = ShapedRunCache::with_capacity(4);

    cache.begin_frame(1);
    cache.insert(
        key.clone(),
        "folder-open.svg",
        dummy_run("folder-open.svg", 96.0),
    );

    cache.begin_frame(2);
    cache.finish_frame();

    assert!(cache.contains_exact(&key, "folder-open.svg"));
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.report().evicted_count, 0);
}

#[test]
fn shaped_run_cache_get_or_insert_shapes_only_on_miss() {
    let style = UiResolvedStyle::default();
    let key = key_for("asset", &style);
    let mut shape_count = 0_u32;
    let mut cache = ShapedRunCache::with_capacity(2);
    cache.begin_frame(1);

    let first = cache.get_or_insert_with(key.clone(), "asset", || {
        shape_count += 1;
        dummy_run("asset", 32.0)
    });
    let second = cache.get_or_insert_with(key, "asset", || {
        shape_count += 1;
        dummy_run("asset", 64.0)
    });

    assert_eq!(shape_count, 1);
    assert_eq!(first.measured_width, 32.0);
    assert_eq!(second.measured_width, 32.0);
    assert_eq!(cache.report().hit_count, 1);
    assert_eq!(cache.report().miss_count, 1);
}

fn key_for(text: &str, style: &UiResolvedStyle) -> ShapedRunCacheKey {
    ShapedRunCacheKey::from_request(&TextShapeRequest::horizontal(
        text,
        style,
        UiTextDirection::LeftToRight,
        source_range_for(text),
    ))
}

fn dummy_run(text: &str, measured_width: f32) -> ShapedGlyphRun {
    ShapedGlyphRun {
        source_text: text.to_string(),
        source_range: source_range_for(text),
        direction: UiTextDirection::LeftToRight,
        orientation: TextOrientation::Horizontal,
        vertical_mode: VerticalMode::Mixed,
        include_kerning: true,
        measured_width,
        measured_height: 16.0,
        lines: Vec::new(),
    }
}

fn source_range_for(text: &str) -> UiTextRange {
    UiTextRange {
        start: 0,
        end: text.len(),
    }
}
