use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

pub(crate) const STATIC_FIELD_CACHE_CAPACITY: usize = 16;
const MAX_HEATMAP_CELLS: usize = 4_096;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WeightHeatmapSource {
    x: f32,
    y: f32,
    weight: f32,
    selected: bool,
}

impl WeightHeatmapSource {
    pub(crate) fn new(x: f32, y: f32, weight: f32, selected: bool) -> Self {
        Self {
            x: normalized_unit_value(x),
            y: normalized_unit_value(y),
            weight: normalized_unit_value(weight),
            selected,
        }
    }

    pub(crate) fn x(&self) -> f32 {
        self.x
    }

    pub(crate) fn y(&self) -> f32 {
        self.y
    }

    pub(crate) fn weight(&self) -> f32 {
        self.weight
    }

    pub(crate) fn selected(&self) -> bool {
        self.selected
    }
}

#[derive(Clone, Debug)]
pub(crate) struct WeightHeatmapGeneration {
    columns: usize,
    rows: usize,
    low_label: Arc<str>,
    high_label: Arc<str>,
    sources: Arc<[WeightHeatmapSource]>,
    static_generation: u64,
    dynamic_generation: u64,
}

pub(crate) struct WeightHeatmapGenerationInput {
    pub(crate) columns: i32,
    pub(crate) rows: i32,
    pub(crate) low_label: String,
    pub(crate) high_label: String,
    pub(crate) sources: Vec<WeightHeatmapSource>,
}

#[derive(Debug)]
pub(crate) struct WeightHeatmapStaticField {
    columns: usize,
    rows: usize,
    intensities: Arc<[f32]>,
    generation: u64,
}

impl WeightHeatmapStaticField {
    pub(crate) fn columns(&self) -> usize {
        self.columns
    }

    pub(crate) fn rows(&self) -> usize {
        self.rows
    }

    pub(crate) fn cell_count(&self) -> usize {
        self.columns.saturating_mul(self.rows)
    }

