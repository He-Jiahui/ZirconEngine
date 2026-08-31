import unittest
from pathlib import Path


class RuntimeUiWinitTranslationOwnerStructureTests(unittest.TestCase):
    STATUS = (
        "runtime_09_15_winit_translation_domain_owner_split_"
        "static_passed_cargo_product_profile_deferred"
    )

    def test_winit_translation_domains_are_child_owned(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        owner_path = (
            repo_root / "zircon_runtime/src/ui/platform_input/winit_translation.rs"
        )
        owner = owner_path.read_text(encoding="utf-8")

        # The root retains its existing inline behavior tests; production routing is under 100 lines.
        self.assertLessEqual(len(owner.splitlines()), 540)
        for module in ("ime", "keyboard", "pointer", "window"):
            self.assertIn(f"mod {module};", owner)

        for moved_anchor in (
            "fn translate_keyboard_event",
            "fn translate_pointer_moved",
            "fn translate_ime_event",
            "fn translate_mouse_wheel_event",
            "fn window_metadata",
            "fn clamp_byte_index",
        ):
            self.assertNotIn(moved_anchor, owner)

        for retained_anchor in (
            "pub fn translate_winit_window_event",
            "pub use keyboard::translate_winit_modifiers;",
            "WindowEvent::PointerMoved",
            "WindowEvent::KeyboardInput",
            "WindowEvent::Ime",
            "WindowEvent::MouseWheel",
        ):
            self.assertIn(retained_anchor, owner)

        child_contracts = {
            "keyboard.rs": (
                80,
                "pub fn translate_winit_modifiers",
                "pub(super) fn translate_keyboard_event",
                "native_scan_code",
            ),
            "pointer.rs": (
                190,
                "pub(super) fn translate_pointer_moved",
                "pub(super) fn translate_pointer_button",
                "PIXEL_SCROLL_LINE_DELTA_SCALE",
            ),
            "ime.rs": (
                90,
                "pub(super) fn translate_ime_event",
                "UiTextByteRange::new",
                "fn clamp_byte_index",
            ),
            "window.rs": (
                100,
                "pub(super) fn input_event",
                "pub(super) fn window_event",
                "pub(super) fn window_metrics_from_physical_size",
            ),
        }
        owner_dir = owner_path.with_suffix("")
        for filename, anchors in child_contracts.items():
            budget, *required = anchors
            child = (owner_dir / filename).read_text(encoding="utf-8")
            self.assertLessEqual(len(child.splitlines()), budget, filename)
            for anchor in required:
                self.assertIn(anchor, child, filename)

    def test_winit_translation_owner_status_is_mirrored(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        mirrors = (
            repo_root
            / "docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md",
            repo_root
            / "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            repo_root / "docs/plans/engine-code-structure-convention.md",
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md",
        )
        for mirror_path in mirrors:
            mirror = mirror_path.read_text(encoding="utf-8")
            self.assertIn(self.STATUS, mirror, mirror_path.as_posix())

        structure_plan = mirrors[1].read_text(encoding="utf-8")
        for current_path in (
            "zircon_runtime/src/ui/platform_input/winit_translation.rs",
            "zircon_runtime/src/ui/platform_input/winit_translation/keyboard.rs",
            "zircon_runtime/src/ui/platform_input/winit_translation/pointer.rs",
            "zircon_runtime/src/ui/platform_input/winit_translation/ime.rs",
            "zircon_runtime/src/ui/platform_input/winit_translation/window.rs",
            "tools/tests/test_runtime_ui_winit_translation_owner_structure.py",
        ):
            self.assertIn(current_path, structure_plan)


if __name__ == "__main__":
    unittest.main()
