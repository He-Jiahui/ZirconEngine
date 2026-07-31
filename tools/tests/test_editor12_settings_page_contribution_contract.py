from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SETTINGS = ROOT / "zircon_editor" / "src" / "core" / "settings"
EXTENSIONS = ROOT / "zircon_editor" / "src" / "core" / "editor_extension.rs"
MATERIALIZER = ROOT / "zircon_editor" / "src" / "core" / "plugin" / "materializer.rs"


class SettingsPageContributionContractTests(unittest.TestCase):
    def test_settings_page_is_owned_by_settings_and_registered_once(self) -> None:
        page = (SETTINGS / "page.rs").read_text(encoding="utf-8")
        settings_mod = (SETTINGS / "mod.rs").read_text(encoding="utf-8")
        extensions = EXTENSIONS.read_text(encoding="utf-8")
        materializer = MATERIALIZER.read_text(encoding="utf-8")

        self.assertIn("pub struct SettingsPageDescriptor", page)
        self.assertIn("pub use page::SettingsPageDescriptor;", settings_mod)
        self.assertIn("settings_pages: BTreeMap<String, SettingsPageDescriptor>", extensions)
        self.assertIn("pub fn register_settings_page", extensions)
        self.assertIn("pub fn settings_pages(&self) -> Vec<&SettingsPageDescriptor>", extensions)
        self.assertIn("SerializedEditorContribution::SettingsPage", materializer)
        self.assertNotIn("Unsupported {\n                kind: \"settings page\"", materializer)


if __name__ == "__main__":
    unittest.main()
