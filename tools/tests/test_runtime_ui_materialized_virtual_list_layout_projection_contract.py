from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SLOT_INDEX = ROOT / "zircon_runtime/src/ui/layout/pass/slot.rs"
PROJECTION = ROOT / "zircon_runtime/src/ui/layout/pass/virtual_list_layout.rs"
ARRANGE = ROOT / "zircon_runtime/src/ui/layout/pass/arrange.rs"
ARRANGE_TESTS = ROOT / "zircon_runtime/src/ui/layout/pass/arrange/tests.rs"
VIRTUAL_LIST_ARRANGE = (
    ROOT / "zircon_runtime/src/ui/layout/pass/arrange/virtual_list.rs"
)
MATERIALIZATION = (
    ROOT / "zircon_runtime/src/ui/surface/virtual_list_materialization.rs"
)


class RuntimeUiMaterializedVirtualListLayoutProjectionContractTests(
    unittest.TestCase
):
    def test_layout_slot_index_owns_a_separate_rebuildable_projection(self) -> None:
        slot_index = SLOT_INDEX.read_text(encoding="utf-8")

        self.assertTrue(PROJECTION.exists())
        self.assertIn("virtual_lists: RefCell<UiMaterializedVirtualListLayoutIndex>", slot_index)
        self.assertIn("virtual_lists: RefCell::new(self.virtual_lists.borrow().clone())", slot_index)

    def test_surface_publishes_only_registered_physical_slot_assignments(self) -> None:
        source = MATERIALIZATION.read_text(encoding="utf-8")

        self.assertIn("publish_layout_projection", source)
        self.assertIn("replace_materialized_virtual_list", source)
        self.assertIn("clear_materialized_virtual_list", source)
        self.assertIn("state.slot_node_ids.len() != state.slots.slot_count()", source)

    def test_arrangement_uses_logical_extent_and_direct_logical_positioning(self) -> None:
        source = ARRANGE.read_text(encoding="utf-8") + VIRTUAL_LIST_ARRANGE.read_text(
            encoding="utf-8"
        )

        self.assertIn("arrange_materialized_virtual_list_children", source)
        self.assertIn("projection.logical_count()", source)
        self.assertIn("projection.logical_index_for_child(child_id)", source)
        self.assertIn("fixed_extent_virtual_list_content_extent", source)
        self.assertNotIn("0..projection.logical_count()", source)

    def test_lower_contract_covers_position_extent_and_bounded_visitation(self) -> None:
        source = ARRANGE_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "materialized_virtual_list_places_physical_slot_at_logical_offset", source
        )
        self.assertIn(
            "materialized_virtual_list_keeps_logical_content_extent", source
        )
        self.assertIn(
            "materialized_virtual_list_arrangement_visits_only_physical_slots", source
        )


if __name__ == "__main__":
    unittest.main()
