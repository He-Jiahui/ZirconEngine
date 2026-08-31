use serde::{Deserialize, Serialize};

use super::super::{EditorEventRecord, EditorEventSource};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorEventListenerFilter {
    #[serde(default)]
    pub operation_path_prefixes: Vec<String>,
    #[serde(default)]
    pub operation_groups: Vec<String>,
    #[serde(default)]
    pub sources: Vec<EditorEventSource>,
    #[serde(default = "default_filter_includes_events")]
    pub include_successes: bool,
    #[serde(default = "default_filter_includes_events")]
    pub include_failures: bool,
}

impl Default for EditorEventListenerFilter {
    fn default() -> Self {
        Self {
            operation_path_prefixes: Vec::new(),
            operation_groups: Vec::new(),
            sources: Vec::new(),
            include_successes: true,
            include_failures: true,
        }
    }
}

impl EditorEventListenerFilter {
    pub fn operation_prefix(prefix: impl Into<String>) -> Self {
        let prefix = prefix.into();
        Self {
            operation_path_prefixes: vec![normalize_operation_path_prefix(&prefix)],
            ..Self::default()
        }
    }

    pub fn operation_group(group: impl Into<String>) -> Self {
        Self {
            operation_groups: vec![group.into()],
            ..Self::default()
        }
    }

    pub fn source(source: EditorEventSource) -> Self {
        Self {
            sources: vec![source],
            ..Self::default()
        }
    }

    pub fn with_sources<I>(mut self, sources: I) -> Self
    where
        I: IntoIterator<Item = EditorEventSource>,
    {
        self.sources = sources.into_iter().collect();
        self
    }

    pub fn failures_only(mut self) -> Self {
        self.include_successes = false;
        self.include_failures = true;
        self
    }

    pub fn successes_only(mut self) -> Self {
        self.include_successes = true;
        self.include_failures = false;
        self
    }

    pub fn operation_groups(&self) -> &[String] {
        &self.operation_groups
    }

    pub(super) fn normalized(mut self) -> Self {
        for prefix in &mut self.operation_path_prefixes {
            *prefix = normalize_operation_path_prefix(prefix);
        }
        self.operation_path_prefixes.sort_unstable();
        self.operation_path_prefixes.dedup();
        self.operation_groups.sort_unstable();
        self.operation_groups.dedup();

        let mut seen_sources = [false; 5];
        self.sources.retain(|source| {
            let seen = &mut seen_sources[editor_event_source_index(source)];
            !std::mem::replace(seen, true)
        });
        self
    }

    fn accepts_operation_group(&self, operation_group: &str) -> bool {
        self.operation_groups
            .binary_search_by(|group| group.as_str().cmp(operation_group))
            .is_ok()
    }

    pub(super) fn accepts(&self, record: &EditorEventRecord) -> bool {
        if !self.operation_path_prefixes.is_empty() {
            let Some(operation_id) = record.operation_id.as_deref() else {
                return false;
            };
            if !self
                .operation_path_prefixes
                .iter()
                .any(|prefix| operation_id.starts_with(prefix))
            {
                return false;
            }
        }

        if !self.operation_groups.is_empty() {
            let Some(operation_group) = record.operation_group.as_deref() else {
                return false;
            };
            if !self.accepts_operation_group(operation_group) {
                return false;
            }
        }

        if !self.sources.is_empty() && !self.sources.contains(&record.source) {
            return false;
        }

        if record.result.error.is_some() {
            return self.include_failures;
        }
        self.include_successes
    }
}

fn normalize_operation_path_prefix(prefix: &str) -> String {
    prefix.trim().to_ascii_lowercase()
}

fn editor_event_source_index(source: &EditorEventSource) -> usize {
    match source {
        EditorEventSource::RetainedHost => 0,
        EditorEventSource::Headless => 1,
        EditorEventSource::Cli => 2,
        EditorEventSource::Mcp => 3,
        EditorEventSource::Replay => 4,
    }
}

