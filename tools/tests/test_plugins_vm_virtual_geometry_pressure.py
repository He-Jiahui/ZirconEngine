import unittest
from pathlib import Path

from tools.plugins_vm_virtual_geometry_pressure import run


ROOT = Path(__file__).resolve().parents[2]
CALL_TABLE = ROOT / (
    "zircon_plugins/zr_vm_language/runtime/src/call_site/script_call_table.rs"
)
CALL_TABLE_TESTS = ROOT / "zircon_plugins/zr_vm_language/runtime/src/call_site/tests.rs"
HOT_INHERITANCE = ROOT / (
    "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/"
    "pending_completion/apply_gpu_page_table_entries.rs"
)


class PluginsVmVirtualGeometryPressureTests(unittest.TestCase):
    def test_generation_qualified_tokens_remove_per_field_atomic_and_hash_work(self) -> None:
        tokens = run()["callsite_tokens"]

        self.assertEqual(tokens["baseline_compile_atomic_operation_count"], 4_096)
        self.assertEqual(tokens["candidate_compile_atomic_operation_count"], 1)
        self.assertEqual(tokens["baseline_retained_token_hash_entry_count"], 4_096)
        self.assertEqual(tokens["candidate_retained_token_hash_entry_count"], 0)
        self.assertEqual(tokens["baseline_token_hash_lookup_count"], 131_072)
        self.assertEqual(tokens["candidate_token_hash_lookup_count"], 0)
        self.assertEqual(tokens["candidate_token_direct_index_count"], 131_072)

    def test_call_table_source_uses_generation_and_dense_site_index(self) -> None:
        source = CALL_TABLE.read_text(encoding="utf-8")
        tests = CALL_TABLE_TESTS.read_text(encoding="utf-8")

        self.assertIn("sites: Arc<Vec<CompiledCallSite>>", source)
        self.assertIn("let generation = (token >> 32) as u32", source)
        self.assertIn("ordinal.saturating_sub(1) as usize", source)
        self.assertNotIn("by_token", source)
        self.assertIn("generation_qualified_direct_token_release_benchmark", tests)

    def test_indexed_hot_inheritance_eliminates_repeated_edge_scans(self) -> None:
        hot = run()["hot_inheritance"]

        self.assertEqual(
            hot["baseline_repeated_descendant_edge_scan_count"], 8_388_608
        )
        self.assertEqual(hot["candidate_repeated_descendant_edge_scan_count"], 0)
        self.assertEqual(hot["baseline_slot_owner_operation_count"], 131_072)
        self.assertEqual(hot["candidate_slot_owner_operation_count"], 2_048)
        self.assertEqual(
            hot["candidate_shared_hot_ancestor_parent_lookup_count"], 4_160
        )
        self.assertEqual(hot["candidate_indexed_hot_ancestor_count"], 4_096)

    def test_virtual_geometry_source_keeps_oracle_and_release_contract(self) -> None:
        source = HOT_INHERITANCE.read_text(encoding="utf-8")

        self.assertIn("indexed_hot_inheritance_release_benchmark", source)
        self.assertIn("VIRTUAL_GEOMETRY_HOT_INHERITANCE_BENCH_V1", source)
        self.assertIn("BENCH_CANDIDATE_PAGES: usize = 2_048", source)
        self.assertIn("BENCH_HIERARCHY_EDGES: usize = 4_096", source)

    def test_release_acceptance_is_explicit_and_pending(self) -> None:
        acceptance = run()["acceptance"]

        self.assertEqual(acceptance["direct_token_p95_maximum_legacy_ratio"], 0.50)
        self.assertEqual(acceptance["indexed_hot_p95_maximum_legacy_ratio"], 0.25)
        self.assertTrue(acceptance["release_timing_pending"])

    def test_model_is_bound_to_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "ca3ac3cc6ad218d04a5cd469447cea2452441321",
        )
        self.assertEqual(len(binding["source_sha256"]), 4)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_non_positive_workloads(self) -> None:
        with self.assertRaises(ValueError):
            run(reflected_fields=0)
        with self.assertRaises(ValueError):
            run(token_rounds=0)
        with self.assertRaises(ValueError):
            run(candidate_pages=0)
        with self.assertRaises(ValueError):
            run(hot_pages=0)
        with self.assertRaises(ValueError):
            run(hierarchy_edges=0)


if __name__ == "__main__":
    unittest.main()