    pub(crate) fn intensity_at(&self, row: usize, column: usize) -> f32 {
        self.intensities
            .get(row.saturating_mul(self.columns).saturating_add(column))
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn intensities(&self) -> &[f32] {
        &self.intensities
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
}

impl Default for WeightHeatmapGeneration {
    fn default() -> Self {
        Self::new(WeightHeatmapGenerationInput {
            columns: 12,
            rows: 8,
            low_label: "0.0".to_owned(),
            high_label: "1.0".to_owned(),
            sources: Vec::new(),
        })
    }
}

impl WeightHeatmapGeneration {
    pub(crate) fn new(input: WeightHeatmapGenerationInput) -> Self {
        let columns = normalize_columns(input.columns);
        let rows = normalize_rows(input.rows);
        let low_label: Arc<str> = Arc::from(input.low_label);
        let high_label: Arc<str> = Arc::from(input.high_label);
        let sources: Arc<[WeightHeatmapSource]> = input.sources.into();
        let static_generation = static_generation(columns, rows, &low_label, &high_label, &sources);
        let dynamic_generation = dynamic_generation(&sources);

        Self {
            columns,
            rows,
            low_label,
            high_label,
            sources,
            static_generation,
            dynamic_generation,
        }
    }

    pub(crate) fn columns(&self) -> usize {
        self.columns
    }

    pub(crate) fn rows(&self) -> usize {
        self.rows
    }

    pub(crate) fn low_label(&self) -> &str {
        &self.low_label
    }

    pub(crate) fn high_label(&self) -> &str {
        &self.high_label
    }

    pub(crate) fn sources(&self) -> &[WeightHeatmapSource] {
        &self.sources
    }

    pub(crate) fn static_generation(&self) -> u64 {
        self.static_generation
    }

    pub(crate) fn dynamic_generation(&self) -> u64 {
        self.dynamic_generation
    }

    pub(crate) fn static_field_for_plot_size(
        &self,
        plot_width: f32,
        plot_height: f32,
    ) -> Arc<WeightHeatmapStaticField> {
        let (columns, rows) = bounded_grid_dimensions(
            self.columns,
            self.rows,
            plot_width,
            plot_height,
            MAX_HEATMAP_CELLS,
        );
        let key = StaticFieldCacheKey {
            static_generation: self.static_generation,
            columns,
            rows,
        };
        if let Some(field) = static_field_cache()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(key)
        {
            return field;
        }

        // Source evaluation is performed once per immutable generation/visual budget.
        let candidate = Arc::new(WeightHeatmapStaticField::new(
            columns,
            rows,
            &self.sources,
            self.static_generation,
        ));
        static_field_cache()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert_or_get(key, candidate)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct StaticFieldCacheKey {
    static_generation: u64,
    columns: usize,
    rows: usize,
}

struct StaticFieldCacheEntry {
    field: Arc<WeightHeatmapStaticField>,
    last_used: u64,
}

#[derive(Default)]
struct StaticFieldCache {
    entries: HashMap<StaticFieldCacheKey, StaticFieldCacheEntry>,
    access_generation: u64,
}

impl StaticFieldCache {
    fn get(&mut self, key: StaticFieldCacheKey) -> Option<Arc<WeightHeatmapStaticField>> {
        self.rebase_access_generations_if_needed();
        let entry = self.entries.get_mut(&key)?;
        self.access_generation += 1;
        entry.last_used = self.access_generation;
        Some(Arc::clone(&entry.field))
    }

    fn insert_or_get(
        &mut self,
        key: StaticFieldCacheKey,
        candidate: Arc<WeightHeatmapStaticField>,
    ) -> Arc<WeightHeatmapStaticField> {
        if let Some(field) = self.get(key) {
            return field;
        }
        while self.entries.len() >= STATIC_FIELD_CACHE_CAPACITY {
            let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| *key)
            else {
                break;
            };
            self.entries.remove(&oldest);
        }
        let last_used = self.next_access_generation();
        self.entries.insert(
            key,
            StaticFieldCacheEntry {
                field: Arc::clone(&candidate),
                last_used,
            },
        );
        candidate
    }

    fn next_access_generation(&mut self) -> u64 {
        self.rebase_access_generations_if_needed();
        self.access_generation += 1;
        self.access_generation
    }

    fn rebase_access_generations_if_needed(&mut self) {
        if self.access_generation != u64::MAX {
            return;
        }
        let mut entries = self.entries.values_mut().collect::<Vec<_>>();
        entries.sort_unstable_by_key(|entry| entry.last_used);
        for (index, entry) in entries.into_iter().enumerate() {
            entry.last_used = index as u64 + 1;
        }
        self.access_generation = self.entries.len() as u64;
    }
}

static STATIC_FIELD_CACHE: OnceLock<Mutex<StaticFieldCache>> = OnceLock::new();

fn static_field_cache() -> &'static Mutex<StaticFieldCache> {
    STATIC_FIELD_CACHE.get_or_init(|| Mutex::new(StaticFieldCache::default()))
}

#[cfg(test)]
pub(crate) fn static_field_cache_entry_count() -> usize {
    static_field_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .entries
        .len()
}

impl WeightHeatmapStaticField {
    fn new(
        columns: usize,
        rows: usize,
        sources: &[WeightHeatmapSource],
        static_generation: u64,
    ) -> Self {
        let intensities: Arc<[f32]> = (0..rows)
            .flat_map(|row| {
                (0..columns).map(move |column| {
                    let x = (column as f32 + 0.5) / columns as f32;
                    let y = 1.0 - (row as f32 + 0.5) / rows as f32;
                    heat_intensity(sources, x, y)
                })
            })
            .collect::<Vec<_>>()
            .into();
        let mut generation = GenerationHash::new();
        generation.add_u64(static_generation);
        generation.add_u64(columns as u64);
        generation.add_u64(rows as u64);
        generation.add_u64(intensities.len() as u64);
        for intensity in intensities.iter() {
            generation.add_f32(*intensity);
        }
        Self {
            columns,
            rows,
            intensities,
            generation: generation.finish(),
        }
    }
}

fn normalize_columns(columns: i32) -> usize {
    columns.clamp(4, 32) as usize
}

fn normalize_rows(rows: i32) -> usize {
    rows.clamp(3, 24) as usize
}

fn normalized_unit_value(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn bounded_grid_dimensions(
    requested_columns: usize,
    requested_rows: usize,
    plot_width: f32,
    plot_height: f32,
    max_cells: usize,
) -> (usize, usize) {
    let max_cells = max_cells.max(1);
    let mut columns = requested_columns
        .min(pixel_axis_budget(plot_width))
        .min(max_cells);
    let mut rows = requested_rows
        .min(pixel_axis_budget(plot_height))
        .min(max_cells);
    if columns.saturating_mul(rows) <= max_cells {
        return (columns, rows);
    }

    let square_root = (max_cells as f64).sqrt().floor() as usize;
    if columns <= square_root {
        rows = rows.min(max_cells / columns);
    } else if rows <= square_root {
        columns = columns.min(max_cells / rows);
    } else {
        let scale = (max_cells as f64 / (columns as f64 * rows as f64)).sqrt();
        columns = ((columns as f64 * scale).floor() as usize).max(1);
        rows = ((rows as f64 * scale).floor() as usize).max(1);
        rows = rows.min(max_cells / columns);
    }
    (columns.max(1), rows.max(1))
}

fn pixel_axis_budget(extent: f32) -> usize {
    if extent.is_finite() && extent > 0.0 {
        (extent.ceil() as usize).max(1)
    } else {
        1
    }
}

fn heat_intensity(sources: &[WeightHeatmapSource], x: f32, y: f32) -> f32 {
    let mut intensity = 0.0f32;
    for source in sources {
        let dx = x - source.x;
        let dy = y - source.y;
        let influence = source.weight * (-8.0 * (dx * dx + dy * dy)).exp();
        intensity = intensity.max(influence);
    }
    intensity.clamp(0.0, 1.0)
}

fn static_generation(
    columns: usize,
    rows: usize,
    low_label: &str,
    high_label: &str,
    sources: &[WeightHeatmapSource],
) -> u64 {
    let mut generation = GenerationHash::new();
    generation.add_u64(columns as u64);
    generation.add_u64(rows as u64);
    generation.add_str(low_label);
    generation.add_str(high_label);
    generation.add_u64(sources.len() as u64);
    for source in sources {
        generation.add_f32(source.x);
        generation.add_f32(source.y);
        generation.add_f32(source.weight);
    }
    generation.finish()
}

fn dynamic_generation(sources: &[WeightHeatmapSource]) -> u64 {
    let mut generation = GenerationHash::new();
    generation.add_u64(sources.len() as u64);
    for source in sources {
        generation.add_byte(u8::from(source.selected));
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

    fn add_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.add_byte(*byte);
        }
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

    fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod hash_generation_tests;
