use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex, OnceLock},
};

const MAX_TIMELINE_TICKS: usize = 4_096;
pub(crate) const STATIC_CONTENT_CACHE_CAPACITY: usize = 16;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TimelineStripTick {
    value: f32,
    label: Arc<str>,
}

impl TimelineStripTick {
    pub(crate) fn value(&self) -> f32 {
        self.value
    }

    pub(crate) fn label(&self) -> &str {
        &self.label
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TimelineStripKey {
    time: f32,
    label: Arc<str>,
    selected: bool,
}

impl TimelineStripKey {
    pub(crate) fn new(time: f32, label: impl Into<String>, selected: bool) -> Self {
        Self {
            time,
            label: Arc::from(label.into()),
            selected,
        }
    }

    pub(crate) fn time(&self) -> f32 {
        self.time
    }

    pub(crate) fn label(&self) -> &str {
        &self.label
    }

    pub(crate) fn selected(&self) -> bool {
        self.selected
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TimelineStripGeneration {
    duration: f32,
    current_time: f32,
    tick_interval: f32,
    track_label: Arc<str>,
    keys: Arc<[TimelineStripKey]>,
    static_generation: u64,
    dynamic_generation: u64,
}

pub(crate) struct TimelineStripGenerationInput {
    pub(crate) duration: f32,
    pub(crate) current_time: f32,
    pub(crate) tick_interval: f32,
    pub(crate) track_label: String,
    pub(crate) keys: Vec<TimelineStripKey>,
}

#[derive(Debug)]
pub(crate) struct TimelineStripStaticContent {
    ticks: Arc<[TimelineStripTick]>,
    generation: u64,
}

impl TimelineStripStaticContent {
    pub(crate) fn ticks(&self) -> &[TimelineStripTick] {
        &self.ticks
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
}

impl Default for TimelineStripGeneration {
    fn default() -> Self {
        Self::new(TimelineStripGenerationInput {
            duration: 1.0,
            current_time: 0.0,
            tick_interval: 0.25,
            track_label: String::new(),
            keys: Vec::new(),
        })
    }
}

impl TimelineStripGeneration {
    pub(crate) fn new(input: TimelineStripGenerationInput) -> Self {
        let duration = normalized_duration(input.duration);
        let current_time = normalized_current_time(input.current_time, duration);
        let tick_interval = normalized_tick_interval(input.tick_interval, duration);
        let track_label: Arc<str> = Arc::from(input.track_label);
        let keys: Arc<[TimelineStripKey]> = input
            .keys
            .into_iter()
            .filter(|key| key.time.is_finite())
            .map(|key| TimelineStripKey {
                time: key.time.clamp(0.0, duration),
                ..key
            })
            .collect::<Vec<_>>()
            .into();
        let static_generation = static_generation(duration, tick_interval, &track_label, &keys);
        let dynamic_generation = dynamic_generation(current_time, &keys);

        Self {
            duration,
            current_time,
            tick_interval,
            track_label,
            keys,
            static_generation,
            dynamic_generation,
        }
    }

    pub(crate) fn duration(&self) -> f32 {
        self.duration
    }

    pub(crate) fn current_time(&self) -> f32 {
        self.current_time
    }

    pub(crate) fn tick_interval(&self) -> f32 {
        self.tick_interval
    }

    pub(crate) fn track_label(&self) -> &str {
        &self.track_label
    }

    pub(crate) fn keys(&self) -> &[TimelineStripKey] {
        &self.keys
    }

    pub(crate) fn static_generation(&self) -> u64 {
        self.static_generation
    }

    pub(crate) fn dynamic_generation(&self) -> u64 {
        self.dynamic_generation
    }

    pub(crate) fn static_content_for_plot_width(
        &self,
        plot_width: f32,
    ) -> Arc<TimelineStripStaticContent> {
        let visual_budget = visual_tick_budget(plot_width);
        let key = StaticContentCacheKey {
            static_generation: self.static_generation,
            visual_budget,
        };
        if let Some(content) = static_content_cache()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(key)
        {
            return content;
        }

        let candidate = Arc::new(TimelineStripStaticContent::new(
            self.duration,
            self.tick_interval,
            self.static_generation,
            visual_budget,
        ));
        static_content_cache()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert_or_get(key, candidate)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct StaticContentCacheKey {
    static_generation: u64,
    visual_budget: usize,
}

#[derive(Default)]
struct StaticContentCache {
    entries: BTreeMap<StaticContentCacheKey, Arc<TimelineStripStaticContent>>,
    recency: VecDeque<StaticContentCacheKey>,
}

impl StaticContentCache {
    fn get(&mut self, key: StaticContentCacheKey) -> Option<Arc<TimelineStripStaticContent>> {
        let content = self.entries.get(&key).cloned();
        if content.is_some() {
            self.touch(key);
        }
        content
    }

    fn insert_or_get(
        &mut self,
        key: StaticContentCacheKey,
        candidate: Arc<TimelineStripStaticContent>,
    ) -> Arc<TimelineStripStaticContent> {
        if let Some(content) = self.get(key) {
            return content;
        }
        self.entries.insert(key, candidate.clone());
        self.touch(key);
        while self.entries.len() > STATIC_CONTENT_CACHE_CAPACITY {
            let Some(oldest) = self.recency.pop_front() else {
                break;
            };
            self.entries.remove(&oldest);
        }
        candidate
    }

    fn touch(&mut self, key: StaticContentCacheKey) {
        self.recency.retain(|entry| *entry != key);
        self.recency.push_back(key);
    }
}

static STATIC_CONTENT_CACHE: OnceLock<Mutex<StaticContentCache>> = OnceLock::new();

fn static_content_cache() -> &'static Mutex<StaticContentCache> {
    STATIC_CONTENT_CACHE.get_or_init(|| Mutex::new(StaticContentCache::default()))
}

#[cfg(test)]
pub(crate) fn static_content_cache_entry_count() -> usize {
    static_content_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .entries
        .len()
}

impl TimelineStripStaticContent {
    fn new(
        duration: f32,
        tick_interval: f32,
        static_generation: u64,
        visual_budget: usize,
    ) -> Self {
        let ticks: Arc<[TimelineStripTick]> =
            timeline_tick_values(duration, tick_interval, visual_budget)
                .into_iter()
                .map(TimelineStripTick::from_value)
                .collect::<Vec<_>>()
                .into();
        let mut generation = GenerationHash::new();
        generation.add_u64(static_generation);
        generation.add_u64(visual_budget as u64);
        generation.add_u64(ticks.len() as u64);
        for tick in ticks.iter() {
            generation.add_f32(tick.value);
            generation.add_str(&tick.label);
        }
        Self {
            ticks,
            generation: generation.finish(),
        }
    }
}

impl TimelineStripTick {
    fn from_value(value: f32) -> Self {
        Self {
            value,
            label: Arc::from(format_time(value)),
        }
    }
}

fn normalized_duration(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

fn normalized_current_time(value: f32, duration: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, duration)
    } else {
        0.0
    }
}

fn normalized_tick_interval(value: f32, duration: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value.min(duration)
    } else {
        duration.min(0.25)
    }
}

fn visual_tick_budget(plot_width: f32) -> usize {
    let columns = if plot_width.is_finite() && plot_width > 0.0 {
        plot_width.ceil() as usize
    } else {
        1
    };
    columns.saturating_add(1).clamp(2, MAX_TIMELINE_TICKS)
}

fn timeline_tick_values(duration: f32, interval: f32, max_ticks: usize) -> Vec<f32> {
    let max_ticks = max_ticks.clamp(2, MAX_TIMELINE_TICKS);
    let requested_segments = (duration / interval).ceil();
    let segment_budget = max_ticks - 1;
    let segment_count =
        if !requested_segments.is_finite() || requested_segments >= segment_budget as f32 {
            segment_budget
        } else {
            (requested_segments as usize).max(1)
        };
    let step = if requested_segments > segment_count as f32 {
        duration / segment_count as f32
    } else {
        interval
    };
    let mut ticks = Vec::with_capacity(segment_count + 1);
    for index in 0..segment_count {
        ticks.push(index as f32 * step);
    }
    ticks.push(duration);
    ticks
}

fn format_time(time: f32) -> String {
    format!("{time:.1}")
}

fn static_generation(
    duration: f32,
    tick_interval: f32,
    track_label: &str,
    keys: &[TimelineStripKey],
) -> u64 {
    let mut generation = GenerationHash::new();
    generation.add_f32(duration);
    generation.add_f32(tick_interval);
    generation.add_str(track_label);
    generation.add_u64(keys.len() as u64);
    for key in keys {
        generation.add_f32(key.time);
        generation.add_str(&key.label);
    }
    generation.finish()
}

fn dynamic_generation(current_time: f32, keys: &[TimelineStripKey]) -> u64 {
    let mut generation = GenerationHash::new();
    generation.add_f32(current_time);
    generation.add_u64(keys.len() as u64);
    for key in keys {
        generation.add_byte(u8::from(key.selected));
    }
    generation.finish()
}

struct GenerationHash(u64);

impl GenerationHash {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    fn new() -> Self {
        Self(Self::FNV_OFFSET)
    }

    fn add_byte(&mut self, byte: u8) {
        self.0 ^= u64::from(byte);
        self.0 = self.0.wrapping_mul(Self::FNV_PRIME);
    }

    fn add_f32(&mut self, value: f32) {
        self.add_bytes(&value.to_bits().to_le_bytes());
    }

    fn add_u64(&mut self, value: u64) {
        self.add_bytes(&value.to_le_bytes());
    }

    fn add_str(&mut self, value: &str) {
        self.add_u64(value.len() as u64);
        self.add_bytes(value.as_bytes());
    }

    fn add_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.add_byte(*byte);
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}
