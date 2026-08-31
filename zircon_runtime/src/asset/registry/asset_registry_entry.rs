use std::collections::{BTreeSet, HashSet};

use serde::{Deserialize, Serialize};

use crate::asset::{AssetKind, AssetUri, AssetUuid};

/// Metadata required for discovery and dependency queries without loading asset payloads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRegistryEntry {
    uuid: AssetUuid,
    path: AssetUri,
    type_marker: AssetKind,
    tags: BTreeSet<String>,
    dependencies: Vec<AssetUuid>,
    source_digest: String,
}

impl AssetRegistryEntry {
    pub fn new(
        uuid: AssetUuid,
        path: AssetUri,
        type_marker: AssetKind,
        source_digest: impl Into<String>,
    ) -> Self {
        Self {
            uuid,
            path,
            type_marker,
            tags: BTreeSet::new(),
            dependencies: Vec::new(),
            source_digest: source_digest.into(),
        }
    }

    pub fn with_tags(mut self, tags: BTreeSet<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<AssetUuid>) -> Self {
        self.dependencies = unique_dependencies(dependencies);
        self
    }

    pub fn uuid(&self) -> AssetUuid {
        self.uuid
    }

    pub fn path(&self) -> &AssetUri {
        &self.path
    }

    pub fn type_marker(&self) -> AssetKind {
        self.type_marker
    }

    pub fn tags(&self) -> &BTreeSet<String> {
        &self.tags
    }

    pub fn dependencies(&self) -> &[AssetUuid] {
        &self.dependencies
    }

    pub fn source_digest(&self) -> &str {
        &self.source_digest
    }

    pub(super) fn set_dependencies(&mut self, dependencies: Vec<AssetUuid>) {
        self.dependencies = unique_dependencies(dependencies);
    }
}

fn unique_dependencies(dependencies: Vec<AssetUuid>) -> Vec<AssetUuid> {
    if dependencies.len() <= 1 {
        return dependencies;
    }
    let mut unique = Vec::with_capacity(dependencies.len());
    let mut seen = HashSet::with_capacity(dependencies.len());
    for dependency in dependencies {
        if seen.insert(dependency) {
            unique.push(dependency);
        }
    }
    unique
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::unique_dependencies;
    use crate::asset::AssetUuid;

    const DEPENDENCIES_PER_SAMPLE: usize = 65_536;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_fu_runtime477_single_dependency_keeps_input_without_hash_set() {
        let dependency = AssetUuid::new();
        let dependencies = unique_dependencies(vec![dependency]);

        assert_eq!(dependencies, vec![dependency]);
        assert_eq!(dependencies.capacity(), 1);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fu_runtime477_single_dependency_allocation_benchmark() {
        let dependency = AssetUuid::new();
        for _ in 0..4 {
            black_box(measure_single_dependencies(dependency, false));
            black_box(measure_single_dependencies(dependency, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_single_dependencies(dependency, false));
                optimized_samples.push(measure_single_dependencies(dependency, true));
            } else {
                optimized_samples.push(measure_single_dependencies(dependency, true));
                legacy_samples.push(measure_single_dependencies(dependency, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME477_SINGLE_DEPENDENCY_ALLOCATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} dependencies_per_sample={DEPENDENCIES_PER_SAMPLE} legacy_hash_set_allocations_per_dependency=1 optimized_hash_set_allocations_per_dependency=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 75 / 100);
    }

    fn measure_single_dependencies(dependency: AssetUuid, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..DEPENDENCIES_PER_SAMPLE {
            let input = vec![black_box(dependency)];
            let output = if optimized {
                unique_dependencies(input)
            } else {
                let mut unique = Vec::with_capacity(input.len());
                let mut seen = std::collections::HashSet::with_capacity(input.len());
                for dependency in input {
                    if seen.insert(dependency) {
                        unique.push(dependency);
                    }
                }
                unique
            };
            black_box(output);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
