
from tools.tests.rust_test_files import read_rust_test_file
from pathlib import Path
import unittest

import sys as _history_sys
from pathlib import Path as _HistoryPath
_HISTORY_REPO_ROOT = _HistoryPath(__file__).resolve().parents[2]
_history_sys.path.insert(0, str(_HISTORY_REPO_ROOT / '.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts'))
from runtime_structure_audits.dynamic_runtime_api_archive_inventory import HistoricalArchiveReader as _ContractHistoryReader
_HISTORY_PATHS = {'docs/plans/optimize/zircon_runtime/60/2026-08-28-sparse-component-locator-pages.md': 'versions/reconciliation-515/3600b9c918ffaaf3058e6e5be76a410aa942d5d803f4ab809a5e17dfcde7581a/docs/plans/optimize/zircon_runtime/60/2026-08-28-sparse-component-locator-pages.md', 'docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-ecs-archetype-columnar-storage.md': 'docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-ecs-archetype-columnar-storage.md'}
_HISTORY_IDENTITIES = {'versions/reconciliation-515/3600b9c918ffaaf3058e6e5be76a410aa942d5d803f4ab809a5e17dfcde7581a/docs/plans/optimize/zircon_runtime/60/2026-08-28-sparse-component-locator-pages.md': (8811, '3600b9c918ffaaf3058e6e5be76a410aa942d5d803f4ab809a5e17dfcde7581a'), 'docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-ecs-archetype-columnar-storage.md': (15729, 'fb19d9900dde0b1ce0048df537e096d5baf0db31d5a6fe98e7a2978a078000f3')}
_CURRENT_POLICY_INPUTS = ('docs/crates/zircon_runtime/scene/ecs.md', 'docs/testing-and-validation/runtime-ecs-validation-worklist.md')

def _read_contract_input(path, *args, **kwargs):
    candidate = _HistoryPath(path)
    try:
        relative = candidate.absolute().relative_to(_HISTORY_REPO_ROOT).as_posix()
    except ValueError:
        relative = None
    key = _HISTORY_PATHS.get(relative)
    if key is None:
        if relative and relative.startswith('docs/plans/') and relative not in _CURRENT_POLICY_INPUTS:
            raise AssertionError('Unregistered current/history contract input: ' + relative)
        return candidate.read_text(*args, **kwargs)
    if args or kwargs.get('encoding', 'utf-8').lower() not in ('utf-8', 'utf8') or kwargs.get('errors') not in (None, 'strict') or set(kwargs) - {'encoding', 'errors'}:
        raise AssertionError('Historical evidence requires strict UTF-8')
    reader = _ContractHistoryReader(identities=_HISTORY_IDENTITIES)
    text = reader.read(key)
    if text is None:
        raise AssertionError(reader.records)
    return text



REPO_ROOT = Path(__file__).resolve().parents[2]
SPARSE_SOURCE = (
    REPO_ROOT
    / "zircon_runtime/src/scene/ecs/storage/component_storage/sparse.rs"
)
SPARSE_TESTS = (
    REPO_ROOT / "zircon_runtime/src/scene/ecs/storage/component_storage/sparse/tests/cases.rs"
)
LOCATOR_SOURCE = (
    REPO_ROOT
    / "zircon_runtime/src/scene/ecs/storage/component_storage/sparse/locator.rs"
)
COMPONENT_STORAGE_SOURCE = (
    REPO_ROOT / "zircon_runtime/src/scene/ecs/storage/component_storage/store.rs"
)
STATUS = "runtime_08_60_sparse_component_locator_source_complete_cargo_product_profile_pending"
STATUS_DOCS = (
    'docs/testing-and-validation/runtime-ecs-validation-worklist.md',
    "docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-ecs-archetype-columnar-storage.md",
    'docs/crates/zircon_runtime/scene/ecs.md',
    "docs/plans/optimize/zircon_runtime/60/2026-08-28-sparse-component-locator-pages.md",
)


class RuntimeSparseComponentLocatorPagesContractTests(unittest.TestCase):
    def test_sparse_locator_is_bounded_and_page_backed(self) -> None:
        source = _read_contract_input(SPARSE_SOURCE, encoding="utf-8")
        locator = _read_contract_input(LOCATOR_SOURCE, encoding="utf-8")

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
        source = _read_contract_input(SPARSE_SOURCE, encoding="utf-8")
        tests = _read_contract_input(SPARSE_TESTS, encoding="utf-8")

        self.assertIn("#[path = \"sparse/tests/cases.rs\"]", source)
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

    def test_locator_memory_diagnostics_aggregate_through_component_storage(self) -> None:
        sparse = _read_contract_input(SPARSE_SOURCE, encoding="utf-8")
        locator = _read_contract_input(LOCATOR_SOURCE, encoding="utf-8")
        storage = _read_contract_input(COMPONENT_STORAGE_SOURCE, encoding="utf-8")

        for anchor in (
            "struct SparseLocatorDiagnostics",
            "fn locator_diagnostics(&self) -> SparseLocatorDiagnostics",
        ):
            self.assertIn(anchor, sparse)
        self.assertIn("fn diagnostics(&self) -> SparseLocatorDiagnostics", locator)
        for anchor in (
            "pub(crate) struct ComponentStorageLocatorDiagnostics",
            "pub(crate) fn sparse_locator_diagnostics(&self) -> ComponentStorageLocatorDiagnostics",
            "storage.locator_diagnostics()",
            "sparse_locator_diagnostics_aggregate_all_sparse_component_owners",
            "sparse_component_storage_count",
            "locator_entry_count",
            "locator_page_count",
            "locator_allocated_bytes",
            "populated.locator_allocated_bytes <= 2 * 16 * 1024",
        ):
            self.assertIn(anchor, ((read_rust_test_file("zircon_runtime/src/scene/ecs/storage/component_storage/tests/store.rs") if anchor in {"populated.locator_allocated_bytes <= 2 * 16 * 1024", "sparse_locator_diagnostics_aggregate_all_sparse_component_owners"} else storage)))

    def test_sparse_locator_status_is_mirrored_by_canonical_plans(self) -> None:
        for relative_path in STATUS_DOCS:
            source = _read_contract_input(REPO_ROOT / relative_path, encoding="utf-8")
            self.assertIn(STATUS, source, relative_path)


if __name__ == "__main__":
    unittest.main()
