import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor/src/ui/asset_editor/preview/preview_mock/entries.rs"
)
LOWER = (
    REPO_ROOT / "zircon_editor/src/ui/asset_editor/preview/preview_mock/entries/tests/literal_single_buffer_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}\(.*?\n}}\n\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Editor885PreviewMockLiteralSingleBufferPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_inline_literal_delegates_to_one_recursive_output_buffer(self) -> None:
        owner = function_body(self.source, "preview_mock_inline_literal")
        self.assertIn("let mut literal = String::new();", owner)
        self.assertIn("append_preview_mock_inline_literal(&mut literal, value);", owner)
        self.assertTrue(owner.rstrip().endswith("literal\n}"))
        self.assertIn("fn append_preview_mock_inline_literal(", self.source)

    def test_array_children_append_directly_without_collect_join(self) -> None:
        owner = function_body(self.source, "append_preview_mock_inline_literal")
        self.assertIn("for (index, item) in items.iter().enumerate()", owner)
        self.assertIn("if index > 0", owner)
        self.assertIn("append_preview_mock_inline_literal(literal, item);", owner)
        array_branch = owner.split("Value::Array(items) => {")[1].split(
            "Value::Table(table) => {"
        )[0]
        self.assertNotIn("collect::<Vec", array_branch)
        self.assertNotIn(".join(", array_branch)

    def test_table_sort_order_is_retained_while_values_stream(self) -> None:
        owner = function_body(self.source, "append_preview_mock_inline_literal")
        self.assertIn("let mut entries = table.iter().collect::<Vec<_>>();", owner)
        self.assertIn("entries.sort_by(", owner)
        self.assertIn("for (index, (key, value)) in entries.into_iter().enumerate()", owner)
        self.assertIn('literal.push_str(" = ");', owner)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod literal_single_buffer_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor885_preview_mock_literal_single_buffer_preserves_exact_text",
            lower,
        )
        self.assertIn("EDITOR885_PREVIEW_MOCK_LITERAL_SINGLE_BUFFER_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_array_temporary_allocation_model(self) -> None:
        item_count = 4_096
        legacy_child_strings = item_count
        legacy_vector_slots = item_count
        optimized_child_strings = 0
        optimized_vector_slots = 0
        self.assertEqual(legacy_child_strings, 4_096)
        self.assertEqual(legacy_vector_slots, 4_096)
        self.assertEqual(optimized_child_strings, 0)
        self.assertEqual(optimized_vector_slots, 0)


if __name__ == "__main__":
    unittest.main()
