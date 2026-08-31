import unittest
from pathlib import Path


class RuntimeUiPointerComponentStateOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_09_15_ui_pointer_component_state_owner_split_"
        "static_passed_cargo_profile_deferred"
    )

    def test_pointer_component_state_is_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/ui/surface/surface/pointer_component_events.rs"
        )
        child_path = (
            repo_root
            / "zircon_runtime/src/ui/surface/surface/pointer_component_events/state_invalidation.rs"
        )

        owner = owner_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 800)
        self.assertIn("mod state_invalidation;", owner)
        self.assertIn("mod template_action;", owner)
        for moved in (
            "fn apply_pointer_component_state",
            "fn apply_pointer_transient_state_dirty",
            "fn set_node_pressed_dirty",
            "fn mark_component_state_render_dirty",
            "fn mark_component_states_render_dirty",
            "fn minimal_changed_subtree_roots",
            "fn node_is_covered_by_roots",
        ):
            self.assertNotIn(moved, owner)

        child = child_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(child.splitlines()), 800)
        for anchor in (
            "pub(in crate::ui::surface::surface) fn apply_pointer_component_state",
            "pub(in crate::ui::surface::surface) fn apply_pointer_transient_state_dirty",
            "fn set_node_pressed_dirty",
            "pub(crate) fn mark_component_state_render_dirty",
            "pub(crate) fn mark_component_states_render_dirty",
            "fn minimal_changed_subtree_roots",
            "fn node_is_covered_by_roots",
            "set_hovered",
            "set_pressed",
            "set_focused",
            "node_state_can_affect_descendants",
            "apply_runtime_state_style_subtree",
            "apply_runtime_state_style_node",
            "mark_node_dirty",
        ):
            self.assertIn(anchor, child)

        for event_owner_anchor in (
            "pub(super) fn pointer_component_events",
            "pub(super) fn push_state_damage_frames",
            "pub(super) fn push_pointer_component_events_with_drag_metrics",
            "compiled_binding_event_sources",
        ):
            self.assertIn(event_owner_anchor, owner)

        action_child = (
            owner_path.parent / "pointer_component_events/template_action.rs"
        ).read_text(encoding="utf-8")
        for action_owner_anchor in (
            "template_action_for_compiled_binding_with_overrides",
            "dense_compiled_payload_overrides_for_benchmark",
            "compiled_binding_handle_for_source",
        ):
            self.assertIn(action_owner_anchor, action_child)

    def test_pointer_component_state_owner_status_is_mirrored(self) -> None:
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
            "zircon_runtime/src/ui/surface/surface/pointer_component_events.rs",
            "zircon_runtime/src/ui/surface/surface/pointer_component_events/state_invalidation.rs",
            "tools/tests/test_runtime_ui_pointer_component_state_owner_structure.py",
        ):
            self.assertIn(current_path, runtime_plan)


if __name__ == "__main__":
    unittest.main()
