from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PLUGIN_RS = ROOT / "zircon_plugins/navigation/editor/src/plugin.rs"


class NavigationEditorContractTests(unittest.TestCase):
    def test_host_contract_marker_uses_host_module_surface(self) -> None:
        source = PLUGIN_RS.read_text(encoding="utf-8")

        self.assertIn(
            "zircon_editor::ui::host::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY",
            source,
        )
        self.assertNotIn("zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY", source)


if __name__ == "__main__":
    unittest.main()
