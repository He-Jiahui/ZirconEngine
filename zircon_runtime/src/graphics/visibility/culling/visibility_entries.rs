use crate::core::framework::render::RenderFrameExtract;

pub(crate) fn visibility_mesh_indices(extract: &RenderFrameExtract) -> Vec<usize> {
    let mut indices = (0..extract.geometry.meshes.len()).collect::<Vec<_>>();
    sort_visibility_indices_by_stable_key(&mut indices, |index| {
        extract.geometry.meshes[index].stable_instance_key
    });
    indices
}

fn sort_visibility_indices_by_stable_key(indices: &mut [usize], stable_key: impl Fn(usize) -> u64) {
    indices.sort_unstable_by_key(|index| (stable_key(*index), *index));
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::sort_visibility_indices_by_stable_key;

    #[test]
    fn visibility_entries_order_meshes_by_stable_instance_key() {
        let source = include_str!("visibility_entries.rs");

        assert!(source.contains("visibility_mesh_indices"));
        assert!(source.contains("stable_instance_key"));
        assert!(!source.contains("HashMap"));
    }

    #[test]
    fn optimization_batch_da_visibility_index_sort_preserves_stable_ties() {
        let keys = [30, 10, 30, 10, 20, 10];
        let mut expected = (0..keys.len()).collect::<Vec<_>>();
        expected.sort_by_key(|index| keys[*index]);
        let mut actual = (0..keys.len()).collect::<Vec<_>>();

        sort_visibility_indices_by_stable_key(&mut actual, |index| keys[index]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn optimization_batch_da_visibility_index_sort_uses_total_unstable_key() {
        let source = include_str!("visibility_entries.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("sort_unstable_by_key"));
        assert!(production.contains("(stable_key(*index), *index)"));
        assert!(!production.contains("indices.sort_by_key"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_da_visibility_index_unstable_sort_p95() {
        const INDEX_COUNT: usize = 65_536;
        const SAMPLE_COUNT: usize = 17;
        let keys = (0..INDEX_COUNT)
            .map(|index| (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15))
            .collect::<Vec<_>>();
        let template = (0..INDEX_COUNT).rev().collect::<Vec<_>>();

        let (legacy_samples, optimized_samples) = paired_samples::<SAMPLE_COUNT>(&template, &keys);
        assert_eq!(
            legacy_sort(&template, &keys),
            optimized_sort(&template, &keys)
        );

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        println!(
            "PERF_RESULT RUNTIME404_VISIBILITY_INDEX_UNSTABLE_SORT_BENCH_V1 indices={INDEX_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
        );
        assert!(
            optimized_p95 * 10 <= legacy_p95 * 7,
            "optimized P95 {optimized_p95}ns must be no more than 70% of legacy P95 {legacy_p95}ns"
        );
    }

    fn legacy_sort(template: &[usize], keys: &[u64]) -> Vec<usize> {
        let mut indices = template.to_vec();
        indices.sort_by_key(|index| keys[*index]);
        indices
    }

    fn optimized_sort(template: &[usize], keys: &[u64]) -> Vec<usize> {
        let mut indices = template.to_vec();
        sort_visibility_indices_by_stable_key(&mut indices, |index| keys[index]);
        indices
    }

    fn paired_samples<const SAMPLE_COUNT: usize>(
        template: &[usize],
        keys: &[u64],
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy_sort(template, keys));
        black_box(optimized_sort(template, keys));
        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample_index in 0..SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(sample_sort(template, |indices| {
                    indices.sort_by_key(|index| keys[*index]);
                }));
                optimized_samples.push(sample_sort(template, |indices| {
                    sort_visibility_indices_by_stable_key(indices, |index| keys[index]);
                }));
            } else {
                optimized_samples.push(sample_sort(template, |indices| {
                    sort_visibility_indices_by_stable_key(indices, |index| keys[index]);
                }));
                legacy_samples.push(sample_sort(template, |indices| {
                    indices.sort_by_key(|index| keys[*index]);
                }));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn sample_sort(template: &[usize], operation: impl FnOnce(&mut [usize])) -> u128 {
        let mut indices = template.to_vec();
        let started = Instant::now();
        operation(black_box(&mut indices));
        let elapsed = started.elapsed().as_nanos();
        black_box(indices);
        elapsed
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        sorted[(sorted.len() * percentile).div_ceil(100) - 1]
    }
}
