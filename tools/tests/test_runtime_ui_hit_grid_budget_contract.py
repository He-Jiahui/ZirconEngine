from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HIT_TEST = ROOT / "zircon_runtime/src/ui/tree/hit_test.rs"


class RuntimeUiHitGridBudgetContractTests(unittest.TestCase):
    def test_base_hit_grid_has_checked_geometry_budget(self):
        source = HIT_TEST.read_text(encoding="utf-8")
        self.assertIn("HIT_GRID_MAX_AXIS_CELLS: u32 = 128", source)
        self.assertIn("HIT_GRID_MAX_CELL_COUNT", source)
        self.assertIn("checked_mul", source)
        self.assertIn("frame_is_finite_positive", source)
        self.assertIn("HIT_GRID_MAX_ENTRY_CELL_COUNT", source)
        self.assertIn("hit_grid_dimensions", source)
        self.assertIn("cell_count_for_frame", source)
        self.assertIn("cell_span_for_frame", source)
        self.assertIn("grid.columns > HIT_GRID_MAX_AXIS_CELLS", source)
        self.assertIn("grid.cells.len() > HIT_GRID_MAX_CELL_COUNT", source)
        self.assertIn('"ui.hit_grid.invalid_geometry_entry_count"', source)
        self.assertIn('"ui.hit_grid.coarse_fallback_count"', source)

if __name__ == "__main__":
    unittest.main()
