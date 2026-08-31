import unittest
from pathlib import Path


class RuntimeUiPointerTemplateActionOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_09_15_ui_pointer_template_action_owner_split_"
        "static_passed_cargo_profile_deferred"
    )

    def test_pointer_template_actions_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root
            / "zircon_runtime/src/ui/surface/surface/pointer_component_events.rs"
        )
        child_path = (
            repo_root
            / "zircon_runtime/src/ui/surface/surface/pointer_component_events/template_action.rs"
        )

        owner = owner_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 550)
        self.assertIn("mod template_action;", owner)
        for moved in (
            "fn template_action_for_binding",
            "fn template_action_for_binding_with_overrides",
            "fn template_action_for_compiled_binding_with_overrides",
            "fn dense_compiled_payload_overrides_for_benchmark",
            "fn template_action_for_compiled_binding_with_legacy_overrides_for_benchmark",
            "fn compiled_binding_handle_for_source",
            "fn template_action_payload_value",
            "fn template_action_control_property_value",
            "fn template_action_property_value",
        ):
            self.assertNotIn(moved, owner)

        child = child_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(child.splitlines()), 400)
        for anchor in (
            "pub(crate) fn template_action_for_binding",
            "pub(crate) fn template_action_for_binding_with_overrides",
            "pub(crate) fn template_action_for_compiled_binding_with_overrides",
            "pub(crate) fn dense_compiled_payload_overrides_for_benchmark",
            "pub(crate) fn template_action_for_compiled_binding_with_legacy_overrides_for_benchmark",
            "pub(crate) fn compiled_binding_handle_for_source",
            "pub(crate) fn template_action_payload_value",
            "pub(crate) fn template_action_control_property_value",
            "pub(crate) fn template_action_property_value",
            "UiBindingMissingValueResolution",
            "resolve_compiled_action_payload_value",
            "template_action_property_value",
        ):
            self.assertIn(anchor, child)

        for event_owner_anchor in (
            "pub(super) fn pointer_component_events",
            "pub(super) fn push_focus_component_events",
            "pub(super) fn push_state_damage_frames",
            "pub(super) fn push_pointer_component_events_with_drag_metrics",
            "fn push_pointer_component_event_for_binding",
        ):
            self.assertIn(event_owner_anchor, owner)

    def test_pointer_template_action_owner_status_is_mirrored(self) -> None:
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
            "zircon_runtime/src/ui/surface/surface/pointer_component_events/template_action.rs",
            "tools/tests/test_runtime_ui_pointer_template_action_owner_structure.py",
        ):
            self.assertIn(current_path, runtime_plan)


if __name__ == "__main__":
    unittest.main()
