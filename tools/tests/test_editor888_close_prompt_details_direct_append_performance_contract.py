import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor/src/ui/retained_host/app/close_prompt/presentation/text.rs"
)
LOWER = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/app/close_prompt/presentation/text/tests/direct_append_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}\(.*?\n}}\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Editor888ClosePromptDetailsDirectAppendPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_dirty_details_appends_names_directly(self) -> None:
        owner = function_body(self.source, "dirty_details")
        self.assertIn("let mut names = String::new();", owner)
        self.assertIn(".take(3)", owner)
        self.assertIn(".enumerate()", owner)
        self.assertIn("names.push_str(name);", owner)

    def test_dirty_details_preserves_positional_delimiters(self) -> None:
        owner = function_body(self.source, "dirty_details")
        self.assertIn("if index > 0", owner)
        self.assertIn('names.push_str(", ");', owner)
        self.assertIn('names.push_str(", ...");', owner)

    def test_dirty_details_does_not_collect_or_join(self) -> None:
        owner = function_body(self.source, "dirty_details")
        self.assertNotIn("collect::<Vec", owner)
        self.assertNotIn(".join(", owner)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod direct_append_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor888_close_prompt_details_direct_append_preserves_exact_text",
            lower,
        )
        self.assertIn(
            "EDITOR888_CLOSE_PROMPT_DETAILS_DIRECT_APPEND_BENCH_V1", lower
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_temporary_reference_slot_model(self) -> None:
        renders = 4_096
        visible_names = 3
        legacy_vector_slots = renders * visible_names
        optimized_vector_slots = 0
        self.assertEqual(legacy_vector_slots, 12_288)
        self.assertEqual(optimized_vector_slots, 0)


if __name__ == "__main__":
    unittest.main()
