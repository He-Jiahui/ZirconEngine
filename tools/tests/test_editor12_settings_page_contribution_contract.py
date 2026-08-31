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
        self.assertIn("localization_bundle_id: EditorLocalizationBundleId", page)
        self.assertIn("label_key: EditorLocalizationKey", page)
        self.assertIn("category_keys: Vec<EditorLocalizationKey>", page)
        self.assertNotIn("display_name: String", page)
        self.assertNotIn("category_path: String", page)
        self.assertNotIn("is_valid_category_path", page)
        self.assertIn("pub use page::SettingsPageDescriptor;", settings_mod)
        self.assertIn("localization_bundles: BTreeMap<String, EditorLocalizationBundle>", extensions)
        self.assertIn("settings_pages: BTreeMap<String, SettingsPageDescriptor>", extensions)
        self.assertIn("pub fn register_localization_bundle", extensions)
        self.assertIn("pub fn register_settings_page", extensions)
        self.assertIn("pub fn settings_pages(&self) -> Vec<&SettingsPageDescriptor>", extensions)
        self.assertIn("SerializedEditorContribution::SettingsPage", materializer)
        self.assertIn("SerializedEditorContribution::LocalizationBundle", materializer)
        self.assertNotIn("Unsupported {\n                kind: \"settings page\"", materializer)


if __name__ == "__main__":
    unittest.main()
