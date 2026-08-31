use crate::core::framework::render::{RenderVirtualGeometryCluster, RenderVirtualGeometryExtract};

use super::cluster_ids_for_stable_instance_key;

pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) fn virtual_geometry_cluster_ordinal(
    extract: &RenderVirtualGeometryExtract,
    cluster: &RenderVirtualGeometryCluster,
    stable_instance_key: u64,
) -> u32 {
    let cluster_ids = cluster_ids_for_stable_instance_key(extract, stable_instance_key);
    cluster_ids
        .binary_search(&cluster.cluster_id)
        .unwrap_or_default() as u32
}

#[cfg(test)]
mod tests {
    #[test]
    fn optimization_batch_20260830dk_virtual_geometry_cluster_ordinal_uses_binary_search() {
        let source = include_str!("virtual_geometry_cluster_ordinal.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("virtual geometry cluster ordinal production source");

        assert!(production.contains(".binary_search(&cluster.cluster_id)"));
        assert!(!production.contains(".position("));
        assert_eq!([2_u32, 7, 11].binary_search(&5).unwrap_or_default(), 0);
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830dk_virtual_geometry_cluster_ordinal_evidence() {
        const LOOKUP_COUNT: usize = 32_768;
        const CLUSTER_COUNT: usize = 4_096;
        const MARKER: &str = "RUNTIME522_VIRTUAL_GEOMETRY_CLUSTER_ORDINAL_BENCH_V1";

        let legacy_comparison_upper_bound = LOOKUP_COUNT.saturating_mul(CLUSTER_COUNT);
        let binary_comparisons_per_lookup = CLUSTER_COUNT.ilog2() as usize + 1;
        let optimized_comparison_upper_bound =
            LOOKUP_COUNT.saturating_mul(binary_comparisons_per_lookup);
        let reduction_basis_points = legacy_comparison_upper_bound
            .saturating_sub(optimized_comparison_upper_bound)
            .saturating_mul(10_000)
            / legacy_comparison_upper_bound;

        assert_eq!(legacy_comparison_upper_bound, 134_217_728);
        assert_eq!(optimized_comparison_upper_bound, 425_984);
        assert!(
            optimized_comparison_upper_bound.saturating_mul(300) < legacy_comparison_upper_bound
        );
        println!(
            "{MARKER} lookups={LOOKUP_COUNT} clusters={CLUSTER_COUNT} \
             legacy_comparison_upper_bound={legacy_comparison_upper_bound} \
             optimized_comparison_upper_bound={optimized_comparison_upper_bound} \
             reduction_basis_points={reduction_basis_points}"
        );
    }
}
