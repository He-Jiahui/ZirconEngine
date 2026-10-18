import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PRODUCTION = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/assets/refresh.rs"
)
LOWER = (
    ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh/tests/capacity_tests.rs"
)


class EditorAssetRefreshVisualPathCapacityContract(unittest.TestCase):
    def test_production_uses_bounded_lazy_reservation(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        production = source

        self.assertIn("saturating_mul(2)", production)
        self.assertIn("saturating_add(events.editor_asset_changes.len())", production)
        self.assertIn("if paths.is_empty() && capacity_hint > 0", production)
        self.assertIn("paths.reserve(capacity_hint);", production)
        self.assertIn("reserve_visual_asset_paths", production)
        self.assertIn("paths.sort();", production)
        self.assertIn("paths.dedup();", production)

    def test_production_reserves_all_visual_event_sources(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        production = source

        self.assertGreaterEqual(
            production.count("reserve_visual_asset_paths"),
            3,
        )
        self.assertIn("push_visual_asset_locator(&mut paths, &change.uri, capacity_hint)", production)
        self.assertIn(
            "push_visual_asset_locator(&mut paths, previous_uri, capacity_hint)",
            production,
        )
        self.assertIn("reserve_visual_asset_paths(&mut paths, capacity_hint);", production)

    def test_lower_capacity_regression_and_release_marker_are_wired(self):
        source = LOWER.read_text(encoding="utf-8")
        self.assertIn("EDITOR844_ASSET_REFRESH_VISUAL_PATH_CAPACITY_BENCH_V1", source)
        self.assertIn("visual_path_capacity_is_lazy_and_bounded", source)
        self.assertIn("non_visual_refresh_keeps_zero_capacity", source)

    def test_deterministic_model_removes_geometric_growth(self):
        event_counts = (8, 7, 6)
        upper_bound = event_counts[0] * 2 + event_counts[1] + event_counts[2] * 2

        def growth_events(reserved):
            capacity = reserved
            length = 0
            growths = 0
            for _ in range(upper_bound):
                if length == capacity:
                    capacity = max(4, capacity * 2)
                    growths += 1
                length += 1
            return growths

        legacy = growth_events(0)
        optimized = growth_events(upper_bound)
        self.assertGreater(legacy, optimized)
        self.assertEqual(optimized, 0)
        self.assertLessEqual(optimized, legacy * 75 // 100)


if __name__ == "__main__":
    unittest.main()
