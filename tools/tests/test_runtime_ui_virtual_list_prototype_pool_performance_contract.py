from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "zircon_runtime/src/ui/surface/surface.rs"
PROTOTYPE_POOL = ROOT / "zircon_runtime/src/ui/surface/virtual_list_prototype_pool.rs"


class RuntimeUiVirtualListPrototypePoolPerformanceContractTests(unittest.TestCase):
    def test_surface_owns_a_nonserialized_prototype_pool(self) -> None:
        surface = SURFACE.read_text(encoding="utf-8")

        self.assertIn("UiVirtualListPrototypePoolIndex", surface)
        self.assertRegex(
            surface,
            r"#\[serde\(default, skip\)\]\s+pub\(super\) virtual_list_prototype_pool",
        )

    def test_pool_materializes_only_physical_slot_capacity(self) -> None:
        self.assertTrue(PROTOTYPE_POOL.exists())
        source = PROTOTYPE_POOL.read_text(encoding="utf-8")

        self.assertIn("pub fn ensure_virtual_list_prototype_slots", source)
        self.assertIn("while state.slot_root_ids.len() < slot_capacity", source)
        self.assertIn("while state.slot_root_ids.len() > slot_capacity", source)
        self.assertNotIn("tree.nodes.iter()", source)
        self.assertNotIn("0..logical_count", source)

    def test_blueprint_preserves_complete_subtree_and_slot_topology(self) -> None:
        source = PROTOTYPE_POOL.read_text(encoding="utf-8")

        self.assertIn("struct UiVirtualListPrototypeBlueprint", source)
        self.assertIn("prototype_node_ids", source)
        self.assertIn("internal_slots", source)
        self.assertIn("clone_slot_subtree", source)
        self.assertIn("prototype_pool_clones_complete_subtree_for_each_physical_slot", source)

    def test_lower_contract_covers_identity_bounds_and_reuse(self) -> None:
        source = PROTOTYPE_POOL.read_text(encoding="utf-8")

        self.assertIn("unchanged_capacity_preserves_slot_roots_without_new_nodes", source)
        self.assertIn("logical_count_does_not_change_physical_tree_size", source)
        self.assertIn("shrinking_then_growing_reuses_bounded_subtrees", source)


if __name__ == "__main__":
    unittest.main()
