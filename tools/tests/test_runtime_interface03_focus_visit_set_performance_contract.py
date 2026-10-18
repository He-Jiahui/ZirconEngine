import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/focus.rs"


class RuntimeInterface03FocusVisitSetPerformanceContract(unittest.TestCase):
    def test_focus_traversal_uses_hash_membership_without_changing_output_order(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("use std::collections::HashSet", source)
        self.assertIn("let mut visited = HashSet::with_capacity(tree.nodes.len())", source)
        self.assertIn("visited: &mut HashSet<UiNodeId>", source)

    def test_release_benchmark_and_behavior_oracle_are_present(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn(
            "runtime_interface03_batch22_hashed_focus_visit_set_preserves_membership",
            source,
        )
        self.assertIn("RUNTIME_INTERFACE03_HASHED_FOCUS_VISIT_SET_BENCH_V1", source)


if __name__ == "__main__":
    unittest.main()
