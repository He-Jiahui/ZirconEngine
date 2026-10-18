import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout.rs"
REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout/tests/logical_batch_capacity_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    brace = source.index("{", start)
    depth = 0
    for index in range(brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[brace : index + 1]
    raise AssertionError(f"unterminated function body for {signature}")


class RuntimeLogicalTextBatchCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8") if REGRESSION.exists() else ""

    def test_logical_batches_reserve_layout_line_bound(self) -> None:
        body = function_body(self.source, "pub(super) fn logical_text_batches(")
        compact = "".join(body.split())
        self.assertIn("layout.lines.len()", body)
        self.assertIn("Vec::with_capacity(layout.lines.len())", compact)
        self.assertNotIn("let mut batches = Vec::new();", body)
        self.assertIn("append_visual_line_batch(&mut batches, line)", body)

    def test_lower_owner_is_wired_into_the_runtime_test_tree(self) -> None:
        self.assertIn(
            "#[path = \"resolved_layout/tests/logical_batch_capacity_tests.rs\"]",
            self.source,
        )
        self.assertIn("mod logical_batch_capacity_tests;", self.source)

    def test_lower_capacity_regression_and_release_marker_exist(self) -> None:
        self.assertTrue(self.regression)
        self.assertIn(
            'const BENCHMARK_MARKER: &str = "RUNTIME859_LOGICAL_TEXT_BATCH_CAPACITY_BENCH_V1";',
            self.regression,
        )
        self.assertIn("runtime859_logical_text_batch_capacity_preserves_order", self.regression)
        self.assertIn("runtime859_logical_text_batch_capacity_keeps_empty_layout_zero_capacity", self.regression)
        self.assertIn("runtime859_logical_text_batch_capacity_bench", self.regression)

    def test_only_the_bounded_batch_collector_changes(self) -> None:
        body = function_body(self.source, "pub(super) fn logical_text_batches(")
        self.assertEqual(body.count("Vec::with_capacity(layout.lines.len())"), 1)
        self.assertNotIn("source_isomorphic_plain_text_batches", body)
        self.assertRegex(self.source, r"fn source_isomorphic_plain_text_batches\(")


if __name__ == "__main__":
    unittest.main()
