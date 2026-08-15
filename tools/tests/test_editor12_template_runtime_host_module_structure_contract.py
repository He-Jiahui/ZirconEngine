from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RUNTIME_HOST = (
    ROOT / "zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs"
)
DYNAMIC_CONTROL_STATE = (
    ROOT
    / "zircon_editor/src/ui/template_runtime/runtime/runtime_host/dynamic_control_state.rs"
)


class TemplateRuntimeHostModuleStructureContractTests(unittest.TestCase):
    def test_runtime_host_delegates_dynamic_control_state_to_child_module(self) -> None:
        source = RUNTIME_HOST.read_text(encoding="utf-8")

        self.assertIn("mod dynamic_control_state;", source)
        self.assertNotIn("fn apply_template_control_property(", source)
        self.assertNotIn("fn apply_template_control_attributes_to_surface(", source)
        self.assertNotIn("fn project_pane_body(", source)

    def test_dynamic_control_state_module_owns_pane_projection_and_actions(self) -> None:
        self.assertTrue(DYNAMIC_CONTROL_STATE.is_file())
        source = DYNAMIC_CONTROL_STATE.read_text(encoding="utf-8")

        self.assertIn("impl EditorUiHostRuntime", source)
        for method in (
            "project_pane_body",
            "apply_pane_component_patches_to_surface",
            "bind_template_actions_for_pane",
            "update_template_action_control_state",
            "select_template_table_row",
            "remove_template_actions_for_pane",
            "dispatch_template_action_for_token",
            "apply_template_control_property",
        ):
            self.assertIn(f"fn {method}", source)

    def test_runtime_host_stays_within_production_warning_budget(self) -> None:
        line_count = len(RUNTIME_HOST.read_text(encoding="utf-8").splitlines())
        self.assertLessEqual(line_count, 800)


if __name__ == "__main__":
    unittest.main()
