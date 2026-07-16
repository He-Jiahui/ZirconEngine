use std::sync::Arc;

pub(crate) const DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY: usize = 2048;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextLayoutWidthValidity {
    Exact(u32),
    Range {
        min_width_bits: u32,
        max_width_bits: u32,
    },
}

impl TextLayoutWidthValidity {
    pub(crate) fn exact(width: f32) -> Self {
        Self::Exact(normalized_width_bits(width))
    }

    pub(crate) fn range(min_width: f32, max_width: f32) -> Self {
        Self::Range {
            min_width_bits: normalized_width_bits(min_width),
            max_width_bits: normalized_width_bits(max_width),
        }
    }

    pub(crate) fn contains(self, width: f32) -> bool {
        match self {
            Self::Exact(bits) => normalized_width_bits(width) == bits,
            Self::Range {
                min_width_bits,
                max_width_bits,
            } => {
                let min_width = f32::from_bits(min_width_bits);
                let max_width = f32::from_bits(max_width_bits);
                min_width.is_finite()
                    && max_width.is_finite()
                    && width.is_finite()
                    && min_width <= width
                    && width < max_width
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextLayoutCacheReport {
    pub(crate) frame_index: u64,
    pub(crate) capacity: usize,
    pub(crate) entry_count: usize,
    pub(crate) hit_count: u64,
    pub(crate) miss_count: u64,
    pub(crate) collision_miss_count: u64,
    pub(crate) width_miss_count: u64,
    pub(crate) insert_count: u64,
    pub(crate) update_count: u64,
    pub(crate) evicted_count: u64,
    pub(crate) trim_count: u64,
    pub(crate) clear_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct TextLayoutCacheEntry<K, V> {
    key: K,
    text: Arc<str>,
    width_validity: TextLayoutWidthValidity,
    value: V,
    last_used_frame: u64,
    touch_order: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextLayoutCache<K, V> {
    entries: Vec<TextLayoutCacheEntry<K, V>>,
    capacity: usize,
    current_frame: u64,
    touch_order: u64,
    frame_report: TextLayoutCacheReport,
}

impl<K, V> Default for TextLayoutCache<K, V>
where
    K: Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> TextLayoutCache<K, V>
where
    K: Eq,
{
    pub(crate) fn new() -> Self {
        Self::with_capacity(DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY)
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let mut cache = Self {
            entries: Vec::new(),
            capacity,
            current_frame: 0,
            touch_order: 0,
            frame_report: TextLayoutCacheReport::default(),
        };
        cache.frame_report.capacity = capacity;
        cache
    }

    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.current_frame = frame_index;
        self.frame_report = TextLayoutCacheReport {
            frame_index,
            capacity: self.capacity,
            entry_count: self.entries.len(),
            ..TextLayoutCacheReport::default()
        };
    }

    pub(crate) fn finish_frame(&mut self) {
        self.trim_to_capacity();
    }

    pub(crate) fn clear(&mut self) {
        self.frame_report.evicted_count = self
            .frame_report
            .evicted_count
            .saturating_add(self.entries.len() as u64);
        self.frame_report.clear_count = self.frame_report.clear_count.saturating_add(1);
        self.entries.clear();
        self.refresh_report_size();
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn report(&self) -> TextLayoutCacheReport {
        let mut report = self.frame_report;
        report.entry_count = self.entries.len();
        report
    }

    pub(crate) fn contains_exact(
        &self,
        key: &K,
        text: &str,
        width_validity: TextLayoutWidthValidity,
    ) -> bool {
        self.entries.iter().any(|entry| {
            &entry.key == key
                && entry.text.as_ref() == text
                && entry.width_validity == width_validity
        })
    }

    pub(crate) fn get(&mut self, key: &K, text: &str, wrap_width: f32) -> Option<&V> {
        let mut collision_seen = false;
        let mut width_miss_seen = false;
        let mut hit_index = None;

        for (index, entry) in self.entries.iter().enumerate() {
            if &entry.key != key {
                continue;
            }
            if entry.text.as_ref() != text {
                collision_seen = true;
                continue;
            }
            if !entry.width_validity.contains(wrap_width) {
                width_miss_seen = true;
                continue;
            }
            hit_index = Some(index);
            break;
        }

        let Some(index) = hit_index else {
            self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
            if collision_seen {
                self.frame_report.collision_miss_count =
                    self.frame_report.collision_miss_count.saturating_add(1);
            }
            if width_miss_seen {
                self.frame_report.width_miss_count =
                    self.frame_report.width_miss_count.saturating_add(1);
            }
            return None;
        };

        self.touch_entry(index);
        self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
        Some(&self.entries[index].value)
    }

    pub(crate) fn insert(
        &mut self,
        key: K,
        text: impl Into<Arc<str>>,
        width_validity: TextLayoutWidthValidity,
        value: V,
    ) -> &V {
        let text = text.into();
        if let Some(index) = self.entries.iter().position(|entry| {
            &entry.key == &key
                && entry.text.as_ref() == text.as_ref()
                && entry.width_validity == width_validity
        }) {
            self.entries[index].value = value;
            self.touch_entry(index);
            self.frame_report.update_count = self.frame_report.update_count.saturating_add(1);
            return &self.entries[index].value;
        }

        self.trim_before_insert();
        let touch_order = self.next_touch_order();
        self.entries.push(TextLayoutCacheEntry {
            key,
            text,
            width_validity,
            value,
            last_used_frame: self.current_frame,
            touch_order,
        });
        self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        self.refresh_report_size();
        &self.entries.last().expect("entry was just pushed").value
    }

    pub(crate) fn get_or_insert_with(
        &mut self,
        key: K,
        text: impl Into<Arc<str>>,
        width_validity: TextLayoutWidthValidity,
        wrap_width: f32,
        layout: impl FnOnce() -> V,
    ) -> (&V, bool) {
        let text = text.into();
        let mut collision_seen = false;
        let mut width_miss_seen = false;
        let mut hit_index = None;

        for (index, entry) in self.entries.iter().enumerate() {
            if &entry.key != &key {
                continue;
            }
            if entry.text.as_ref() != text.as_ref() {
                collision_seen = true;
                continue;
            }
            if !entry.width_validity.contains(wrap_width) {
                width_miss_seen = true;
                continue;
            }
            hit_index = Some(index);
            break;
        }

        if let Some(index) = hit_index {
            self.touch_entry(index);
            self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
            return (&self.entries[index].value, false);
        }

        self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
        if collision_seen {
            self.frame_report.collision_miss_count =
                self.frame_report.collision_miss_count.saturating_add(1);
        }
        if width_miss_seen {
            self.frame_report.width_miss_count =
                self.frame_report.width_miss_count.saturating_add(1);
        }
        self.trim_before_insert();
        let touch_order = self.next_touch_order();
        self.entries.push(TextLayoutCacheEntry {
            key,
            text,
            width_validity,
            value: layout(),
            last_used_frame: self.current_frame,
            touch_order,
        });
        self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        self.refresh_report_size();
        (
            &self.entries.last().expect("entry was just pushed").value,
            true,
        )
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

    fn trim_before_insert(&mut self) {
        let mut evicted = 0_u64;
        while self.entries.len() >= self.capacity {
            let Some(index) = self.oldest_entry_index() else {
                break;
            };
            self.entries.remove(index);
            evicted = evicted.saturating_add(1);
        }
        self.record_evictions(evicted);
    }

    fn trim_to_capacity(&mut self) {
        let mut evicted = 0_u64;
        while self.entries.len() > self.capacity {
            let Some(index) = self.oldest_entry_index() else {
                break;
            };
            self.entries.remove(index);
            evicted = evicted.saturating_add(1);
        }
        self.record_evictions(evicted);
    }

    fn record_evictions(&mut self, evicted: u64) {
        if evicted > 0 {
            self.frame_report.evicted_count =
                self.frame_report.evicted_count.saturating_add(evicted);
            self.frame_report.trim_count = self.frame_report.trim_count.saturating_add(1);
        }
        self.refresh_report_size();
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
    }
}

fn normalized_width_bits(width: f32) -> u32 {
    if width == 0.0 {
        0.0_f32.to_bits()
    } else {
        width.to_bits()
    }
}
