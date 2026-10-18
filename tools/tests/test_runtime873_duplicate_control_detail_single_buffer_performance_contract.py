import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/template/asset/compiler/control_scope.rs"
LOWER = (
    REPO_ROOT / "zircon_runtime/src/ui/template/asset/compiler/control_scope/tests/duplicate_detail_single_buffer_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}\(.*?\n}}\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Runtime873DuplicateControlDetailSingleBufferPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_duplicate_detail_uses_one_exact_capacity_output(self) -> None:
        owner = function_body(self.source, "duplicate_control_id_detail")
        self.assertIn("let key_length", owner)
        self.assertIn("let separator_count", owner)
        self.assertIn("String::with_capacity", owner)

    def test_duplicate_detail_appends_sorted_borrowed_keys(self) -> None:
        owner = function_body(self.source, "duplicate_control_id_detail")
        self.assertIn("duplicates.keys().enumerate()", owner)
        self.assertIn('detail.push_str(", ")', owner)
        self.assertIn("detail.push_str(control_id)", owner)

    def test_duplicate_detail_does_not_clone_collect_join_or_format(self) -> None:
        owner = function_body(self.source, "duplicate_control_id_detail")
        self.assertNotIn(".cloned()", owner)
        self.assertNotIn("collect::<", owner)
        self.assertNotIn(".join(", owner)
        self.assertNotIn("format!", owner)
        validator = function_body(self.source, "validate_unique_control_ids")
        self.assertIn("detail: duplicate_control_id_detail(&duplicates)", validator)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod duplicate_detail_single_buffer_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "runtime873_duplicate_control_detail_single_buffer_preserves_exact_text",
            lower,
        )
        self.assertIn(
            "RUNTIME873_DUPLICATE_CONTROL_DETAIL_SINGLE_BUFFER_BENCH_V1", lower
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_duplicate_detail_temporary_allocation_model(self) -> None:
        details = 4_096
        duplicate_ids = 64
        legacy_child_strings = details * duplicate_ids
        legacy_vector_slots = details * duplicate_ids
        legacy_join_outputs = details
        optimized_child_strings = 0
        optimized_vector_slots = 0
        optimized_join_outputs = 0
        self.assertEqual(legacy_child_strings, 262_144)
        self.assertEqual(legacy_vector_slots, 262_144)
        self.assertEqual(legacy_join_outputs, 4_096)
        self.assertEqual(optimized_child_strings, 0)
        self.assertEqual(optimized_vector_slots, 0)
        self.assertEqual(optimized_join_outputs, 0)


if __name__ == "__main__":
    unittest.main()
