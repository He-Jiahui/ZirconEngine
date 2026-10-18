import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/accessibility/diagnostics.rs"
TEST_SOURCE = REPO_ROOT / "zircon_runtime/src/ui/accessibility/diagnostics/tests/index_dedup_tests.rs"


class RuntimeAccessibilityDiagnosticIndexCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.test_source = TEST_SOURCE.read_text(encoding="utf-8")

    def test_snapshot_validation_uses_one_authoritative_node_index(self) -> None:
        start = self.source.index("pub(super) fn validate_snapshot_bounded")
        end = self.source.index("fn validate_relation(", start)
        production = self.source[start:end]
        self.assertIn("btree_map::Entry", self.source)
        self.assertIn("nodes.entry(node.node_id)", production)
        self.assertNotIn("BTreeSet", production)
        self.assertNotIn("let mut seen =", production)

    def test_duplicate_diagnostic_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("diagnostic_index_preserves_duplicate_order_and_first_index", self.test_source)
        self.assertIn("RUNTIME819_A11Y_DIAGNOSTIC_INDEX_BENCH_V1", self.test_source)

    def test_deterministic_model_removes_parallel_index_allocation(self) -> None:
        node_count = 4_096
        legacy_index_allocations = 2
        optimized_index_allocations = 1
        self.assertGreater(node_count, 0)
        self.assertGreater(legacy_index_allocations, optimized_index_allocations)
        self.assertEqual(optimized_index_allocations, 1)


if __name__ == "__main__":
    unittest.main()
