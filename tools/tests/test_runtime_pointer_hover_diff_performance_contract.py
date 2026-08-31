import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
ROUTING = ROOT / "zircon_runtime/src/ui/surface/surface/event_routing.rs"


class RuntimePointerHoverDiffPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = ROUTING.read_text(encoding="utf-8")
        cls.hover = cls.source.split("fn hover_diff(", 1)[1].split(
            "#[cfg(test)]", 1
        )[0]

    def test_small_hover_paths_keep_the_allocation_free_membership_path(self) -> None:
        self.assertIn("HOVER_DIFF_LINEAR_COMPARISON_BUDGET: usize = 64", self.source)
        self.assertIn("hover_diff_linear(current, previous)", self.hover)
        linear = self.hover.split("fn hover_diff_linear", 1)[1]
        self.assertIn("previous.contains", linear)
        self.assertIn("current.contains", linear)

    def test_equal_paths_return_before_any_membership_structure(self) -> None:
        equal_fast_path = self.hover.index("if current == previous")
        membership = self.hover.index("let mut membership")
        self.assertLess(equal_fast_path, membership)
        self.assertIn("return (Vec::new(), Vec::new())", self.hover)

    def test_large_hover_paths_use_one_reused_hash_membership_table(self) -> None:
        self.assertIn("HashSet::with_capacity(current.len().max(previous.len()))", self.hover)
        self.assertIn("membership.clear()", self.hover)
        self.assertEqual(self.hover.count("HashSet::with_capacity"), 1)

    def test_large_path_membership_is_linear_not_nested_contains(self) -> None:
        indexed = self.hover.split("let mut membership", 1)[1].split(
            "fn hover_diff_linear", 1
        )[0]
        self.assertNotIn("previous.contains", indexed)
        self.assertNotIn("current.contains", indexed)
        self.assertIn("membership.extend(previous.iter().copied())", indexed)
        self.assertIn("membership.extend(current.iter().copied())", indexed)


if __name__ == "__main__":
    unittest.main()
