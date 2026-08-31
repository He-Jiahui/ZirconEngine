from pathlib import Path
import unittest

from tools.runtime_ui_virtual_list_logical_identity_pressure import run


ROOT = Path(__file__).resolve().parents[2]
MATERIALIZATION = ROOT / "zircon_runtime/src/ui/surface/virtual_list_materialization.rs"
IDENTITY = (
    ROOT
    / "zircon_runtime/src/ui/surface/virtual_list_materialization/identity.rs"
)


class RuntimeUiVirtualListLogicalIdentityPerformanceContractTests(unittest.TestCase):
    def test_identity_contract_is_split_from_the_materializer_owner(self) -> None:
        root = MATERIALIZATION.read_text(encoding="utf-8")
        identity = IDENTITY.read_text(encoding="utf-8")

        self.assertIn("mod identity;", root)
        self.assertIn("pub use identity::", root)
        self.assertIn("pub struct UiVirtualListItemIdentity", identity)
        self.assertIn("pub struct UiVirtualListNodeBinding", identity)
        self.assertIn("pub assignment_generation: u64", identity)
        self.assertIn("pub const fn item_identity", identity)

    def test_assignment_generation_changes_only_for_rebound_slots(self) -> None:
        source = MATERIALIZATION.read_text(encoding="utf-8")

        self.assertIn("slot_assignment_generations: Vec<u64>", source)
        self.assertIn("candidate_assignment_generations", source)
        self.assertIn("for change in changes.iter()", source)
        self.assertIn("candidate_assignment_generations[change.slot_index] = generation", source)
        self.assertNotIn("slot_assignment_generations.fill", source)

    def test_surface_can_reject_a_stale_physical_slot_binding(self) -> None:
        source = MATERIALIZATION.read_text(encoding="utf-8")

        self.assertIn("pub fn virtual_list_binding_is_current", source)
        self.assertIn("current == binding", source)
        self.assertIn("rebound_slot_rejects_its_previous_logical_identity", source)
        self.assertIn("unchanged_slot_preserves_its_assignment_generation", source)

    def test_pressure_model_counts_only_changed_slot_identity_refreshes(self) -> None:
        result = run(
            slot_count=41,
            scroll_update_count=4_096,
            large_seek_count=64,
        )

        self.assertEqual(result["owner_generation_token_refreshes"], 167_936)
        self.assertEqual(result["changed_slot_token_refreshes"], 6_656)
        self.assertEqual(result["preserved_unchanged_slot_tokens"], 161_280)
        self.assertEqual(result["token_refresh_reduction_ratio"], 25.23)
        self.assertFalse(result["runtime_cpu_measured"])
        self.assertFalse(result["accesskit_adapter_wired"])


if __name__ == "__main__":
    unittest.main()
