use std::sync::Arc;

pub(crate) const DEFAULT_TEXT_MEASURE_CACHE_CAPACITY: usize = 4096;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextMeasureCacheReport {
    pub(crate) frame_index: u64,
    pub(crate) capacity: usize,
    pub(crate) entry_count: usize,
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
struct TextMeasureCacheEntry<K, V> {
    key: K,
    text: Arc<str>,
    value: V,
    last_used_frame: u64,
    touch_order: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextMeasureCache<K, V> {
    entries: Vec<TextMeasureCacheEntry<K, V>>,
    capacity: usize,
    current_frame: u64,
    touch_order: u64,
    frame_report: TextMeasureCacheReport,
}

impl<K, V> Default for TextMeasureCache<K, V>
where
    K: Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> TextMeasureCache<K, V>
where
    K: Eq,
{
    pub(crate) fn new() -> Self {
        Self::with_capacity(DEFAULT_TEXT_MEASURE_CACHE_CAPACITY)
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let mut cache = Self {
            entries: Vec::new(),
            capacity,
            current_frame: 0,
            touch_order: 0,
            frame_report: TextMeasureCacheReport::default(),
        };
        cache.frame_report.capacity = capacity;
        cache
    }

    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.current_frame = frame_index;
        self.frame_report = TextMeasureCacheReport {
            frame_index,
            capacity: self.capacity,
            entry_count: self.entries.len(),
            ..TextMeasureCacheReport::default()
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

    pub(crate) fn report(&self) -> TextMeasureCacheReport {
        let mut report = self.frame_report;
        report.entry_count = self.entries.len();
        report
    }

    pub(crate) fn contains_exact(&self, key: &K, text: &str) -> bool {
        self.entries
            .iter()
            .any(|entry| &entry.key == key && entry.text.as_ref() == text)
    }

    pub(crate) fn get(&mut self, key: &K, text: &str) -> Option<&V> {
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

        self.touch_entry(index);
        self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
        Some(&self.entries[index].value)
    }

    pub(crate) fn insert(&mut self, key: K, text: impl Into<Arc<str>>, value: V) -> &V {
        let text = text.into();
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| &entry.key == &key && entry.text.as_ref() == text.as_ref())
        {
            self.entries[index].value = value;
            self.touch_entry(index);
            self.frame_report.update_count = self.frame_report.update_count.saturating_add(1);
            return &self.entries[index].value;
        }

        self.trim_before_insert();
        let touch_order = self.next_touch_order();
        self.entries.push(TextMeasureCacheEntry {
            key,
            text,
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
        measure: impl FnOnce() -> V,
    ) -> (&V, bool) {
        let text = text.into();
        let mut collision_seen = false;
        let mut hit_index = None;

        for (index, entry) in self.entries.iter().enumerate() {
            if &entry.key != &key {
                continue;
            }
            if entry.text.as_ref() == text.as_ref() {
                hit_index = Some(index);
                break;
            }
            collision_seen = true;
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
        self.trim_before_insert();
        let touch_order = self.next_touch_order();
        self.entries.push(TextMeasureCacheEntry {
            key,
            text,
            value: measure(),
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
