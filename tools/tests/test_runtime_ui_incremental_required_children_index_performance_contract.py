from pathlib import Path
import unittest

from tools.ui_required_children_index_pressure import run

ROOT = Path(__file__).resolve().parents[2]
ENGINE = ROOT / "zircon_runtime/src/ui/layout/pass/engine.rs"
INCREMENTAL = ROOT / "zircon_runtime/src/ui/layout/pass/incremental.rs"


class RuntimeUiIncrementalRequiredChildrenIndexPerformanceContract(unittest.TestCase):
    def test_incremental_engine_owns_a_parent_scoped_required_child_index(self) -> None:
        source = ENGINE.read_text(encoding="utf-8")

        self.assertIn("required_children_by_parent", source)
        self.assertIn("fn index_required_children", source)
        self.assertIn("self.required_children_by_parent.get", source)
        self.assertNotIn(
            "self.required_node_ids.iter().copied().filter",
            source,
        )

    def test_incremental_layout_builds_the_index_once_before_arrangement(self) -> None:
        source = INCREMENTAL.read_text(encoding="utf-8")

        self.assertIn("engine_context.index_required_children(tree)", source)
        self.assertLess(
            source.index("engine_context.index_required_children(tree)"),
            source.index("for root_id in &measurement_roots"),
        )

    def test_pressure_model_preserves_lists_and_removes_parent_scan_work(self) -> None:
        result = run(parent_count=32, required_count=256)

        self.assertTrue(result["semantic_lists_match"])
        self.assertEqual(result["old_scan_checks"], 32 * 256)
        self.assertEqual(result["indexed_build_checks"], 256)
        self.assertEqual(result["scan_reduction_ratio"], 32.0)


if __name__ == "__main__":
    unittest.main()
