import unittest
from pathlib import Path


class RuntimeWidgetMenuBehaviorTestStructureTests(unittest.TestCase):
    def test_control_anchored_overlay_tests_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        behavior_path = repo_root / "zircon_runtime/src/ui/tests/widget_menu_behavior.rs"
        overlay_path = (
            repo_root
            / "zircon_runtime/src/ui/tests/widget_menu_behavior/control_anchored_overlays.rs"
        )

        behavior = behavior_path.read_text(encoding="utf-8")
        overlays = overlay_path.read_text(encoding="utf-8")

        self.assertLessEqual(len(behavior.splitlines()), 800)
        self.assertLessEqual(len(overlays.splitlines()), 800)
        self.assertIn("mod control_anchored_overlays;", behavior)
        self.assertEqual(behavior.count("#[test]"), 11)
        self.assertEqual(overlays.count("#[test]"), 5)
        for test_name in (
            "control_anchored_popup_routes_rendered_menu_items_and_rejects_old_placeholder_hits",
            "control_anchored_popup_frame_hit_grid_is_the_instance_authority",
            "parent_input_policy_incremental_patch_updates_descendant_frame_authority",
            "control_anchored_popup_escape_dismissal_restores_trigger_focus",
            "control_anchored_dropdown_routes_actual_option_overlay_and_dismisses_old_placeholder",
        ):
            self.assertNotIn(f"fn {test_name}", behavior)
            self.assertIn(f"fn {test_name}", overlays)

        for anchor in (
            "component_event: super::typed_component_event_kind_for_test(id)",
            "mode: Default::default()",
        ):
            self.assertIn(anchor, behavior)


if __name__ == "__main__":
    unittest.main()
