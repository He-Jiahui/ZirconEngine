use std::collections::HashSet;

use super::editor_subsystems::EditorSubsystemReport;
use super::minimal_host_contract::EditorHostMinimalReport;

fn sorted_unique_capabilities(capabilities: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut capabilities = capabilities.into_iter().collect::<HashSet<_>>();
    let mut capabilities = capabilities.drain().collect::<Vec<_>>();
    capabilities.sort_unstable();
    capabilities
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorCapabilitySnapshot {
    enabled_capabilities: Vec<String>,
    disabled_capabilities: Vec<String>,
    diagnostics: Vec<String>,
}

impl EditorCapabilitySnapshot {
    pub(crate) fn from_reports(
        minimal: &EditorHostMinimalReport,
        subsystems: &EditorSubsystemReport,
    ) -> Self {
        let enabled_capabilities = sorted_unique_capabilities(
            minimal
                .loaded_capabilities()
                .into_iter()
                .chain(subsystems.enabled_subsystems().iter().cloned()),
        );

        Self {
            enabled_capabilities,
            disabled_capabilities: subsystems.disabled_subsystems().to_vec(),
            diagnostics: subsystems.diagnostics().to_vec(),
        }
    }

    pub fn enabled_capabilities(&self) -> &[String] {
        &self.enabled_capabilities
    }

    pub fn disabled_capabilities(&self) -> &[String] {
        &self.disabled_capabilities
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn is_enabled(&self, capability: &str) -> bool {
        self.enabled_capabilities
            .binary_search_by(|enabled| enabled.as_str().cmp(capability))
            .is_ok()
    }

    pub(crate) fn allows_all(&self, capabilities: &[String]) -> bool {
        capabilities
            .iter()
            .all(|capability| self.is_enabled(capability))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const CAPABILITY_COUNT: usize = 65_536;
    const UNIQUE_CAPABILITY_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn capabilities() -> Vec<String> {
        (0..CAPABILITY_COUNT)
            .map(|index| {
                format!(
                    "editor.capability.{:04}",
                    (index * 4_099) % UNIQUE_CAPABILITY_COUNT
                )
            })
            .collect()
    }

    fn legacy_capability_union(capabilities: &[String]) -> Vec<String> {
        capabilities
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn optimized_capability_union(capabilities: &[String]) -> Vec<String> {
        let mut unique = capabilities.iter().cloned().collect::<HashSet<_>>();
        let mut unique = unique.drain().collect::<Vec<_>>();
        unique.sort_unstable();
        unique
    }

    #[test]
    fn enabled_capability_lookup_uses_the_sorted_snapshot() {
        let source = include_str!("editor_capabilities.rs");
        let sorted_lookup = ["binary", "_search_by"].concat();

        assert!(source.contains(&sorted_lookup));
    }

    #[test]
    fn optimization_batch_20260826q_editor06_hash_union_preserves_sorted_unique_capabilities() {
        let capabilities = vec![
            "viewport".to_string(),
            "assets".to_string(),
            "viewport".to_string(),
            "console".to_string(),
        ];

        assert_eq!(
            sorted_unique_capabilities(capabilities),
            vec![
                "assets".to_string(),
                "console".to_string(),
                "viewport".to_string()
            ]
        );
    }

    #[test]
    fn optimization_batch_20260826q_editor06_capability_snapshot_uses_hash_union() {
        let source = include_str!("editor_capabilities.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("collect::<HashSet<_>>()"));
        assert!(production.contains("capabilities.sort_unstable();"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826q_editor06_capability_hash_union_performance_evidence() {
        let capabilities = capabilities();
        let expected = legacy_capability_union(&capabilities);
        assert_eq!(expected.len(), UNIQUE_CAPABILITY_COUNT);
        assert_eq!(optimized_capability_union(&capabilities), expected);

        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(legacy_capability_union(black_box(&capabilities)));
                legacy_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(optimized_capability_union(black_box(&capabilities)));
                optimized_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(optimized_capability_union(black_box(&capabilities)));
                optimized_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(legacy_capability_union(black_box(&capabilities)));
                legacy_samples.push(started.elapsed());
            }
        }

        let legacy_p95 = percentile_95(&mut legacy_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "EDITOR06_CAPABILITY_SNAPSHOT_HASH_UNION_BENCH_V1 capabilities={CAPABILITY_COUNT} \
             unique_capabilities={UNIQUE_CAPABILITY_COUNT} ordered_admissions={CAPABILITY_COUNT} \
             hash_admissions={CAPABILITY_COUNT} sorted_values={UNIQUE_CAPABILITY_COUNT} \
             legacy_p95_ns={} optimized_p95_ns={}",
            legacy_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60,
            "hash-union P95 {:?} exceeded 60% of ordered-union P95 {:?}",
            optimized_p95,
            legacy_p95,
        );
    }
}
