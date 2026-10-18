import unittest
from pathlib import Path


def _function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    open_brace = source.index("{", start)
    depth = 0
    for index in range(open_brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[open_brace + 1 : index]
    raise AssertionError(f"unterminated function body: {signature}")


class Editor787ScenePickerSinglePassWindowPerformanceContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.source_path = (
            self.repo_root / "zircon_editor/src/ui/retained_host/app/scene_picker_session.rs"
        )
        self.source = self.source_path.read_text(encoding="utf-8")
        self.body = _function_body(self.source, "fn scene_open_query_window<'a>")

    def test_query_window_uses_one_match_scan_and_two_bounded_pages(self) -> None:
        self.assertIn(
            "let mut requested_entries: Option<Vec<&'a ScenePickerEntry>> = None;",
            self.body,
        )
        self.assertIn(
            "let mut last_entries: Option<Vec<&'a ScenePickerEntry>> = None;",
            self.body,
        )
        self.assertEqual(
            self.body.count("Vec::with_capacity(SCENE_PICKER_WINDOW_ENTRIES)"),
            2,
        )
        self.assertEqual(self.body.count("get_or_insert_with"), 2)
        self.assertIn("for entry in entries {", self.body)
        self.assertIn("total_match_count = total_match_count.saturating_add(1)", self.body)
        self.assertIn(
            "match_index % SCENE_PICKER_WINDOW_ENTRIES == 0",
            self.body,
        )
        self.assertIn("last_entries.clear()", self.body)
        self.assertNotIn(".count()", self.body)
        self.assertNotIn(".filter(|entry|", self.body)

    def test_query_window_preserves_offset_fallback_and_order(self) -> None:
        self.assertIn("normalized_window_offset(requested_offset, total_match_count)", self.body)
        self.assertIn("requested_page_offset", self.body)
        self.assertIn("window_offset == requested_page_offset", self.body)
        self.assertIn("SceneOpenQueryWindow", self.body)
        self.assertIn("total_match_count", self.body)

    def test_lower_single_pass_regression_and_benchmark_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"scene_picker_session/tests/single_pass_tests.rs\"]",
            self.source,
        )
        lower = (
            self.repo_root / "zircon_editor/src/ui/retained_host/app/scene_picker_session/tests/single_pass_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "optimization_batch_20260915_editor787_scene_picker_single_pass_preserves_windows",
            lower,
        )
        self.assertIn(
            "optimization_batch_20260915_editor787_scene_picker_single_pass_p95",
            lower,
        )
        self.assertIn(
            "EDITOR787_SCENE_PICKER_SINGLE_PASS_WINDOW_BENCH_V1",
            lower,
        )
        self.assertIn("legacy_scan", lower)
        self.assertIn("single_pass", lower)


if __name__ == "__main__":
    unittest.main()
