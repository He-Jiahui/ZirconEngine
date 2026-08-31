use std::collections::HashSet;

use crate::core::resource::ResourceMutationBatch;

use crate::asset::project::ProjectManager;
use crate::asset::AssetUri;

pub(in crate::asset::pipeline::manager) fn clear_removed_project_resources(
    mut batch: ResourceMutationBatch,
    previous_locators: &HashSet<AssetUri>,
    project: &ProjectManager,
) -> ResourceMutationBatch {
    let current = collect_project_locator_refs(
        project
            .registry()
            .values()
            .map(|metadata| metadata.primary_locator()),
    );
    for locator in previous_locators {
        if !current.contains(locator) {
            batch = batch.remove(locator.clone());
        }
    }
    batch
}

fn collect_project_locator_refs<'a>(
    locators: impl IntoIterator<Item = &'a AssetUri>,
) -> HashSet<&'a AssetUri> {
    locators.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::hint::black_box;
    use std::time::Instant;

    use crate::asset::AssetUri;

    use super::collect_project_locator_refs;

    #[test]
    fn optimization_batch_dw_project_locator_index_borrows_source_storage() {
        let locators = project_locators_fixture(8);

        let indexed = collect_project_locator_refs(locators.iter());
        let first = indexed.get(&locators[0]).expect("first locator indexed");

        assert_eq!(indexed.len(), locators.len());
        assert!(std::ptr::eq(*first, &locators[0]));
    }

    #[test]
    fn optimization_batch_dw_project_locator_index_preserves_removed_membership() {
        let previous = project_locators_fixture(8)
            .into_iter()
            .collect::<HashSet<_>>();
        let current_locators = project_locators_fixture(6);
        let current = collect_project_locator_refs(current_locators.iter());
        let removed = previous
            .iter()
            .filter(|locator| !current.contains(*locator))
            .collect::<Vec<_>>();

        assert_eq!(removed.len(), 2);
        assert!(removed.iter().all(|locator| {
            locator.path().ends_with("asset_0006.bin") || locator.path().ends_with("asset_0007.bin")
        }));
    }

    #[test]
    fn optimization_batch_dw_project_locator_index_avoids_locator_clones() {
        let production = include_str!("clear_removed_project_resources.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("project resource sync production source");
        let collection = production
            .split("fn collect_project_locator_refs")
            .nth(1)
            .expect("borrowed project locator collector");

        assert!(collection.contains("HashSet<&'a AssetUri>"));
        assert!(!collection.contains("clone()"));
        assert!(!collection.contains("to_owned()"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dw_borrowed_project_locator_index_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const COLLECTIONS_PER_SAMPLE: usize = 128;
        const LOCATORS_PER_PROJECT: usize = 1_024;

        let locators = project_locators_fixture(LOCATORS_PER_PROJECT);
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_locator_collections(
                    &locators,
                    COLLECTIONS_PER_SAMPLE,
                    false,
                ));
                optimized_samples.push(measure_locator_collections(
                    &locators,
                    COLLECTIONS_PER_SAMPLE,
                    true,
                ));
            } else {
                optimized_samples.push(measure_locator_collections(
                    &locators,
                    COLLECTIONS_PER_SAMPLE,
                    true,
                ));
                legacy_samples.push(measure_locator_collections(
                    &locators,
                    COLLECTIONS_PER_SAMPLE,
                    false,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME431_BORROWED_PROJECT_LOCATOR_INDEX_BENCH_V1 collections_per_sample={COLLECTIONS_PER_SAMPLE} locators_per_project={LOCATORS_PER_PROJECT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "borrowed project locator index p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn project_locators_fixture(count: usize) -> Vec<AssetUri> {
        (0..count)
            .map(|index| {
                AssetUri::parse(&format!(
                    "res://catalog/{}/asset_{index:04}.bin",
                    "long_project_segment/".repeat(8)
                ))
                .expect("valid project locator")
            })
            .collect()
    }

    fn measure_locator_collections(
        locators: &[AssetUri],
        collection_count: usize,
        optimized: bool,
    ) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..collection_count {
            if optimized {
                let indexed = collect_project_locator_refs(locators.iter());
                checksum = checksum.wrapping_add(indexed.len());
                black_box(indexed);
            } else {
                let indexed = locators.iter().cloned().collect::<HashSet<_>>();
                checksum = checksum.wrapping_add(indexed.len());
                black_box(indexed);
            }
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
