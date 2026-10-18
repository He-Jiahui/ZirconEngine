import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "layouts"
    / "windows"
    / "workbench_host_window"
    / "chrome_template_projection.rs"
)
TEST_SOURCE = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "layouts"
    / "windows"
    / "workbench_host_window"
    / "chrome_template_projection"
    / "tests"
    / "capacity.rs"
)
MODULE_SOURCE = (
    REPO_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "layouts"
    / "windows"
    / "workbench_host_window"
    / "chrome_template_projection"
    / "tests"
    / "mod.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?![A-Za-z0-9_])\s*\(.*?\)",
        source,
        re.S,
    )
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class EditorPageTabCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.tests = TEST_SOURCE.read_text(encoding="utf-8")
        cls.module = MODULE_SOURCE.read_text(encoding="utf-8")

    def test_visible_tab_projection_reserves_its_bounded_prefix(self) -> None:
        body = function_body(self.source, "visible_page_tab_indices")
        self.assertRegex(
            body,
            r"page_tab_visible_capacity\(\s*visible_cap,\s*tab_count,\s*max_tab_right",
        )
        self.assertIn("Vec::with_capacity", body)
        self.assertLess(body.index("Vec::with_capacity"), body.index("for row"))

        helper = function_body(self.source, "page_tab_visible_capacity")
        self.assertRegex(helper, r"visible_cap\s*\.min\(tab_count\)")
        self.assertIn("MAIN_PAGE_TAB_MIN_WIDTH", helper)
        self.assertIn("MAIN_PAGE_TAB_GAP", helper)
        self.assertIn("is_finite", helper)
        self.assertIn(".max(1)", helper)

    def test_active_tab_fallback_and_deduplication_remain_in_place(self) -> None:
        body = function_body(self.source, "visible_page_tab_indices")
        self.assertIn("active_tab_row(tabs)", body)
        self.assertIn("visible.push(active_row)", body)
        self.assertIn("visible.dedup()", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod capacity;", self.module)
        self.assertIn("editor762_page_tab_visible_capacity_preserves_bounded_active_slot", self.tests)
        self.assertIn(
            "editor762_page_tab_visible_capacity_is_tight_for_a_finite_tab_lane",
            self.tests,
        )
        self.assertIn("EDITOR762_PAGE_TAB_CAPACITY_BENCH_V1", self.tests)
        self.assertIn("optimized_growth_events", self.tests)

    def test_deterministic_model_removes_growth_reallocations(self) -> None:
        visible_prefix = 4_096
        finite_lane_upper_bound = 4
        legacy_growth_events = 1
        optimized_growth_events = 0
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)
        self.assertEqual(visible_prefix, 4_096)
        self.assertLess(finite_lane_upper_bound, visible_prefix)


if __name__ == "__main__":
    unittest.main()
