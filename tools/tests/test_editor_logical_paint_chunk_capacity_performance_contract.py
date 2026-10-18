import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PRODUCTION = (
    ROOT
    / "zircon_editor/src/ui/layouts/views/asset_browser/logical_paint_source.rs"
)
LOWER = (
    ROOT / "zircon_editor/src/ui/layouts/views/asset_browser/logical_paint_source/tests/capacity_tests.rs"
)


class EditorLogicalPaintChunkCapacityContract(unittest.TestCase):
    def test_production_reserves_each_built_chunk_from_source_length(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        self.assertIn("let mut projected_items = Vec::with_capacity(source_chunk.len())", source)
        self.assertIn("projected_items.extend(", source)
        self.assertIn("projected_items.into()", source)
        self.assertIn("source_chunk.len()", source)

    def test_reuse_path_remains_before_new_chunk_projection(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        reuse_marker = "if let Some(reused) = reused"
        projection_marker = "let mut projected_items = Vec::with_capacity(source_chunk.len())"
        self.assertLess(source.index(reuse_marker), source.index(projection_marker))
        self.assertIn("cloned_chunk(chunk_index)", source)

    def test_lower_capacity_regression_and_release_marker_are_wired(self):
        source = LOWER.read_text(encoding="utf-8")
        self.assertIn("EDITOR849_LOGICAL_PAINT_CHUNK_CAPACITY_BENCH_V1", source)
        self.assertIn("logical_paint_chunk_capacity_matches_source_chunk_length", source)
        self.assertIn("logical_paint_chunk_capacity_model_removes_geometric_growth", source)

    def test_deterministic_chunk_model_removes_geometric_growth(self):
        chunk_length = 64

        def growth_events(reserved):
            capacity = reserved
            length = 0
            growths = 0
            for _ in range(chunk_length):
                if length == capacity:
                    capacity = max(4, capacity * 2)
                    growths += 1
                length += 1
            return growths

        legacy = growth_events(0)
        optimized = growth_events(chunk_length)
        self.assertEqual(legacy, 5)
        self.assertEqual(optimized, 0)


if __name__ == "__main__":
    unittest.main()
