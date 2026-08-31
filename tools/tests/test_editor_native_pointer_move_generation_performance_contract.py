from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RETAINED_HOST = ROOT / "zircon_editor/src/ui/retained_host"
MOVE = RETAINED_HOST / "host_contract/native_pointer/move_dispatch"


class EditorNativePointerMoveGenerationPerformanceContractTests(unittest.TestCase):
    def test_capture_returns_before_idle_hover_instrumentation(self) -> None:
        source = (MOVE / "entry.rs").read_text(encoding="utf-8")
        capture = source.index("dispatch_pointer_move_capture(ui, x, y)")
        idle = source.index("enter_ui_perf_scenario(UiPerfScenario::IdleHover)")

        self.assertLess(capture, idle)

    def test_pane_and_workbench_reuse_the_body_generation(self) -> None:
        body = (MOVE / "entry/body.rs").read_text(encoding="utf-8")
        pane = (MOVE / "pane/entry.rs").read_text(encoding="utf-8")
        workbench = (MOVE / "workbench.rs").read_text(encoding="utf-8")

        self.assertIn("dispatch_pane_pointer_move(ui, &generation, x, y)", body)
        self.assertIn("dispatch_workbench_template_hit(ui, &generation, hit)", body)
        self.assertIn("generation: &HostPresentationGeneration", pane)
        self.assertIn("generation: &HostPresentationGeneration", workbench)
        self.assertNotIn("let before = ui.get_host_presentation_generation()", pane)
        self.assertNotIn("let before = ui.get_host_presentation_generation()", workbench)
        self.assertIn("get_pane_interaction_generation", pane)
        self.assertIn("get_pane_interaction_generation", workbench)

    def test_clear_and_menu_use_narrow_interaction_reads(self) -> None:
        clear = (MOVE / "clear.rs").read_text(encoding="utf-8")
        menu = (MOVE / "menu.rs").read_text(encoding="utf-8")

        self.assertNotIn("get_pane_interaction_state()", clear)
        self.assertEqual(2, clear.count("get_pane_interaction_generation()"))
        self.assertIn("get_host_interaction_generation()", menu)
        self.assertNotIn("get_host_presentation_generation()", menu)

    def test_stable_template_hover_compares_before_owned_state_clone(self) -> None:
        setter = (
            RETAINED_HOST
            / "host_contract/window/presentation/template_hover_state.rs"
        ).read_text(encoding="utf-8")
        pane = (MOVE / "pane/template.rs").read_text(encoding="utf-8")
        workbench = (MOVE / "workbench/hover.rs").read_text(encoding="utf-8")

        first_update = setter.index("state.update_pane_interaction")
        self.assertLess(setter.index("hovered_template_control_id"), first_update)
        self.assertIn("control_id: &str", setter)
        self.assertIn("frame: &FrameRect", setter)
        self.assertNotIn("hit.control_id.clone()", pane + workbench)
        self.assertNotIn("hit.dispatch_kind.clone()", workbench)
        self.assertNotIn("hit.action_id.clone()", workbench)
        self.assertNotIn("hit.value_text.clone()", workbench)


if __name__ == "__main__":
    unittest.main()
