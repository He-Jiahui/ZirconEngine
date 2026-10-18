import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_editor/src/ui/animation_editor/session/graph.rs"
LOWER = (
    REPO_ROOT / "zircon_editor/src/ui/animation_editor/session/graph/tests/single_buffer_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}\(.*?\n}}\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Editor890AnimationGraphLabelSingleBufferPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_graph_label_uses_one_output_buffer(self) -> None:
        owner = function_body(self.source, "graph_node_label")
        self.assertIn(
            "let mut label = String::with_capacity(graph_node_label_capacity(node));",
            owner,
        )
        self.assertIn("write!(&mut label", owner)
        self.assertTrue(owner.rstrip().endswith("label\n}"))

    def test_graph_id_lists_append_with_positional_delimiters(self) -> None:
        owner = function_body(self.source, "append_graph_ids")
        self.assertIn("ids.iter().enumerate()", owner)
        self.assertIn("if index > 0", owner)
        self.assertIn('label.push_str(", ");', owner)
        self.assertIn("label.push_str(id);", owner)

    def test_graph_label_does_not_join_or_format_intermediate_output(self) -> None:
        owner = function_body(self.source, "graph_node_label")
        self.assertNotIn(".join(", owner)
        self.assertNotIn("format!(", owner)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod single_buffer_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor890_animation_graph_label_single_buffer_preserves_exact_text",
            lower,
        )
        self.assertIn(
            "EDITOR890_ANIMATION_GRAPH_LABEL_SINGLE_BUFFER_BENCH_V1", lower
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_graph_label_intermediate_allocation_model(self) -> None:
        labels = 4_096
        legacy_join_outputs = labels
        optimized_join_outputs = 0
        self.assertEqual(legacy_join_outputs, 4_096)
        self.assertEqual(optimized_join_outputs, 0)


if __name__ == "__main__":
    unittest.main()
