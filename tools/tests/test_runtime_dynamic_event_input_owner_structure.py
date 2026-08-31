import unittest
from pathlib import Path


class RuntimeDynamicEventInputOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_10_12_15_dynamic_event_keyboard_ime_gamepad_owner_split_"
        "static_passed_cargo_deferred"
    )

    def test_keyboard_ime_and_gamepad_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = repo_root / "zircon_runtime/src/dynamic_api/session/events.rs"
        keyboard_path = (
            repo_root / "zircon_runtime/src/dynamic_api/session/events/keyboard_ime.rs"
        )
        gamepad_path = repo_root / "zircon_runtime/src/dynamic_api/session/events/gamepad.rs"

        owner = owner_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(owner.splitlines()), 800)
        self.assertIn("mod gamepad;", owner)
        self.assertIn("mod keyboard_ime;", owner)
        self.assertNotIn("fn handle_keyboard", owner)
        self.assertNotIn("fn handle_ime", owner)
        self.assertNotIn("fn handle_gamepad_connection", owner)
        self.assertNotIn("fn handle_gamepad_button", owner)
        self.assertNotIn("fn handle_gamepad_axis", owner)
        self.assertEqual(owner.count("#[test]"), 6)

        keyboard = keyboard_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(keyboard.splitlines()), 800)
        for anchor in (
            "pub(super) fn handle_keyboard",
            "pub(super) fn handle_ime",
            "UiImeInputEventKind::DeleteSurrounding",
            "self.submit_input_event(input_event)",
        ):
            self.assertIn(anchor, keyboard)

        gamepad = gamepad_path.read_text(encoding="utf-8")
        self.assertLessEqual(len(gamepad.splitlines()), 800)
        for anchor in (
            "pub(super) fn handle_gamepad_connection",
            "pub(super) fn handle_gamepad_button",
            "pub(super) fn handle_gamepad_axis",
            "pub(super) fn ui_gamepad_navigation",
            "pub(super) fn ui_gamepad_analog_control",
        ):
            self.assertIn(anchor, gamepad)

        for concurrent_anchor in (
            "ZR_RUNTIME_EVENT_KIND_VIEWPORT_CAMERA_V1",
            "ZR_RUNTIME_EVENT_KIND_EDITOR_TRANSFORM_WRITE_V1",
            "submit_clock_discontinuity",
            "record_submitted_pointer_move",
            "dispatch_runtime_ui_event(|metadata|",
        ):
            self.assertIn(concurrent_anchor, owner)

    def test_dynamic_event_input_owner_status_is_mirrored(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            repo_root
            / "docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            repo_root / "docs/zircon_runtime/dynamic_api/session.md",
            repo_root / "docs/zircon_runtime/input/input_state.md",
            repo_root / "docs/plans/engine-code-structure-convention.md",
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md",
        )
        for mirror_path in mirrors:
            mirror = mirror_path.read_text(encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        runtime_plan = mirrors[2].read_text(encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/dynamic_api/session/events.rs",
            "zircon_runtime/src/dynamic_api/session/events/keyboard_ime.rs",
            "zircon_runtime/src/dynamic_api/session/events/gamepad.rs",
            "tools/tests/test_runtime_dynamic_event_input_owner_structure.py",
        ):
            self.assertIn(current_path, runtime_plan)


if __name__ == "__main__":
    unittest.main()
