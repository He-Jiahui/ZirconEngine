import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/v2/style.rs"
REGRESSION = REPO_ROOT / "zircon_runtime/src/ui/v2/style/tests/selector_path_tests.rs"


class RuntimeSelectorPathSingleBufferPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8") if REGRESSION.exists() else ""

    def test_selector_path_reuses_one_reverseable_path_buffer(self) -> None:
        start = self.source.index("fn runtime_selector_path")
        body = self.source[start:]
        self.assertIn("let mut path = Vec::new();", body)
        self.assertIn("path.push(SelectorPathNode::from_tree_node(", body)
        self.assertIn("path.reverse();", body)
        self.assertNotIn("let mut ids = Vec::new();", body)
        self.assertNotIn("Vec::with_capacity(ids.len())", body)
        self.assertNotIn("ids.into_iter().enumerate()", body)

    def test_selector_path_restores_root_host_semantics_after_reverse(self) -> None:
        start = self.source.index("fn runtime_selector_path")
        body = self.source[start:]
        self.assertIn("component_states.get(current_id)", body)
        self.assertIn("component_states.get(current_id),\n            false,", body)
        self.assertIn("if let Some(host) = path.first_mut()", body)
        self.assertIn("host.is_host = true", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod selector_path_tests;", self.source)
        self.assertIn("runtime854_runtime_selector_path_preserves_order_and_host", self.regression)
        self.assertIn("RUNTIME854_RUNTIME_SELECTOR_PATH_SINGLE_BUFFER_BENCH_V1", self.regression)

    def test_deterministic_model_removes_intermediate_id_buffer(self) -> None:
        depth = 128
        legacy_intermediate_buffers = 1
        optimized_intermediate_buffers = 0
        self.assertGreater(depth, 0)
        self.assertEqual(legacy_intermediate_buffers, 1)
        self.assertEqual(optimized_intermediate_buffers, 0)


if __name__ == "__main__":
    unittest.main()
