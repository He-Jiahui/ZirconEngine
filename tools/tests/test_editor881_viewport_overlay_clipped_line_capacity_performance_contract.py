import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/data/viewport_image/overlay.rs"
)
LOWER = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/host_contract/data/viewport_image/overlay/tests/capacity_tests.rs"
)


class Editor881ViewportOverlayClippedLineCapacityPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        owner = re.search(
            r"pub\(crate\) fn from_screen_lines\(.*?\n    }\n\n",
            cls.source,
            re.S,
        )
        if owner is None:
            raise AssertionError("viewport overlay construction owner is missing")
        cls.owner = owner.group(0)
        helper = re.search(
            r"fn clipped_raster_lines\(.*?\n}\n\n",
            cls.source,
            re.S,
        )
        cls.helper = helper.group(0) if helper is not None else ""

    def test_overlay_owner_routes_clipping_through_shared_helper(self) -> None:
        self.assertIn("clipped_raster_lines(lines, source_size)", self.owner)

    def test_helper_lazily_reserves_the_source_line_bound(self) -> None:
        self.assertIn("let mut clipped = Vec::new()", self.helper)
        self.assertIn("if clipped.is_empty()", self.helper)
        self.assertIn("clipped.reserve(lines.len())", self.helper)
        self.assertIn("clipped.push(line)", self.helper)

    def test_owner_no_longer_collects_filtered_lines(self) -> None:
        self.assertNotIn("collect::<Vec", self.owner)
        self.assertNotIn(".collect()", self.owner)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod capacity_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor881_clipped_line_capacity_keeps_all_rejected_path_lazy",
            lower,
        )
        self.assertIn("EDITOR881_VIEWPORT_OVERLAY_CLIPPED_LINE_CAPACITY_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_growth_model(self) -> None:
        line_count = 4_096
        legacy_growth_events = 11
        optimized_growth_events = 0
        self.assertEqual(line_count, 4_096)
        self.assertEqual(legacy_growth_events, 11)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
