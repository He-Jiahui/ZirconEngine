import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
ENGINE_ENTRY = ROOT / "zircon_app/src/entry/engine_entry.rs"


class EngineEntryEditorConfigContractTests(unittest.TestCase):
    def test_editor_config_keys_use_the_public_host_namespace(self):
        source = ENGINE_ENTRY.read_text(encoding="utf-8")

        self.assertIn(
            "zircon_editor::ui::host::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY",
            source,
        )
        self.assertIn(
            "zircon_editor::ui::host::EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY",
            source,
        )
        self.assertNotIn(
            "zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY",
            source,
        )
        self.assertNotIn(
            "zircon_editor::EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY",
            source,
        )


if __name__ == "__main__":
    unittest.main()
