use std::collections::{HashMap, HashSet};

use zircon_runtime_interface::{HotspotEntry, HotspotReport, ProfileSnapshot, ProfileSpanSnapshot};

pub fn analyze_hotspots(snapshot: &ProfileSnapshot) -> HotspotReport {
    let budget_us = (snapshot.frame_budget_ms.max(0.0) * 1_000.0) as u64;
    let mut groups: HashMap<HotspotKey<'_>, HotspotAccumulator> = HashMap::new();
    for span in &snapshot.spans {
        groups
            .entry(HotspotKey::from(span))
            .or_default()
            .push(span, budget_us);
    }

    let mut hotspots = groups
        .into_iter()
        .map(|(key, accumulator)| accumulator.finish(key))
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
struct HotspotKey<'a> {
    stream: &'a str,
    category: &'a str,
    name: &'a str,
    path: &'a str,
}

impl<'a> From<&'a ProfileSpanSnapshot> for HotspotKey<'a> {
    fn from(span: &'a ProfileSpanSnapshot) -> Self {
        Self {
            stream: span.stream.as_str(),
            category: span.category.as_str(),
            name: span.name.as_str(),
            path: span.path.as_str(),
        }
    }
}

#[derive(Default)]
struct HotspotAccumulator {
    durations: Vec<u64>,
    frames: HashSet<u64>,
    total_us: u64,
    max_us: u64,
    over_budget_count: u64,
}

impl HotspotAccumulator {
    fn push(&mut self, span: &ProfileSpanSnapshot, budget_us: u64) {
        self.durations.push(span.duration_us);
        self.total_us += span.duration_us;
        self.max_us = self.max_us.max(span.duration_us);
        if span.duration_us > budget_us {
            self.over_budget_count += 1;
        }
        if let Some(frame) = span.frame_index {
            self.frames.insert(frame);
        }
    }

    fn finish(mut self, key: HotspotKey<'_>) -> HotspotEntry {
        let count = self.durations.len() as u64;
        let avg_us = if count == 0 { 0 } else { self.total_us / count };
        let p95_us = percentile(&mut self.durations, 95);
        HotspotEntry {
            stream: key.stream.to_owned(),
            category: key.category.to_owned(),
            name: key.name.to_owned(),
            path: key.path.to_owned(),
            total_us: self.total_us,
            avg_us,
            p95_us,
            max_us: self.max_us,
            count,
            frame_count: self.frames.len() as u64,
            over_budget_count: self.over_budget_count,
        }
    }
}

fn percentile(values: &mut [u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let index = ((values.len() - 1) * percentile).div_ceil(100);
    let (_, selected, _) = values.select_nth_unstable(index.min(values.len() - 1));
    *selected
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

    use super::{analyze_hotspots, percentile};

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

    #[test]
    fn percentile_selection_matches_the_sorted_order_statistic() {
        for mut values in [
            Vec::new(),
            vec![9],
            vec![9, 1],
            vec![8, 3, 5, 1, 9, 2, 7, 4, 6],
            (0..101).rev().collect::<Vec<_>>(),
        ] {
            let mut sorted = values.clone();
            sorted.sort_unstable();
            let expected = if sorted.is_empty() {
                0
            } else {
                let index = ((sorted.len() - 1) * 95).div_ceil(100);
                sorted[index]
            };

            assert_eq!(percentile(&mut values, 95), expected);
        }
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
