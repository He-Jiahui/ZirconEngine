import unittest
from pathlib import Path


class RuntimeUiSurfacePropertyTransactionOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_09_15_ui_surface_property_transaction_owner_split_"
        "static_passed_cargo_profile_deferred"
    )

    def test_property_transaction_is_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = repo_root / "zircon_runtime/src/ui/surface/surface.rs"
        child_path = (
            repo_root
            / "zircon_runtime/src/ui/surface/surface/property_transaction.rs"
        )

        owner = owner_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 800)
        self.assertIn("mod property_transaction;", owner)
        for moved in (
            "pub fn mutate_property",
            "fn mutate_property_with_popup_branch_close",
            "fn mutate_editable_text_property",
            "fn synchronize_open_popup_state",
            "fn reset_popup_open_state",
            "fn text_edit_property_is_tracked",
        ):
            self.assertNotIn(moved, owner)

        child = child_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(child.splitlines()), 800)
        for anchor in (
            "pub fn mutate_property",
            "pub(crate) fn dismiss_transient_ui",
            "pub(crate) fn dismiss_popup_by_id",
            "pub(crate) fn set_declarative_popup_open_by_id",
            "fn mutate_property_with_popup_branch_close",
            "fn mutate_editable_text_property",
            "fn synchronize_open_popup_state",
            "fn sync_popup_open_alias_state",
            "pub(crate) fn reject_runtime_anchored_popup",
            "fn reset_popup_open_state",
            "fn text_edit_property_is_tracked",
            "fn focus_reconcile_reason",
            "mutate_tree_property",
            "sync_from_property",
            "apply_runtime_state_style_subtree",
            "reconcile_focus_after_tree_change",
            "invalidate_clipboard_transfers_for",
            "commit_editable_text_properties_with_value",
            "sync_popup_stack_for_node",
            "mark_node_dirty",
        ):
            self.assertIn(anchor, child)

        for concurrent_anchor in (
            "mod compiled_binding_event_index;",
            "mod font_generation;",
            "compiled_binding_event_index",
            "observed_text_font_generation",
            "arranged_visibility",
            "virtual_list_materialization",
            "adopt_hot_reload_state_from",
            "debug_snapshot_for_selection",
            "pub fn reflector_snapshot",
        ):
            self.assertIn(concurrent_anchor, owner)

    def test_property_transaction_owner_status_is_mirrored(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            repo_root
            / "docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            repo_root
            / "docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/09/2026-08-07-runtime-ui-incremental-refresh.md",
            repo_root / "docs/zircon_runtime/ui/architecture.md",
            repo_root / "docs/plans/engine-code-structure-convention.md",
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md",
        )
        for mirror_path in mirrors:
            mirror = mirror_path.read_text(encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        runtime_plan = mirrors[1].read_text(encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/ui/surface/surface.rs",
            "zircon_runtime/src/ui/surface/surface/property_transaction.rs",
            "tools/tests/test_runtime_ui_surface_property_transaction_owner_structure.py",
        ):
            self.assertIn(current_path, runtime_plan)


if __name__ == "__main__":
    unittest.main()
