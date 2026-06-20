use std::collections::{BTreeSet, HashMap};

use zircon_runtime_interface::{
    CounterHotspotEntry, CounterHotspotReport, ProfileCounterSnapshot, ProfileSnapshot,
};

pub fn analyze_counter_hotspots(snapshot: &ProfileSnapshot) -> CounterHotspotReport {
    let mut groups: HashMap<CounterHotspotKey, CounterHotspotAccumulator> = HashMap::new();
    let mut accepted_counter_count = 0;
    for counter in &snapshot.counters {
        if !counter.value.is_finite() || counter.value <= 0.0 {
            continue;
        }
        accepted_counter_count += 1;
        groups
            .entry(CounterHotspotKey::from(counter))
            .or_default()
            .push(counter);
    }

    let mut counters = groups
        .into_iter()
        .map(|(key, accumulator)| accumulator.finish(key))
        .collect::<Vec<_>>();
    counters.sort_by(|left, right| {
        right
            .total
            .total_cmp(&left.total)
            .then_with(|| right.p95.total_cmp(&left.p95))
            .then_with(|| left.path.cmp(&right.path))
    });

    CounterHotspotReport {
        session_id: snapshot.session_id.clone(),
        frame_budget_ms: snapshot.frame_budget_ms,
        generated_from_counter_count: accepted_counter_count,
        hints: counter_hints(&counters),
        counters,
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct CounterHotspotKey {
    stream: String,
    name: String,
    path: String,
}

impl From<&ProfileCounterSnapshot> for CounterHotspotKey {
    fn from(counter: &ProfileCounterSnapshot) -> Self {
        Self {
            stream: counter.stream.clone(),
            name: counter.name.clone(),
            path: format!("{}/counter:{}", counter.stream, counter.name),
        }
    }
}

#[derive(Default)]
struct CounterHotspotAccumulator {
    values: Vec<f64>,
    frames: BTreeSet<u64>,
    latest: Option<(u64, f64)>,
}

impl CounterHotspotAccumulator {
    fn push(&mut self, counter: &ProfileCounterSnapshot) {
        self.values.push(counter.value);
        if let Some(frame) = counter.frame_index {
            self.frames.insert(frame);
        }
        if self
            .latest
            .map(|(timestamp, _)| counter.timestamp_us >= timestamp)
            .unwrap_or(true)
        {
            self.latest = Some((counter.timestamp_us, counter.value));
        }
    }

    fn finish(mut self, key: CounterHotspotKey) -> CounterHotspotEntry {
        self.values.sort_by(f64::total_cmp);
        let count = self.values.len() as u64;
        let total = self.values.iter().sum::<f64>();
        let avg = if count == 0 {
            0.0
        } else {
            total / count as f64
        };
        let max = self.values.last().copied().unwrap_or(0.0);
        let p95 = percentile(&self.values, 95);
        let latest = self.latest.map(|(_, value)| value).unwrap_or(0.0);
        CounterHotspotEntry {
            stream: key.stream,
            name: key.name,
            path: key.path,
            total,
            avg,
            p95,
            max,
            latest,
            count,
            frame_count: self.frames.len() as u64,
        }
    }
}

fn percentile(sorted: &[f64], percentile: usize) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = ((sorted.len() - 1) * percentile).div_ceil(100);
    sorted[index.min(sorted.len() - 1)]
}

fn counter_hints(counters: &[CounterHotspotEntry]) -> Vec<String> {
    counters
        .iter()
        .take(5)
        .map(|entry| {
            format!(
                "{} accumulated {:.2} over {} samples; use this counter evidence with adjacent frame spans before opening an optimization slice.",
                entry.path, entry.total, entry.count
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{ProfileCounterSnapshot, ProfileSnapshot};

    use super::analyze_counter_hotspots;

    #[test]
    fn counter_hotspots_group_sort_and_track_latest() {
        let mut snapshot = ProfileSnapshot {
            session_id: "counter-test".to_string(),
            frame_budget_ms: 16.67,
            ..ProfileSnapshot::default()
        };
        snapshot.counters = vec![
            counter("runtime", "extract.rebuild_clones", 1.0, 10, Some(0)),
            counter("runtime", "extract.rebuild_clones", 2.0, 20, Some(1)),
            counter("runtime", "asset.worker.frame_completed", 4.0, 15, Some(1)),
            counter("runtime", "ignored.zero", 0.0, 30, Some(2)),
            counter("runtime", "ignored.nan", f64::NAN, 40, Some(2)),
        ];

        let report = analyze_counter_hotspots(&snapshot);

        assert_eq!(report.generated_from_counter_count, 3);
        assert_eq!(report.counters.len(), 2);
        assert_eq!(
            report.counters[0].path,
            "runtime/counter:asset.worker.frame_completed"
        );
        assert_eq!(report.counters[0].total, 4.0);
        assert_eq!(
            report.counters[1].path,
            "runtime/counter:extract.rebuild_clones"
        );
        assert_eq!(report.counters[1].count, 2);
        assert_eq!(report.counters[1].frame_count, 2);
        assert_eq!(report.counters[1].latest, 2.0);
        assert!(report
            .hints
            .iter()
            .any(|hint| hint.contains("runtime/counter:asset.worker.frame_completed")));
    }

    fn counter(
        stream: &str,
        name: &str,
        value: f64,
        timestamp_us: u64,
        frame_index: Option<u64>,
    ) -> ProfileCounterSnapshot {
        ProfileCounterSnapshot {
            stream: stream.to_string(),
            name: name.to_string(),
            value,
            timestamp_us,
            frame_index,
        }
    }
}
