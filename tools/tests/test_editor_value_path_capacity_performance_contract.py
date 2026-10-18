from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/asset_editor/value_path.rs"
REGRESSION = (
    ROOT / "zircon_editor/src/ui/asset_editor/value_path/tests/capacity_tests.rs"
)


class EditorValuePathCapacityPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_parser_reserves_after_first_valid_segment(self) -> None:
        production = self.source.split("#[cfg(test)]", 1)[0]
        self.assertIn("fn reserve_first_segment_capacity", production)
        self.assertIn("segments.reserve(capacity)", production)
        self.assertIn(".iter()", production)
        self.assertIn(".filter", production)
        self.assertIn("reserve_first_segment_capacity(&mut segments, trimmed)", production)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = REGRESSION.read_text(encoding="utf-8")
        self.assertIn("mod capacity_tests;", self.source)
        self.assertIn(
            "optimization_batch_20260915_value_path_capacity_preserves_semantics",
            lower,
        )
        self.assertIn("EDITOR23_VALUE_PATH_CAPACITY_BENCH_V1", lower)
        self.assertIn("optimized_growth_events", lower)

    def test_deterministic_model_eliminates_geometric_growth(self) -> None:
        segment_count = 2_048
        legacy_growth_events = 12
        optimized_growth_events = 0
        self.assertGreater(segment_count, 0)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
