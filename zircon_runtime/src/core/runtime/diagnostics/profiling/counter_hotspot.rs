use std::collections::{BTreeSet, HashMap};

use zircon_runtime_interface::{
    CounterHotspotEntry, CounterHotspotReport, ProfileCounterSnapshot, ProfileSnapshot,
};

/// 按 stream/name 汇总有限正值样本；frame_count 只统计带帧号的不同观测帧。
/// 聚合结果用于定位采样集中处，不能单凭计数判定任务或 GPU 工作已经完成。
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct CounterHotspotKey<'a> {
    stream: &'a str,
    name: &'a str,
}

impl<'a> From<&'a ProfileCounterSnapshot> for CounterHotspotKey<'a> {
    fn from(counter: &'a ProfileCounterSnapshot) -> Self {
        Self {
            stream: counter.stream.as_str(),
            name: counter.name.as_str(),
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

    fn finish(mut self, key: CounterHotspotKey<'_>) -> CounterHotspotEntry {
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
            stream: key.stream.to_owned(),
            name: key.name.to_owned(),
            path: format!("{}/counter:{}", key.stream, key.name),
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
#[path = "tests/counter_hotspot.rs"]
mod tests;
