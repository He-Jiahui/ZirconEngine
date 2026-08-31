from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
SPARSE_SOURCE = (
    REPO_ROOT
    / "zircon_runtime/src/scene/ecs/storage/component_storage/sparse.rs"
)
SPARSE_TESTS = (
    REPO_ROOT
    / "zircon_runtime/src/scene/ecs/storage/component_storage/sparse/tests.rs"
)
LOCATOR_SOURCE = (
    REPO_ROOT
    / "zircon_runtime/src/scene/ecs/storage/component_storage/sparse/locator.rs"
)
STATUS = "runtime_08_60_sparse_component_locator_algorithm_source_passed_diagnostics_cargo_product_profile_deferred"
STATUS_DOCS = (
    "docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md",
    "docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-ecs-archetype-columnar-storage.md",
    "docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md",
    "docs/plans/optimize/zircon_runtime/60/2026-08-28-sparse-component-locator-pages.md",
)


class RuntimeSparseComponentLocatorPagesContractTests(unittest.TestCase):
    def test_sparse_locator_is_bounded_and_page_backed(self) -> None:
        source = SPARSE_SOURCE.read_text(encoding="utf-8")
        locator = LOCATOR_SOURCE.read_text(encoding="utf-8")

        for anchor in (
            '#[path = "sparse/locator.rs"]',
            "SparseRowLocation::new(entity.generation(), dense_row)",
        ):
            self.assertIn(anchor, source)

        required = (
            "const SPARSE_LOCATOR_PAGE_BITS: u32 = 8;",
            "pub(super) const SPARSE_LOCATOR_PAGE_SLOTS: usize",
            "const SPARSE_LOCATOR_PREFIX_PROMOTION_FACTOR: usize = 1_024;",
            "const SPARSE_LOCATOR_PREFIX_DEMOTION_FACTOR: usize = 2_048;",
            "struct SparseRowLocator",
            "flat_prefix: Vec<Option<SparseRowLocation>>",
            "flat_window: Vec<Option<SparseRowLocation>>",
            "sparse_pages: HashMap<u32, Box<SparseLocatorPage>, SparseLocatorBuildHasher>",
            "sparse_page_keys: BTreeSet<u32>",
            "fn promote_qualified_prefix(&mut self)",
            "fn promote_qualified_window(&mut self, index: u32)",
            "fn try_rebase_flat_prefix_as_window(&mut self) -> bool",
            "fn demote_flat_prefix(&mut self)",
            "fn demote_flat_window(&mut self)",
            "fn split_locator_index(index: u32) -> (u32, usize)",
            "Keys are private u32 page indices derived from EntityRegistry-issued handles.",
        )
        for anchor in required:
            self.assertIn(anchor, locator)

        combined_source = source + locator
        for retired in (
            "direct_pages",
            "overflow_pages",
            "SPARSE_LOCATOR_DIRECT_",
            "sparse_page_queue",
            "sparse_rows.resize",
        ):
            self.assertNotIn(retired, combined_source)

    def test_sparse_locator_behavior_regressions_are_mounted(self) -> None:
        source = SPARSE_SOURCE.read_text(encoding="utf-8")
        tests = SPARSE_TESTS.read_text(encoding="utf-8")

        self.assertIn('#[path = "sparse/tests.rs"]', source)
        for anchor in (
            "highest_valid_entity_index_allocates_one_locator_page",
            "removing_the_last_row_retires_the_locator_hierarchy",
            "removing_one_row_keeps_the_shared_locator_page_alive",
            "density_bound_promotes_across_empty_low_pages",
            "distant_page_promotes_only_when_the_global_density_bound_reaches_it",
            "growing_high_window_is_absorbed_when_the_zero_prefix_reaches_it",
            "recreated_sparse_page_is_promoted_once_after_key_reinsertion",
            "low_density_prefix_rebases_the_retained_cluster_as_a_flat_window",
            "low_density_high_window_trims_empty_leading_pages_before_demotion",
            "widely_separated_window_rows_demote_to_bounded_sparse_pages",
            "deleting_window_support_rechecks_an_under_dense_prefix",
            "deleting_prefix_support_rechecks_an_under_dense_window",
            "a_third_distant_cluster_remains_in_bounded_sparse_overflow",
            "generation_checks_and_swap_remove_repair_survive_page_boundaries",
            "packed_locator_uses_eight_bytes_per_allocated_slot",
            "locator_matches_a_reference_map_during_mixed_operations",
        ):
            self.assertIn(anchor, tests)

    def test_sparse_locator_status_is_mirrored_by_canonical_plans(self) -> None:
        for relative_path in STATUS_DOCS:
            source = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertIn(STATUS, source, relative_path)


if __name__ == "__main__":
    unittest.main()
