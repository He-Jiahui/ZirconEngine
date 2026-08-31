use std::borrow::Borrow;
use std::collections::{BTreeMap, VecDeque};

const DEFAULT_DIAGNOSTIC_HISTORY_LIMIT: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticPath(String);

impl DiagnosticPath {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for DiagnosticPath {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for DiagnosticPath {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl Borrow<str> for DiagnosticPath {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticMeasurement {
    pub frame_index: u64,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticSeriesSnapshot {
    pub path: DiagnosticPath,
    pub unit: Option<String>,
    pub subsystem_tags: Vec<String>,
    pub current: Option<f64>,
    pub smoothed: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub history: Vec<DiagnosticMeasurement>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticStoreSnapshot {
    pub series: Vec<DiagnosticSeriesSnapshot>,
}

impl DiagnosticStoreSnapshot {
    pub fn is_empty(&self) -> bool {
        self.series.is_empty()
    }
}

impl Default for DiagnosticStoreSnapshot {
    fn default() -> Self {
        Self { series: Vec::new() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticSeriesCurrentSnapshot {
    pub path: DiagnosticPath,
    pub unit: Option<String>,
    pub current: f64,
    pub smoothed: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiagnosticStoreCurrentSnapshot {
    pub series: Vec<DiagnosticSeriesCurrentSnapshot>,
}

impl DiagnosticStoreCurrentSnapshot {
    pub fn is_empty(&self) -> bool {
        self.series.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct DiagnosticStore {
    history_limit: usize,
    series: BTreeMap<DiagnosticPath, DiagnosticSeries>,
}

impl DiagnosticStore {
    pub fn new(history_limit: usize) -> Self {
        Self {
            history_limit: history_limit.max(1),
            series: BTreeMap::new(),
        }
    }

    pub fn record<U, T>(
        &mut self,
        path: impl Into<DiagnosticPath>,
        frame_index: u64,
        value: f64,
        unit: Option<U>,
        subsystem_tags: impl IntoIterator<Item = T>,
    ) where
        U: Into<String>,
        T: Into<String>,
    {
        let series = self
            .series
            .entry(path.into())
            .or_insert_with(|| DiagnosticSeries::new(self.history_limit));
        series.record(frame_index, value, unit, subsystem_tags);
    }

    pub(crate) fn record_static(
        &mut self,
        path: &'static str,
        frame_index: u64,
        value: f64,
        unit: Option<&str>,
        subsystem_tags: &[&str],
    ) {
        if let Some(series) = self.series.get_mut(path) {
            if series.metadata_matches(unit, subsystem_tags) {
                series.record_measurement(frame_index, value);
            } else {
                series.record(frame_index, value, unit, subsystem_tags.iter().copied());
            }
            return;
        }

        let mut series = DiagnosticSeries::new(self.history_limit);
        series.record(frame_index, value, unit, subsystem_tags.iter().copied());
        self.series.insert(DiagnosticPath::new(path), series);
    }

    pub fn snapshot(&self) -> DiagnosticStoreSnapshot {
        DiagnosticStoreSnapshot {
            series: self
                .series
                .iter()
                .map(|(path, series)| series.snapshot(path.clone()))
                .collect(),
        }
    }

    pub fn current_snapshot(&self) -> DiagnosticStoreCurrentSnapshot {
        DiagnosticStoreCurrentSnapshot {
            series: self
                .series
                .iter()
                .filter_map(|(path, series)| series.current_snapshot(path.clone()))
                .collect(),
        }
    }
}

impl Default for DiagnosticStore {
    fn default() -> Self {
        Self::new(DEFAULT_DIAGNOSTIC_HISTORY_LIMIT)
    }
}

#[derive(Clone, Debug)]
struct DiagnosticSeries {
    history_limit: usize,
    unit: Option<String>,
    subsystem_tags: Vec<String>,
    current: Option<f64>,
    smoothed: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
    history: VecDeque<DiagnosticMeasurement>,
}

impl DiagnosticSeries {
    fn new(history_limit: usize) -> Self {
        Self {
            history_limit,
            unit: None,
            subsystem_tags: Vec::new(),
            current: None,
            smoothed: None,
            min: None,
            max: None,
            history: VecDeque::new(),
        }
    }

    fn record<U, T>(
        &mut self,
        frame_index: u64,
        value: f64,
        unit: Option<U>,
        subsystem_tags: impl IntoIterator<Item = T>,
    ) where
        U: Into<String>,
        T: Into<String>,
    {
        if let Some(unit) = unit {
            self.unit = Some(unit.into());
        }
        push_unique_tags(&mut self.subsystem_tags, subsystem_tags);
        self.record_measurement(frame_index, value);
    }

    fn metadata_matches(&self, unit: Option<&str>, subsystem_tags: &[&str]) -> bool {
        if self.unit.as_deref() != unit {
            return false;
        }

        let unique_tag_count = subsystem_tags
            .iter()
            .enumerate()
            .filter(|(index, tag)| !subsystem_tags[..*index].contains(tag))
            .count();
        unique_tag_count == self.subsystem_tags.len()
            && subsystem_tags.iter().all(|tag| {
                self.subsystem_tags
                    .iter()
                    .any(|existing| existing.as_str() == *tag)
            })
    }

    fn record_measurement(&mut self, frame_index: u64, value: f64) {
        self.current = Some(value);
        self.smoothed = Some(match self.smoothed {
            Some(previous) => previous.mul_add(0.9, value * 0.1),
            None => value,
        });
        self.min = Some(self.min.map_or(value, |current| current.min(value)));
        self.max = Some(self.max.map_or(value, |current| current.max(value)));
        if self.history.len() == self.history_limit {
            self.history.pop_front();
        }
        self.history
            .push_back(DiagnosticMeasurement { frame_index, value });
    }

    fn snapshot(&self, path: DiagnosticPath) -> DiagnosticSeriesSnapshot {
        DiagnosticSeriesSnapshot {
            path,
            unit: self.unit.clone(),
            subsystem_tags: self.subsystem_tags.clone(),
            current: self.current,
            smoothed: self.smoothed,
            min: self.min,
            max: self.max,
            history: self.history.iter().cloned().collect(),
        }
    }

    fn current_snapshot(&self, path: DiagnosticPath) -> Option<DiagnosticSeriesCurrentSnapshot> {
        Some(DiagnosticSeriesCurrentSnapshot {
            path,
            unit: self.unit.clone(),
            current: self.current?,
            smoothed: self.smoothed,
            min: self.min,
            max: self.max,
        })
    }
}

fn push_unique_tags(target: &mut Vec<String>, tags: impl IntoIterator<Item = impl Into<String>>) {
    for tag in tags.into_iter().map(Into::into) {
        if !target.iter().any(|existing| existing == &tag) {
            target.push(tag);
        }
    }
    target.sort();
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::time::Instant;

    use super::{DiagnosticMeasurement, DiagnosticSeries, DiagnosticStore};

    #[test]
    fn current_snapshot_omits_retained_history_and_tags() {
        let mut store = DiagnosticStore::new(4);
        for frame_index in 1..=4 {
            store.record(
                "time.frame_time",
                frame_index,
                frame_index as f64,
                Some("ms"),
                ["time", "frame"],
            );
        }

        let full = store.snapshot();
        let current = store.current_snapshot();

        assert_eq!(full.series[0].history.len(), 4);
        assert_eq!(full.series[0].subsystem_tags, ["frame", "time"]);
        assert_eq!(current.series.len(), 1);
        assert_eq!(current.series[0].path.as_str(), "time.frame_time");
        assert_eq!(current.series[0].unit.as_deref(), Some("ms"));
        assert_eq!(current.series[0].current, 4.0);
        assert_eq!(current.series[0].smoothed, full.series[0].smoothed);
        assert_eq!(current.series[0].min, Some(1.0));
        assert_eq!(current.series[0].max, Some(4.0));
    }

    #[test]
    fn static_diagnostic_series_reuses_path_and_metadata_allocations() {
        let mut store = DiagnosticStore::new(4);
        store.record_static("time.frame_time", 1, 16.0, Some("ms"), &["time", "frame"]);

        let series = store.series.get("time.frame_time").unwrap();
        let path_ptr = store.series.keys().next().unwrap().as_str().as_ptr();
        let unit_ptr = series.unit.as_ref().unwrap().as_ptr();
        let tags_ptr = series.subsystem_tags.as_ptr();

        store.record_static("time.frame_time", 2, 17.0, Some("ms"), &["time", "frame"]);

        let series = store.series.get("time.frame_time").unwrap();
        assert_eq!(
            store.series.keys().next().unwrap().as_str().as_ptr(),
            path_ptr
        );
        assert_eq!(series.unit.as_ref().unwrap().as_ptr(), unit_ptr);
        assert_eq!(series.subsystem_tags.as_ptr(), tags_ptr);
        assert_eq!(series.history.len(), 2);
        assert_eq!(series.current, Some(17.0));
    }

    #[test]
    fn diagnostic_store_records_history_summary_and_tags() {
        let mut store = DiagnosticStore::new(2);

        store.record("render.frame_ms", 1, 16.0, Some("ms"), ["render"]);
        store.record("render.frame_ms", 2, 20.0, Some("ms"), ["render", "frame"]);
        store.record("render.frame_ms", 3, 18.0, Some("ms"), ["render"]);

        let snapshot = store.snapshot();
        assert_eq!(snapshot.series.len(), 1);
        let series = &snapshot.series[0];
        assert_eq!(series.path.as_str(), "render.frame_ms");
        assert_eq!(series.unit.as_deref(), Some("ms"));
        assert_eq!(series.current, Some(18.0));
        assert_eq!(series.min, Some(16.0));
        assert_eq!(series.max, Some(20.0));
        assert_eq!(series.subsystem_tags, ["frame", "render"]);
        assert_eq!(series.history.len(), 2);
        assert_eq!(series.history[0].frame_index, 2);
        assert_eq!(series.history[1].value, 18.0);
    }

    #[test]
    fn optimization_wave_20260824b_runtime03_history_eviction_keeps_bounded_capacity() {
        const HISTORY_LIMIT: usize = 64;
        const WRITES: u64 = 4_096;

        let mut series = DiagnosticSeries::new(HISTORY_LIMIT);
        for frame_index in 0..WRITES {
            series.record_measurement(frame_index, frame_index as f64);
        }

        assert_eq!(series.history.len(), HISTORY_LIMIT);
        assert!(series.history.capacity() <= HISTORY_LIMIT);
        assert_eq!(series.history.front().unwrap().frame_index, 4_032);
        assert_eq!(series.history.back().unwrap().frame_index, 4_095);
    }

    #[test]
    fn optimization_wave_20260824b_runtime03_history_eviction_source_contract() {
        let source = include_str!("store.rs");
        let pop = source
            .find("if self.history.len() == self.history_limit")
            .expect("history must check capacity before insertion");
        let push = source
            .find("self.history\n            .push_back")
            .expect("history must append the new measurement");

        assert!(pop < push);
        assert!(!source.contains("while self.history.len() > self.history_limit"));
    }

    #[test]
    #[ignore = "managed release performance evidence"]
    fn optimization_wave_20260824b_runtime03_history_ring_capacity_evidence() {
        const SERIES: usize = 4_096;
        const HISTORY_LIMIT: usize = 64;
        const WRITES: usize = HISTORY_LIMIT * 2;
        const MAX_ELAPSED_NS: u128 = 2_000_000_000;

        let mut legacy = VecDeque::new();
        for frame_index in 0..WRITES {
            legacy.push_back(DiagnosticMeasurement {
                frame_index: frame_index as u64,
                value: frame_index as f64,
            });
            while legacy.len() > HISTORY_LIMIT {
                legacy.pop_front();
            }
        }
        let legacy_capacity_per_series = legacy.capacity();

        let started = Instant::now();
        let mut optimized_capacity_slots = 0usize;
        for _ in 0..SERIES {
            let mut series = DiagnosticSeries::new(HISTORY_LIMIT);
            for frame_index in 0..WRITES {
                series.record_measurement(frame_index as u64, frame_index as f64);
            }
            optimized_capacity_slots =
                optimized_capacity_slots.saturating_add(series.history.capacity());
        }
        let elapsed_ns = started.elapsed().as_nanos();
        let legacy_capacity_slots = legacy_capacity_per_series.saturating_mul(SERIES);
        let capacity_reduction_bps = legacy_capacity_slots
            .saturating_sub(optimized_capacity_slots)
            .saturating_mul(10_000)
            / legacy_capacity_slots;

        println!(
            "RUNTIME_DIAGNOSTIC_HISTORY_BENCH_V1 series={SERIES} history_limit={HISTORY_LIMIT} writes_per_series={WRITES} legacy_capacity_slots={legacy_capacity_slots} optimized_capacity_slots={optimized_capacity_slots} capacity_reduction_bps={capacity_reduction_bps} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}"
        );

        assert!(legacy_capacity_per_series >= HISTORY_LIMIT * 2);
        assert!(optimized_capacity_slots <= SERIES * HISTORY_LIMIT);
        assert!(capacity_reduction_bps >= 5_000);
        assert!(elapsed_ns <= MAX_ELAPSED_NS);
    }
}
