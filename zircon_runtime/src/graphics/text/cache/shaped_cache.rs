use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    mem::size_of,
    sync::Arc,
};

use crate::core::framework::render::{
    ShapedGlyph, ShapedGlyphRun, ShapedTextLine, TextOrientation, TextShapeRequest, VerticalMode,
};
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

pub(crate) const DEFAULT_SHAPED_RUN_CACHE_CAPACITY: usize = 1024;
pub(crate) const DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShapedRunCacheKey {
    pub(crate) text_hash: u64,
    pub(crate) source_range: UiTextRange,
    pub(crate) font_family: Option<String>,
    pub(crate) font_weight: u16,
    pub(crate) font_size_bits: u32,
    pub(crate) line_height_bits: u32,
    pub(crate) tab_size_bits: u32,
    pub(crate) base_direction: UiTextDirection,
    pub(crate) orientation: TextOrientation,
    pub(crate) vertical_mode: VerticalMode,
    pub(crate) features_hash: u64,
    pub(crate) language: Option<String>,
}

impl ShapedRunCacheKey {
    pub(crate) fn from_request(request: &TextShapeRequest<'_>) -> Self {
        let font_size = request.style.font_size.max(1.0);
        let line_height = request.style.line_height.max(font_size);

        Self {
            text_hash: hash_text(request.text),
            source_range: request.source_range,
            font_family: cache_font_family(request.style),
            font_weight: UiResolvedStyle::normalized_font_weight(request.style.font_weight),
            font_size_bits: normalized_f32_bits(font_size),
            line_height_bits: normalized_f32_bits(line_height),
            tab_size_bits: normalized_f32_bits(request.style.tab_size),
            base_direction: request.base_direction,
            orientation: request.orientation,
            vertical_mode: request.vertical_mode,
            features_hash: shaping_features_hash(request.include_kerning),
            language: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShapedRunCacheReport {
    pub(crate) frame_index: u64,
    pub(crate) capacity: usize,
    pub(crate) max_bytes: usize,
    pub(crate) entry_count: usize,
    pub(crate) estimated_bytes: usize,
    pub(crate) hit_count: u64,
    pub(crate) miss_count: u64,
    pub(crate) collision_miss_count: u64,
    pub(crate) insert_count: u64,
    pub(crate) update_count: u64,
    pub(crate) evicted_count: u64,
    pub(crate) trim_count: u64,
    pub(crate) clear_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct ShapedRunCacheEntry {
    key: ShapedRunCacheKey,
    text: Arc<str>,
    run: Arc<ShapedGlyphRun>,
    estimated_bytes: usize,
    last_used_frame: u64,
    touch_order: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ShapedRunCache {
    entries: Vec<ShapedRunCacheEntry>,
    capacity: usize,
    max_bytes: usize,
    estimated_bytes: usize,
    current_frame: u64,
    touch_order: u64,
    frame_report: ShapedRunCacheReport,
}

impl Default for ShapedRunCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapedRunCache {
    pub(crate) fn new() -> Self {
        Self::with_limits(
            DEFAULT_SHAPED_RUN_CACHE_CAPACITY,
            DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES,
        )
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self::with_limits(capacity, DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES)
    }

    pub(crate) fn with_limits(capacity: usize, max_bytes: usize) -> Self {
        let mut cache = Self {
            entries: Vec::new(),
            capacity,
            max_bytes,
            estimated_bytes: 0,
            current_frame: 0,
            touch_order: 0,
            frame_report: ShapedRunCacheReport::default(),
        };
        cache.frame_report.capacity = capacity;
        cache.frame_report.max_bytes = max_bytes;
        cache
    }

    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.current_frame = frame_index;
        self.frame_report = ShapedRunCacheReport {
            frame_index,
            capacity: self.capacity,
            max_bytes: self.max_bytes,
            entry_count: self.entries.len(),
            estimated_bytes: self.estimated_bytes,
            ..ShapedRunCacheReport::default()
        };
    }

    pub(crate) fn finish_frame(&mut self) {
        self.trim_to_limits();
    }

    pub(crate) fn clear(&mut self) {
        self.frame_report.evicted_count = self
            .frame_report
            .evicted_count
            .saturating_add(self.entries.len() as u64);
        self.frame_report.clear_count = self.frame_report.clear_count.saturating_add(1);
        self.entries.clear();
        self.estimated_bytes = 0;
        self.refresh_report_size();
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn estimated_bytes(&self) -> usize {
        self.estimated_bytes
    }

    pub(crate) fn report(&self) -> ShapedRunCacheReport {
        let mut report = self.frame_report;
        report.entry_count = self.entries.len();
        report.estimated_bytes = self.estimated_bytes;
        report
    }

    pub(crate) fn contains_exact(&self, key: &ShapedRunCacheKey, text: &str) -> bool {
        self.entries
            .iter()
            .any(|entry| &entry.key == key && entry.text.as_ref() == text)
    }

    pub(crate) fn get(
        &mut self,
        key: &ShapedRunCacheKey,
        text: &str,
    ) -> Option<Arc<ShapedGlyphRun>> {
        let mut collision_seen = false;
        let mut hit_index = None;

        for (index, entry) in self.entries.iter().enumerate() {
            if &entry.key != key {
                continue;
            }
            if entry.text.as_ref() == text {
                hit_index = Some(index);
                break;
            }
            collision_seen = true;
        }

        let Some(index) = hit_index else {
            self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
            if collision_seen {
                self.frame_report.collision_miss_count =
                    self.frame_report.collision_miss_count.saturating_add(1);
            }
            return None;
        };

        let run = Arc::clone(&self.entries[index].run);
        self.touch_entry(index);
        self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
        Some(run)
    }

    pub(crate) fn insert(
        &mut self,
        key: ShapedRunCacheKey,
        text: impl Into<Arc<str>>,
        run: ShapedGlyphRun,
    ) -> Arc<ShapedGlyphRun> {
        let text = text.into();
        let run = Arc::new(run);
        let estimated_bytes = estimated_entry_bytes(text.as_ref(), run.as_ref());

        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.key == key && entry.text == text)
        {
            self.estimated_bytes = self
                .estimated_bytes
                .saturating_sub(self.entries[index].estimated_bytes)
                .saturating_add(estimated_bytes);
            self.entries[index].run = Arc::clone(&run);
            self.entries[index].estimated_bytes = estimated_bytes;
            self.touch_entry(index);
            self.frame_report.update_count = self.frame_report.update_count.saturating_add(1);
            self.trim_to_limits();
            return run;
        }

        let touch_order = self.next_touch_order();
        self.estimated_bytes = self.estimated_bytes.saturating_add(estimated_bytes);
        self.entries.push(ShapedRunCacheEntry {
            key,
            text,
            run: Arc::clone(&run),
            estimated_bytes,
            last_used_frame: self.current_frame,
            touch_order,
        });
        self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        self.trim_to_limits();
        run
    }

    pub(crate) fn get_or_insert_with(
        &mut self,
        key: ShapedRunCacheKey,
        text: &str,
        shape: impl FnOnce() -> ShapedGlyphRun,
    ) -> Arc<ShapedGlyphRun> {
        if let Some(run) = self.get(&key, text) {
            return run;
        }
        self.insert(key, text, shape())
    }

    fn touch_entry(&mut self, index: usize) {
        let touch_order = self.next_touch_order();
        let entry = &mut self.entries[index];
        entry.last_used_frame = self.current_frame;
        entry.touch_order = touch_order;
    }

    fn next_touch_order(&mut self) -> u64 {
        self.touch_order = self.touch_order.saturating_add(1);
        self.touch_order
    }

    fn trim_to_limits(&mut self) {
        let mut evicted = 0_u64;
        while self.over_limits() {
            let Some(index) = self.oldest_entry_index() else {
                break;
            };
            let removed = self.entries.remove(index);
            self.estimated_bytes = self.estimated_bytes.saturating_sub(removed.estimated_bytes);
            evicted = evicted.saturating_add(1);
        }

        if evicted > 0 {
            self.frame_report.evicted_count =
                self.frame_report.evicted_count.saturating_add(evicted);
            self.frame_report.trim_count = self.frame_report.trim_count.saturating_add(1);
        }
        self.refresh_report_size();
    }

    fn over_limits(&self) -> bool {
        self.entries.len() > self.capacity || self.estimated_bytes > self.max_bytes
    }

    fn oldest_entry_index(&self) -> Option<usize> {
        self.entries
            .iter()
            .enumerate()
            .min_by_key(|(_, entry)| (entry.last_used_frame, entry.touch_order))
            .map(|(index, _)| index)
    }

    fn refresh_report_size(&mut self) {
        self.frame_report.entry_count = self.entries.len();
        self.frame_report.estimated_bytes = self.estimated_bytes;
    }
}

fn hash_text(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

fn shaping_features_hash(include_kerning: bool) -> u64 {
    let mut hasher = DefaultHasher::new();
    b"shaped-run-features-v1".hash(&mut hasher);
    include_kerning.hash(&mut hasher);
    hasher.finish()
}

fn cache_font_family(style: &UiResolvedStyle) -> Option<String> {
    style
        .font_family
        .as_deref()
        .or(style.font.as_deref())
        .map(str::trim)
        .filter(|family| !family.is_empty())
        .map(ToOwned::to_owned)
}

fn normalized_f32_bits(value: f32) -> u32 {
    if value == 0.0 {
        0.0_f32.to_bits()
    } else {
        value.to_bits()
    }
}

fn estimated_entry_bytes(text: &str, run: &ShapedGlyphRun) -> usize {
    let line_bytes = run.lines.iter().fold(0_usize, |total, line| {
        total
            .saturating_add(size_of::<ShapedTextLine>())
            .saturating_add(line.text.len())
            .saturating_add(line.glyphs.len().saturating_mul(size_of::<ShapedGlyph>()))
    });

    size_of::<ShapedRunCacheEntry>()
        .saturating_add(size_of::<ShapedGlyphRun>())
        .saturating_add(text.len())
        .saturating_add(run.source_text.len())
        .saturating_add(line_bytes)
}
