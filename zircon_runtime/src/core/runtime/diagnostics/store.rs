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

    pub fn snapshot(&self) -> DiagnosticStoreSnapshot {
        DiagnosticStoreSnapshot {
            series: self
                .series
                .iter()
                .map(|(path, series)| series.snapshot(path.clone()))
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
        self.current = Some(value);
        self.smoothed = Some(match self.smoothed {
            Some(previous) => previous.mul_add(0.9, value * 0.1),
            None => value,
        });
        self.min = Some(self.min.map_or(value, |current| current.min(value)));
        self.max = Some(self.max.map_or(value, |current| current.max(value)));
        self.history
            .push_back(DiagnosticMeasurement { frame_index, value });
        while self.history.len() > self.history_limit {
            self.history.pop_front();
        }
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
    use super::DiagnosticStore;

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
}
