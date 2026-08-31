from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
BUTTON = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch"
)


class EditorNativePointerButtonReplyPerformanceContractTests(unittest.TestCase):
    def test_capture_release_precedes_presentation_generation_read(self) -> None:
        entry = (BUTTON / "entry/sequence/entry.rs").read_text(encoding="utf-8")
        steps = (BUTTON / "entry/sequence/steps.rs").read_text(encoding="utf-8")

        capture = entry.index("if let Some(result) = finish_primary_capture_if_released")
        generation = entry.index("let input = button_dispatch_input")

        self.assertLess(capture, generation)
        self.assertNotIn("finish_release_capture_step", steps)

    def test_viewport_toolbar_damage_borrows_the_routed_control_id(self) -> None:
        click = (
            BUTTON / "pane_callbacks/viewport/toolbar/click.rs"
        ).read_text(encoding="utf-8")
        entry = (
            BUTTON / "pane_callbacks/viewport/toolbar/entry.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("-> &'a str", click)
        self.assertNotIn(".to_string()", click)
        self.assertNotIn("routed_control_id.as_str()", entry)

    def test_overflow_popup_padding_preserves_only_real_focus_damage(self) -> None:
        source = (BUTTON / "page_overflow_menu.rs").read_text(encoding="utf-8")
        branch = source.split(
            "if host_page_overflow_popup_frame_contains(&popup, x, y)", 1
        )[1].split(
            "if contains(", 1
        )[0]

        self.assertIn("NativePointerDispatchResult::idle()", branch)
        self.assertNotIn("NativePointerDispatchResult::region(popup)", branch)
        self.assertIn("cleared_text_input_frame", branch)

    def test_close_prompt_rejects_non_primary_press_before_action_allocation(self) -> None:
        source = (BUTTON / "close_prompt.rs").read_text(encoding="utf-8")

        gate = source.index(
            "state == NativePointerButtonState::Pressed"
        )
        action = source.index("if let Some(action_id) = close_prompt_action_at")

        self.assertLess(gate, action)

    def test_template_release_rejects_before_owned_hit_materialization(self) -> None:
        source = (
            BUTTON / "pane_callbacks/target/template.rs"
        ).read_text(encoding="utf-8")

        gate = source.index("state != NativePointerButtonState::Pressed")
        materialize = source.index("hit.to_owned_hit()")

        self.assertLess(gate, materialize)
        self.assertEqual(source.count("to_owned_hit()"), 1)


if __name__ == "__main__":
    unittest.main()
