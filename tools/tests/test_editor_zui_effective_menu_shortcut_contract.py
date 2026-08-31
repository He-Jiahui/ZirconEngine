import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
VIEW_MODEL = REPO_ROOT / (
    "zircon_editor/src/ui/workbench/model/workbench_view_model.rs"
)
MODEL_BUILD = REPO_ROOT / (
    "zircon_editor/src/ui/workbench/model/build/workbench_view_model_build.rs"
)
MENU_BUILD = REPO_ROOT / (
    "zircon_editor/src/ui/workbench/model/menu/default_menu_bar.rs"
)
REFLECTION = REPO_ROOT / (
    "zircon_editor/src/ui/host/editor_event_runtime_reflection.rs"
)
MAIN_MENU = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/asset_creation_menu.rs"
)


class EditorEffectiveMenuShortcutContractTests(unittest.TestCase):
    def test_workbench_model_uses_the_manager_effective_keymap(self):
        view_model = VIEW_MODEL.read_text(encoding="utf-8")
        model_build = MODEL_BUILD.read_text(encoding="utf-8")
        reflection = REFLECTION.read_text(encoding="utf-8")

        self.assertIn("pub keymap: EditorKeymap", view_model)
        self.assertIn("keymap: &EditorKeymap", model_build)
        self.assertIn("keymap: keymap.clone()", model_build)
        self.assertIn("let keymap = shell.manager.keymap();", reflection)
        self.assertIn("commands,\n            &keymap,", reflection)

    def test_menu_text_and_keyboard_trigger_share_one_effective_chord_authority(self):
        menu_build = MENU_BUILD.read_text(encoding="utf-8")
        main_menu = MAIN_MENU.read_text(encoding="utf-8")

        self.assertIn("apply_effective_shortcuts(&mut menu_bar, keymap)", menu_build)
        self.assertRegex(menu_build, r"keymap\s*\.chord_for_command")
        self.assertIn("MainMenuShortcutSignature", main_menu)
        self.assertIn('chord_for_command("file.project.open")', main_menu)
        self.assertIn('chord_for_command("file.project.save")', main_menu)
        self.assertIn('chord_for_command("editor.command.palette")', main_menu)
        self.assertNotIn('"Open Project|action=menu.item.open_project,icon=folder|Ctrl+O"', main_menu)


if __name__ == "__main__":
    unittest.main()
