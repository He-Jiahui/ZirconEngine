from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "zircon_runtime/src/ui/surface/surface.rs"
MATERIALIZATION = (
    ROOT / "zircon_runtime/src/ui/surface/virtual_list_materialization.rs"
)
IDENTITY = (
    ROOT
    / "zircon_runtime/src/ui/surface/virtual_list_materialization/identity.rs"
)


class RuntimeUiVirtualListSurfaceMaterializationPerformanceContractTests(
    unittest.TestCase
):
    def test_surface_owns_nonserialized_materialization_assignments(self) -> None:
        surface = SURFACE.read_text(encoding="utf-8")

        self.assertIn("UiVirtualListMaterializationIndex", surface)
        self.assertIn("pub(super) virtual_list_materialization", surface)
        self.assertRegex(
            surface,
            r"#\[serde\(default, skip\)\]\s+pub\(super\) virtual_list_materialization",
        )

    def test_reconcile_is_bounded_by_slot_capacity_not_logical_count(self) -> None:
        self.assertTrue(
            MATERIALIZATION.exists(),
            "surface-owned virtual-list materialization module must exist",
        )
        source = MATERIALIZATION.read_text(encoding="utf-8")

        self.assertIn("UiVirtualListSlotMap", source)
        self.assertIn("fixed_extent_slot_capacity", source)
        self.assertIn("changes: &mut Vec<UiVirtualListMaterializationChange>", source)
        reconcile = source.split("    fn reconcile(", 1)[1].split(
            "    fn owner(", 1
        )[0]
        self.assertIn("changes.clear();", reconcile)
        self.assertIn("self.owners.remove(&owner_id)", reconcile)
        self.assertNotIn("0..request.logical_count", source)
        self.assertNotIn("tree.nodes.iter()", source)

    def test_lower_contract_covers_owner_validation_stability_and_cleanup(self) -> None:
        self.assertTrue(
            MATERIALIZATION.exists(),
            "surface-owned virtual-list materialization module must exist",
        )
        source = MATERIALIZATION.read_text(encoding="utf-8")

        self.assertIn("rejects_a_non_virtualized_owner", source)
        self.assertIn("one_row_scroll_rebinds_only_one_surface_owned_slot", source)
        self.assertIn("identical_request_preserves_surface_owned_generation", source)
        self.assertIn("removed_owner_state_is_pruned_without_scanning_logical_rows", source)
        self.assertIn(
            "invalidated_owner_evicts_assignments_and_clears_reused_changes", source
        )
        self.assertIn("descendant_binding_resolves_through_registered_slot", source)
        self.assertIn("captured_slot_rebind_is_rejected_before_assignment_commit", source)

    def test_physical_slot_registration_is_bounded_and_transactional(self) -> None:
        source = MATERIALIZATION.read_text(encoding="utf-8")

        self.assertIn("pub fn register_virtual_list_slots", source)
        self.assertIn("pub fn virtual_list_binding_for_node", source)
        self.assertIn("slot_node_ids: Vec<Vec<UiNodeId>>", source)
        self.assertIn("let mut candidate = state.slots.clone();", source)
        self.assertIn("ProtectedSlotRebind", source)
        self.assertNotIn("self.tree.nodes.iter()", source)

    def test_logical_item_identity_is_separate_from_recycled_node_identity(self) -> None:
        source = MATERIALIZATION.read_text(encoding="utf-8")
        identity = IDENTITY.read_text(encoding="utf-8")

        self.assertIn("pub struct UiVirtualListItemKey", identity)
        self.assertIn("pub struct UiVirtualListItemIdentity", identity)
        self.assertIn("pub struct UiVirtualListMaterializationChange", source)
        self.assertIn("reconcile_virtual_list_materialization_with_keys", source)
        self.assertIn("slot_item_keys: Vec<Option<UiVirtualListItemKey>>", source)
        self.assertIn("pub item_key: UiVirtualListItemKey", identity)
        self.assertIn("stable_item_key_follows_logical_item_across_slot_reuse", source)
        self.assertIn("key_only_rebind_advances_materialization_generation", source)


if __name__ == "__main__":
    unittest.main()
