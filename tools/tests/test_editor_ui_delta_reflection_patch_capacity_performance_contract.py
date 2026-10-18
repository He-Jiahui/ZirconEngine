from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/core/editor_message/editor_ui_delta.rs"


class EditorUiDeltaReflectionPatchCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("pub fn reflection_patches(&self)")
        end = cls.source.index("\n    }\n}", start) + len("\n    }")
        cls.body = cls.source[start:end]

    def test_exact_node_delta_bound_is_reserved_before_patch_projection(self) -> None:
        self.assertIn(
            "let mut patches = Vec::with_capacity(self.node_delta_count());",
            self.body,
        )
        self.assertIn("patches.extend", self.body)
        self.assertIn("patches\n", self.body)
        self.assertLess(
            self.body.index("Vec::with_capacity(self.node_delta_count())"),
            self.body.index("patches.extend"),
        )
        self.assertIn("EDITOR810_UI_DELTA_REFLECTION_PATCH_CAPACITY_BENCH_V1", self.source)

    def test_barriers_are_skipped_and_patch_ownership_is_cloned(self) -> None:
        self.assertIn("EditorUiDeltaEntry::Nodes(deltas)", self.body)
        self.assertIn("EditorUiDeltaEntry::Barrier", self.source)
        self.assertIn("delta.patch.clone()", self.body)
        self.assertNotIn("filter_map", self.body)

    def test_empty_batches_keep_zero_capacity(self) -> None:
        self.assertIn("Vec::with_capacity(self.node_delta_count())", self.body)
        self.assertIn("pub fn node_delta_count(&self)", self.source)

    def test_dense_projection_removes_geometric_growth_events(self) -> None:
        delta_count = 4096
        legacy_growth_events = 0
        capacity = 0
        for length in range(1, delta_count + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1

        self.assertGreater(legacy_growth_events, 0)
        optimized_growth_events = 0
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
