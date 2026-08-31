from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_resolution.rs"
)


class PreallocatedUnresolvedFeatureCollectionPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_resolution_tracks_the_exact_present_state_count(self) -> None:
        self.assertIn("let mut unresolved_feature_count = 0;", self.source)
        self.assertIn("unresolved_feature_count += 1;", self.source)
        self.assertIn("unresolved_feature_count -= 1;", self.source)

    def test_final_collection_uses_the_exact_tracked_capacity(self) -> None:
        self.assertIn(
            "collect_present_with_capacity(states, unresolved_feature_count)",
            self.source,
        )
        self.assertIn("Vec::with_capacity(present_count)", self.source)
        self.assertIn("present.extend(values.into_iter().flatten());", self.source)
        self.assertNotIn("states.into_iter().flatten().collect()", self.source)

    def test_rust_guard_preserves_order_and_omits_empty_slots(self) -> None:
        self.assertIn(
            "preallocated_present_collection_preserves_order_and_omits_empty_slots",
            self.source,
        )
        self.assertIn("vec![Some(7), None, Some(3), None, Some(11)]", self.source)
        self.assertIn("assert_eq!(present, vec![7, 3, 11]);", self.source)
        self.assertIn("assert!(present.capacity() >= 3);", self.source)


if __name__ == "__main__":
    unittest.main()
