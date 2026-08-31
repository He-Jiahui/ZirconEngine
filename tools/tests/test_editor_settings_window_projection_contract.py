import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class EditorSettingsWindowProjectionContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_settings_catalog_is_shared_and_selected_values_are_versioned(self) -> None:
        snapshot = self.read("zircon_editor/src/core/settings/snapshot.rs")
        authority = self.read("zircon_editor/src/core/settings/authority.rs")
        catalog = self.read(
            "zircon_editor/src/core/settings/catalog/settings_catalog.rs"
        )

        self.assertIn("catalog: Arc<SettingsCatalog>", snapshot)
        self.assertIn("catalog: Arc::clone(&previous.catalog)", snapshot)
        self.assertIn("pub fn resolved_setting(", authority)
        self.assertIn("state.registry.resolve_with_source(key)?", authority)
        self.assertNotIn("registry.clone()", authority)
        self.assertIn("binary_search_by", catalog)

    def test_window_projection_uses_locale_and_domain_identity_not_value_rebuilds(self) -> None:
        projection = self.read(
            "zircon_editor/src/ui/settings/settings_window_projection/capture.rs"
        )
        projection_state = self.read(
            "zircon_editor/src/ui/settings/settings_window_projection/"
            "settings_window_projection.rs"
        )
        plugin_projection = self.read(
            "zircon_editor/src/core/extension/settings_page_projection.rs"
        )

        self.assertIn("let locale = i18n.active_locale();", projection)
        self.assertIn("SettingsPageProjection::capture_for_locale", projection)
        self.assertIn("settings_catalog: settings.catalog_handle()", projection)
        self.assertIn("SettingsLocalizationDomain::BuiltIn", projection)
        self.assertIn("SettingsLocalizationDomain::Plugin", projection)
        self.assertIn("shares_catalog_handle_with", projection_state)
        self.assertNotIn(
            "self.settings_generation == snapshot.generation()", projection_state
        )
        self.assertIn("localization_bundle_id", plugin_projection)
        self.assertIn("Vec<Arc<str>>, Arc<str>", plugin_projection)


if __name__ == "__main__":
    unittest.main()
