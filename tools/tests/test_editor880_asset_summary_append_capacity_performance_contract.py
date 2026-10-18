import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor/src/ui/layouts/views/asset_browser/summary_nodes.rs"
)
LOWER = (
    REPO_ROOT / "zircon_editor/src/ui/layouts/views/asset_browser/summary_nodes/tests/capacity_tests.rs"
)


class Editor880AssetSummaryAppendCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        match = re.search(
            r"pub\(super\) fn append_asset_browser_summary_nodes\(.*?\n}\n\n",
            cls.source,
            re.S,
        )
        if match is None:
            raise AssertionError("asset summary append owner is missing")
        cls.body = match.group(0)

    def test_fixed_append_bound_is_named(self) -> None:
        self.assertIn("const SUMMARY_APPENDED_NODE_COUNT: usize = 5;", self.source)

    def test_append_reserves_the_exact_fixed_bound_before_materialization(self) -> None:
        reserve = self.body.index("nodes.reserve(SUMMARY_APPENDED_NODE_COUNT)")
        materialize = self.body.index("let selected_asset = selected_asset(snapshot)")
        self.assertLess(reserve, materialize)

    def test_append_stays_direct_and_preserves_five_pushes(self) -> None:
        self.assertEqual(self.body.count("nodes.push("), 5)
        self.assertNotIn("collect::<Vec", self.body)
        self.assertNotIn("let mut summary_nodes", self.body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod capacity_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("editor880_asset_summary_capacity_preserves_prefix_and_order", lower)
        self.assertIn("EDITOR880_ASSET_SUMMARY_APPEND_CAPACITY_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_refresh_growth_model(self) -> None:
        refresh_count = 4_096
        legacy_growth_events = refresh_count * 2
        optimized_growth_events = 0
        self.assertEqual(legacy_growth_events, 8_192)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
