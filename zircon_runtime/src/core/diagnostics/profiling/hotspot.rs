use std::collections::{BTreeSet, HashMap};

use zircon_runtime_interface::{HotspotEntry, HotspotReport, ProfileSnapshot, ProfileSpanSnapshot};

pub fn analyze_hotspots(snapshot: &ProfileSnapshot) -> HotspotReport {
    let mut groups: HashMap<HotspotKey, HotspotAccumulator> = HashMap::new();
    for span in &snapshot.spans {
        groups.entry(HotspotKey::from(span)).or_default().push(span);
    }

    let mut hotspots = groups
        .into_iter()
        .map(|(key, accumulator)| accumulator.finish(key, snapshot.frame_budget_ms))
        .collect::<Vec<_>>();
    hotspots.sort_by(|left, right| {
        right
            .total_us
            .cmp(&left.total_us)
            .then_with(|| right.p95_us.cmp(&left.p95_us))
            .then_with(|| left.path.cmp(&right.path))
    });

    HotspotReport {
        session_id: snapshot.session_id.clone(),
        frame_budget_ms: snapshot.frame_budget_ms,
        generated_from_span_count: snapshot.spans.len(),
        hints: optimization_hints(&hotspots, snapshot.frame_budget_ms),
        hotspots,
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct HotspotKey {
    stream: String,
    category: String,
    name: String,
    path: String,
}

impl From<&ProfileSpanSnapshot> for HotspotKey {
    fn from(span: &ProfileSpanSnapshot) -> Self {
        Self {
            stream: span.stream.clone(),
            category: span.category.clone(),
            name: span.name.clone(),
            path: span.path.clone(),
        }
    }
}

#[derive(Default)]
struct HotspotAccumulator {
    durations: Vec<u64>,
    frames: BTreeSet<u64>,
}

impl HotspotAccumulator {
    fn push(&mut self, span: &ProfileSpanSnapshot) {
        self.durations.push(span.duration_us);
        if let Some(frame) = span.frame_index {
            self.frames.insert(frame);
        }
    }

    fn finish(mut self, key: HotspotKey, budget_ms: f64) -> HotspotEntry {
        self.durations.sort_unstable();
        let count = self.durations.len() as u64;
        let total_us = self.durations.iter().sum::<u64>();
        let avg_us = if count == 0 { 0 } else { total_us / count };
        let max_us = self.durations.last().copied().unwrap_or(0);
        let p95_us = percentile(&self.durations, 95);
        let budget_us = (budget_ms.max(0.0) * 1_000.0) as u64;
        let over_budget_count = self
            .durations
            .iter()
            .filter(|duration| **duration > budget_us)
            .count() as u64;
        HotspotEntry {
            stream: key.stream,
            category: key.category,
            name: key.name,
            path: key.path,
            total_us,
            avg_us,
            p95_us,
            max_us,
            count,
            frame_count: self.frames.len() as u64,
            over_budget_count,
        }
    }
}

fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let index = ((sorted.len() - 1) * percentile).div_ceil(100);
    sorted[index.min(sorted.len() - 1)]
}

fn optimization_hints(hotspots: &[HotspotEntry], budget_ms: f64) -> Vec<String> {
    let budget_us = (budget_ms.max(0.0) * 1_000.0) as u64;
    hotspots
        .iter()
        .take(5)
        .filter_map(|entry| {
            if entry.p95_us > budget_us {
                Some(format!(
                    "{} p95 {:.2}ms exceeds {:.2}ms frame budget; inspect recorded `{}` spans first.",
                    entry.stream,
                    entry.p95_us as f64 / 1_000.0,
                    budget_ms,
                    entry.name
                ))
            } else if entry.total_us > budget_us {
                Some(format!(
                    "{} accumulates {:.2}ms in `{}`; compare sibling spans before optimizing.",
                    entry.stream,
                    entry.total_us as f64 / 1_000.0,
                    entry.name
                ))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{ProfileSnapshot, ProfileSpanSnapshot};

    use super::analyze_hotspots;

    #[test]
    fn hotspots_sort_by_total_then_p95() {
        let mut snapshot = ProfileSnapshot {
            session_id: "test".to_string(),
            frame_budget_ms: 16.67,
            ..ProfileSnapshot::default()
        };
        snapshot.spans = vec![
            span("runtime", "render", "submit", 0, 10_000),
            span("runtime", "render", "submit", 1, 20_000),
            span("editor", "ui", "tick", 1, 5_000),
        ];

        let report = analyze_hotspots(&snapshot);

        assert_eq!(report.hotspots[0].stream, "runtime");
        assert_eq!(report.hotspots[0].total_us, 30_000);
        assert_eq!(report.hotspots[0].frame_count, 2);
        assert_eq!(report.hotspots[0].p95_us, 20_000);
    }

    fn span(
        stream: &str,
        category: &str,
        name: &str,
        frame_index: u64,
        duration_us: u64,
    ) -> ProfileSpanSnapshot {
        ProfileSpanSnapshot {
            id: frame_index + 1,
            parent_id: None,
            frame_index: Some(frame_index),
            stream: stream.to_string(),
            category: category.to_string(),
            name: name.to_string(),
            path: format!("{stream}/{category}:{name}"),
            start_us: 0,
            duration_us,
            depth: 0,
        }
    }
}
