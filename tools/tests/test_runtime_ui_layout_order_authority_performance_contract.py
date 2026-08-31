from pathlib import Path
import unittest

from tools.ui_layout_order_authority_pressure import run


ROOT = Path(__file__).resolve().parents[2]
LAYOUT_MEASURE = ROOT / "zircon_runtime/src/ui/layout/pass/measure.rs"
LAYOUT_MEASURE_TRAVERSAL = (
    ROOT / "zircon_runtime/src/ui/layout/pass/measure/traversal.rs"
)
LAYOUT_ARRANGE = ROOT / "zircon_runtime/src/ui/layout/pass/arrange.rs"
LAYOUT_INCREMENTAL = ROOT / "zircon_runtime/src/ui/layout/pass/incremental.rs"
LAYOUT_SLOT = ROOT / "zircon_runtime/src/ui/layout/pass/slot.rs"
UI_TREE = ROOT / "zircon_runtime_interface/src/ui/tree/node/ui_tree.rs"
TREE_BUILDER = ROOT / "zircon_runtime/src/ui/template/build/tree_builder.rs"
NODE_POOL = ROOT / "zircon_runtime/src/ui/surface/node_pool.rs"


class RuntimeUiLayoutOrderAuthorityPerformanceContractTests(unittest.TestCase):
    def test_layout_passes_share_generation_owned_child_order(self) -> None:
        measure = LAYOUT_MEASURE.read_text(encoding="utf-8")
        measure_traversal = LAYOUT_MEASURE_TRAVERSAL.read_text(encoding="utf-8")
        arrange = LAYOUT_ARRANGE.read_text(encoding="utf-8")
        incremental = LAYOUT_INCREMENTAL.read_text(encoding="utf-8")
        slot = LAYOUT_SLOT.read_text(encoding="utf-8")
        tree = UI_TREE.read_text(encoding="utf-8")
        tree_builder = TREE_BUILDER.read_text(encoding="utf-8")
        node_pool = NODE_POOL.read_text(encoding="utf-8")

        self.assertIn("layout_order_generation", tree)
        self.assertIn("pending_layout_order_parent_ids", tree)
        self.assertIn("push_layout_slot", tree)
        self.assertIn("retain_layout_slots", tree)
        self.assertIn("mutate_layout_slot", tree)
        self.assertIn("ordered_children_by_parent", slot)
        self.assertIn("ordered_children_for_container", slot)
        self.assertIn("empty_ordered_children", slot)
        self.assertIn("Arc<[UiNodeId]>", slot)
        self.assertIn("ordered_children_for_container", measure_traversal)
        self.assertIn("synchronize_ordered_children", incremental)
        self.assertNotIn("prepare_ordered_child_desired", measure)
        self.assertNotIn("order_children_for_container", arrange)
        self.assertNotIn(".sort", measure)
        self.assertNotIn(".sort", arrange)
        self.assertNotIn("tree.slots.push(slot)", tree_builder)
        self.assertNotIn("tree.slots.retain", node_pool)

    def test_stable_child_order_is_sorted_once_per_topology_generation(self) -> None:
        result = run(
            child_count=10_000,
            layout_update_count=10_000,
            topology_change_count=1,
        )

        self.assertEqual(result["current_order_sort_count"], 10_000)
        self.assertEqual(result["generation_owned_order_sort_count"], 1)
        self.assertEqual(result["current_order_comparison_work"], 1_400_000_000)
        self.assertEqual(
            result["generation_owned_order_comparison_work"],
            140_000,
        )
        self.assertEqual(result["eliminated_order_comparison_work"], 1_399_860_000)
        self.assertEqual(result["order_comparison_reduction_ratio"], 10_000)
        self.assertEqual(result["required_child_aggregation_work"], 100_000_000)

    def test_order_work_scales_with_real_topology_changes(self) -> None:
        result = run(
            child_count=1_024,
            layout_update_count=4_096,
            topology_change_count=8,
        )

        self.assertEqual(result["current_order_sort_count"], 4_096)
        self.assertEqual(result["generation_owned_order_sort_count"], 8)
        self.assertEqual(result["order_comparison_reduction_ratio"], 512)

    def test_rejects_more_topology_changes_than_layout_updates(self) -> None:
        with self.assertRaises(ValueError):
            run(
                child_count=16,
                layout_update_count=4,
                topology_change_count=5,
            )


if __name__ == "__main__":
    unittest.main()
