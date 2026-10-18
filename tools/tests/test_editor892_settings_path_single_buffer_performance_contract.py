import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/settings_window.rs"
)
LOWER = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/settings_window/tests/path_single_buffer_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}\(.*?\n}}\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Editor892SettingsPathSingleBufferPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_settings_path_uses_one_exact_capacity_output(self) -> None:
        owner = function_body(self.source, "joined")
        self.assertIn("let value_length", owner)
        self.assertIn("let separator_count", owner)
        self.assertIn("String::with_capacity", owner)

    def test_settings_path_appends_borrowed_values_positionally(self) -> None:
        owner = function_body(self.source, "joined")
        self.assertIn("values.iter().enumerate()", owner)
        self.assertIn("if index > 0", owner)
        self.assertIn("output.push('/')", owner)
        self.assertIn("output.push_str(value.as_ref())", owner)

    def test_settings_path_does_not_collect_or_join(self) -> None:
        owner = function_body(self.source, "joined")
        self.assertNotIn("collect::<", owner)
        self.assertNotIn(".join(", owner)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod path_single_buffer_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor892_settings_path_single_buffer_preserves_exact_text", lower
        )
        self.assertIn("EDITOR892_SETTINGS_PATH_SINGLE_BUFFER_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_settings_path_temporary_allocation_model(self) -> None:
        paths = 4_096
        segments = 64
        legacy_reference_slots = paths * segments
        optimized_reference_slots = 0
        self.assertEqual(legacy_reference_slots, 262_144)
        self.assertEqual(optimized_reference_slots, 0)


if __name__ == "__main__":
    unittest.main()
