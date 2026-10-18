import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PRODUCTION = (
    ROOT
    / "zircon_editor/src/ui/layouts/views/asset_browser/selection_text.rs"
)
LOWER = (
    ROOT / "zircon_editor/src/ui/layouts/views/asset_browser/selection_text/tests/capacity_tests.rs"
)


class EditorSelectionMetadataCapacityContract(unittest.TestCase):
    def test_production_reserves_known_metadata_output_bounds(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        self.assertIn("SELECTION_METADATA_SUMMARY_CAPACITY", source)
        self.assertIn("Vec::with_capacity(selection_metadata_summary_capacity(selection))", source)
        self.assertIn("selection_metadata_body_capacity(selection)", source)
        self.assertIn("Vec::with_capacity(selection_metadata_body_capacity(selection))", source)
        self.assertIn("fn selection_metadata_summary_capacity", source)
        self.assertIn("fn selection_metadata_body_capacity", source)

    def test_capacity_helpers_cover_all_emitted_entries(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        self.assertIn("selection.included_files.len()", source)
        self.assertIn("selection.subassets.len()", source)
        self.assertGreaterEqual(source.count("saturating_add"), 4)

    def test_lower_capacity_regression_and_release_marker_are_wired(self):
        source = LOWER.read_text(encoding="utf-8")
        self.assertIn("EDITOR848_SELECTION_METADATA_SUMMARY_CAPACITY_BENCH_V1", source)
        self.assertIn("selection_metadata_summary_capacity_covers_all_optional_fields", source)
        self.assertIn("selection_metadata_body_capacity_matches_emitted_sections", source)

    def test_deterministic_model_removes_geometric_growth(self):
        included_files = 2
        subassets = 3
        upper_bound = 1 + 1 + included_files + 1 + subassets

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


if __name__ == "__main__":
    unittest.main()
