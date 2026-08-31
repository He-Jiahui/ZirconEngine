from pathlib import Path
import unittest

from tools.runtime_ui_virtual_list_slot_materialization_pressure import run


ROOT = Path(__file__).resolve().parents[2]
VIRTUALIZATION = ROOT / "zircon_runtime/src/ui/layout/virtualization.rs"
MATERIALIZATION = (
    ROOT / "zircon_runtime/src/ui/layout/virtualization/materialization.rs"
)
MATERIALIZED_ARRANGE = (
    ROOT / "zircon_runtime/src/ui/layout/pass/arrange/virtual_list.rs"
)
ARRANGE_TESTS = ROOT / "zircon_runtime/src/ui/layout/pass/arrange/tests.rs"


class RuntimeUiVirtualListSlotMaterializationPerformanceContractTests(unittest.TestCase):
    def test_virtualization_module_owns_bounded_slot_materialization(self) -> None:
        root = VIRTUALIZATION.read_text(encoding="utf-8")
        source = MATERIALIZATION.read_text(encoding="utf-8")

        self.assertIn("mod materialization;", root)
        self.assertIn("UiVirtualListSlotMap", source)
        self.assertIn("slot_logical_indices: Vec<Option<usize>>", source)
        self.assertIn("window.first_visible % slot_count", source)
        self.assertIn("% slot_count", source)
        self.assertNotIn("0..logical_count", source)

    def test_reconcile_reuses_caller_change_storage_and_advances_only_on_change(self) -> None:
        source = MATERIALIZATION.read_text(encoding="utf-8")
        reconcile = source.split("pub fn reconcile(", 1)[1].split(
            "pub fn generation", 1
        )[0]

        self.assertIn("changes: &mut Vec<UiVirtualListSlotChange>", reconcile)
        self.assertIn("changes.clear()", reconcile)
        self.assertIn("self.generation = self.generation.wrapping_add(1)", reconcile)
        self.assertNotIn("Vec::new()", reconcile)

    def test_lower_contract_covers_scale_scroll_seek_and_shrink(self) -> None:
        source = MATERIALIZATION.read_text(encoding="utf-8")

        self.assertIn("slot_count_is_independent_of_logical_count", source)
        self.assertIn("one_row_scroll_rebinds_only_one_boundary_slot", source)
        self.assertIn("large_seek_rebinds_at_most_the_slot_capacity", source)
        self.assertIn("model_shrink_clears_out_of_range_assignments", source)
        self.assertIn(
            "fractional_scroll_capacity_keeps_both_partial_boundary_items", source
        )

    def test_boundary_windows_keep_every_physical_slot_materialized(self) -> None:
        source = MATERIALIZATION.read_text(encoding="utf-8")
        arrange = MATERIALIZED_ARRANGE.read_text(encoding="utf-8")
        arrange_tests = ARRANGE_TESTS.read_text(encoding="utf-8")

        self.assertIn("backfill_window_to_slot_count", source)
        self.assertIn("boundary_windows_backfill_to_slot_capacity", source)
        self.assertIn(
            "materialized_virtual_list_arranges_backfilled_slot_outside_visible_window",
            arrange_tests,
        )
        self.assertNotIn(
            "logical_index < visible_window.first_visible",
            arrange,
        )
        self.assertNotIn(
            "logical_index >= visible_window.last_visible_exclusive",
            arrange,
        )

    def test_pressure_model_bounds_planner_work_but_rejects_product_claims(self) -> None:
        result = run(
            logical_count=100_000,
            row_subtree_node_count=6,
            viewport_extent=800.0,
            item_extent=24.0,
            overscan=3,
            scroll_update_count=4_096,
            large_seek_count=64,
        )

        self.assertEqual(result["slot_count"], 41)
        self.assertEqual(result["retained_child_node_count"], 600_000)
        self.assertEqual(result["bounded_slot_node_count"], 246)
        self.assertEqual(result["slot_node_count_reduction_ratio"], 2_439.02)
        self.assertLessEqual(
            result["planner_slot_visits"],
            result["slot_count"] * result["scroll_update_count"],
        )
        self.assertFalse(result["surface_materializer_wired"])
        self.assertFalse(result["cpu_or_rss_measured"])


if __name__ == "__main__":
    unittest.main()
