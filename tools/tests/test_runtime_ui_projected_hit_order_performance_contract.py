import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/surface/frame_hit_test.rs"
TESTS = ROOT / "zircon_runtime/src/ui/surface/frame_hit_test/tests.rs"


class RuntimeUiProjectedHitOrderPerformanceContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.source = SOURCE.read_text(encoding="utf-8")
        self.tests = TESTS.read_text(encoding="utf-8")

    def test_projection_retains_explicit_z_index_cache(self) -> None:
        self.assertIn("projected_order_keys: BTreeMap<UiNodeId, (i32, u64)>", self.source)
        self.assertIn("self.projected_order_keys = order_plan.order_keys", self.source)
        self.assertIn("overlay_z_base.saturating_add(rank)", self.source)

    def test_projected_order_does_not_flatten_every_entry_to_overlay_base(self) -> None:
        self.assertIn("entry.z_index = z_index", self.source)
        self.assertNotIn("entry.z_index = overlay_z_base;", self.source)

    def test_overlapping_popup_regression_checks_inner_and_stack_z_order(self) -> None:
        self.assertIn(
            "projected_order_preserves_inner_z_and_places_next_popup_above_entire_subtree",
            self.tests,
        )
        compact_tests = " ".join(self.tests.split())
        self.assertIn("projected_z(low_z_high_paint.node_id) < projected_z(high_z_low_paint.node_id)", compact_tests)
        self.assertIn("projected_z(high_z_low_paint.node_id) < projected_z(next_popup_low_z.node_id)", compact_tests)


if __name__ == "__main__":
    unittest.main()