fn default_filter_includes_events() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn listener_acceptance_does_not_normalize_prefixes_per_record() {
        let source = include_str!("filter.rs");
        let hot_normalization = [
            "operation_id.starts_with(&",
            "normalize_operation_path_prefix(prefix))",
        ]
        .concat();
        assert!(!source.contains(&hot_normalization));
    }

    #[test]
    fn listener_filter_normalizes_operation_prefixes_once() {
        let filter = super::EditorEventListenerFilter::operation_prefix("  Scene.Node  ");
        assert_eq!(filter.operation_path_prefixes, vec!["scene.node"]);
    }

    #[test]
    fn optimization_wave_20260825vw_editor49_listener_filter_is_compiled_once() {
        let filter: super::EditorEventListenerFilter = serde_json::from_value(json!({
            "operation_path_prefixes": ["  Scene.Node  ", "asset", "scene.node"],
            "operation_groups": ["zeta", "alpha", "middle", "alpha"],
            "sources": ["Headless", "Cli", "Headless"]
        }))
        .expect("listener filter should deserialize");
        let filter = filter.normalized();

        assert_eq!(filter.operation_path_prefixes, vec!["asset", "scene.node"]);
        assert_eq!(
            filter
                .operation_groups()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["alpha", "middle", "zeta"]
        );
        assert_eq!(
            filter.sources,
            vec![
                super::EditorEventSource::Headless,
                super::EditorEventSource::Cli
            ]
        );
        assert!(filter.accepts_operation_group("middle"));
        assert!(!filter.accepts_operation_group("missing"));
    }

    #[test]
    fn listener_acceptance_does_not_linearly_scan_operation_groups() {
        let source = include_str!("filter.rs");
        let linear_group_scan = [".operation_groups", "\n                .iter()"].concat();
        assert!(!source.contains(&linear_group_scan));
    }

    #[test]
    #[ignore = "performance evidence; run in the managed Windows release lane"]
    fn optimization_wave_20260825vw_editor49_listener_group_lookup_evidence() {
        use std::hint::black_box;
        use std::time::{Duration, Instant};

        const GROUP_COUNT: usize = 10_000;
        const QUERY_COUNT: usize = 100_000;
        const MAX_ELAPSED: Duration = Duration::from_millis(500);

        let groups = (0..GROUP_COUNT)
            .rev()
            .map(|index| format!("group-{index:05}"))
            .collect::<Vec<_>>();
        let filter: super::EditorEventListenerFilter = serde_json::from_value(json!({
            "operation_groups": groups
        }))
        .expect("listener filter should deserialize");
        let filter = filter.normalized();
        let target = format!("group-{:05}", GROUP_COUNT - 1);

        let started = Instant::now();
        for _ in 0..QUERY_COUNT {
            assert!(black_box(
                filter.accepts_operation_group(black_box(target.as_str()))
            ));
        }
        let elapsed = started.elapsed();

        let legacy_group_comparisons = GROUP_COUNT * QUERY_COUNT;
        let comparisons_per_query_upper_bound =
            usize::BITS as usize - GROUP_COUNT.leading_zeros() as usize;
        let indexed_comparisons_upper_bound = comparisons_per_query_upper_bound * QUERY_COUNT;
        let reduction_basis_points = (legacy_group_comparisons - indexed_comparisons_upper_bound)
            * 10_000
            / legacy_group_comparisons;
        assert!(elapsed <= MAX_ELAPSED, "indexed lookup took {elapsed:?}");
        println!(
            "EDITOR49_LISTENER_FILTER_BENCH_V1 groups={} queries={} legacy_group_comparisons={} indexed_comparisons_upper_bound={} reduction_basis_points={} elapsed_ns={}",
            GROUP_COUNT,
            QUERY_COUNT,
            legacy_group_comparisons,
            indexed_comparisons_upper_bound,
            reduction_basis_points,
            elapsed.as_nanos()
        );
    }
}
