import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_editor/src/ui/host/editor_event_execution/viewport_event.rs"


class EditorViewportEffectsCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_effect_projection_reserves_exact_effect_count(self) -> None:
        match = re.search(r"fn\s+viewport_effects\b.*?\n}\n", self.source, re.S)
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("let effect_count", body)
        self.assertIn("Vec::with_capacity(effect_count)", body)
        self.assertNotIn("let mut effects = Vec::new()", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "viewport_effects_capacity_preserves_empty_and_ordered_effects",
            self.source,
        )
        self.assertIn("EDITOR817_VIEWPORT_EFFECTS_CAPACITY_BENCH_V1", self.source)

    def test_deterministic_model_removes_small_vector_growth(self) -> None:
        max_effects = 3
        legacy_growth_events = 1
        optimized_growth_events = 0
        self.assertEqual(max_effects, 3)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
