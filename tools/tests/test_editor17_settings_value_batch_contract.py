import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class Editor17SettingsValueBatchContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_catalog_indexes_category_keys_once_and_authority_reads_one_batch(self) -> None:
        catalog = self.read(
            "zircon_editor/src/core/settings/catalog/settings_catalog.rs"
        )
        authority = self.read("zircon_editor/src/core/settings/authority.rs")
        batch = self.read(
            "zircon_editor/src/core/settings/catalog/resolved_settings_batch.rs"
        )

        self.assertIn("category_index: BTreeMap<Arc<str>, Arc<[SettingsKey]>>", catalog)
        self.assertIn("pub fn keys_for_category_path", catalog)
        self.assertIn("pub fn resolved_settings(", authority)
        method = authority.split("pub fn resolved_settings(", 1)[1].split(
            "\n    }", 1
        )[0]
        self.assertEqual(method.count("self.lock_state()"), 1)
        self.assertIn("ResolvedSettingsBatch::from_registry", method)
        self.assertIn("generation: registry.revision", batch)
        self.assertIn("Arc<[ResolvedSettingValue]>", batch)

    def test_workbench_reads_only_the_selected_category_value_batch(self) -> None:
        access = self.read(
            "zircon_editor/src/ui/host/editor_event_runtime_access/"
            "settings_projection.rs"
        )
        open_action = self.read(
            "zircon_editor/src/ui/retained_host/app/settings_window_actions.rs"
        )
        option_action = self.read(
            "zircon_editor/src/ui/retained_host/app/pane_surface_actions/"
            "workbench_surface/option.rs"
        )
        bridge = self.read(
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
            "workbench/settings_window.rs"
        )

        self.assertIn("capture_settings_values_for_category", access)
        self.assertIn("keys_for_category_path", access)
        self.assertIn("capture_settings_values_for_category", open_action)
        self.assertIn("capture_settings_values_for_category", option_action)
        self.assertIn('const SETTINGS_VALUES: &str = "settings_values";', bridge)
        self.assertIn("ResolvedSettingsBatch", bridge)
        self.assertIn("settings_value_payload", bridge)

    def test_retained_projection_carries_generation_value_and_source(self) -> None:
        component = self.read(
            "zircon_editor/assets/ui/editor/components/workbench/floating/"
            "workbench_preferences.zui"
        )
        projection = self.read(
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
            "pane_component_projection/settings_window/mod.rs"
        )
        data = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/data/"
            "template_nodes/settings.rs"
        )

        self.assertIn("settings_values = []", component)
        self.assertIn("SETTINGS_VALUES", projection)
        self.assertIn("value_text", projection)
        self.assertIn("value_source", projection)
        self.assertIn("pub value_text: SharedString", data)
        self.assertIn("pub value_source: SharedString", data)

    def test_open_window_refresh_is_revision_gated_and_values_only_when_possible(self) -> None:
        extension_access = self.read(
            "zircon_editor/src/ui/host/editor_event_runtime_access/extension_access.rs"
        )
        app = self.read("zircon_editor/src/ui/retained_host/app.rs")
        actions = self.read(
            "zircon_editor/src/ui/retained_host/app/settings_window_actions.rs"
        )
        bridge = self.read(
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
            "workbench/settings_window.rs"
        )
        projection = self.read(
            "zircon_editor/src/ui/settings/settings_window_projection/"
            "settings_window_projection.rs"
        )
        projection_capture = self.read(
            "zircon_editor/src/ui/settings/settings_window_projection/capture.rs"
        )
        component = self.read(
            "zircon_editor/assets/ui/editor/components/workbench/floating/"
            "workbench_preferences.zui"
        )

        self.assertIn("extension_projection_revision", extension_access)
        self.assertNotIn("plugin_template_revision", extension_access)
        self.assertIn("plugin_template_generation", app)
        self.assertIn("plugin_template_capabilities", app)
        self.assertIn("sync_open_settings_window", actions)
        self.assertIn("settings_window_revision", bridge)
        self.assertIn("refresh_settings_window", bridge)
        self.assertIn("refresh_settings_values", bridge)
        self.assertIn("contribution_generation", component)
        self.assertIn("enabled_capabilities", component)
        self.assertIn("enabled_capabilities: CapabilitySet", projection)
        self.assertIn(
            "&& self.enabled_capabilities == *capabilities", projection_capture
        )

        sync = actions.split("fn sync_open_settings_window", 1)[1]
        self.assertIn("extension_projection_revision", sync)
        self.assertIn("capture_settings_window_projection", sync)
        self.assertIn("capture_settings_values_for_category", sync)
        self.assertLess(
            sync.index("settings_window_revision"),
            sync.index("extension_projection_revision"),
        )
        self.assertLess(
            sync.index("if directory_is_stale"),
            sync.index("else if values_are_stale"),
        )

    def test_extension_consumers_keep_independent_accepted_revision_caches(self) -> None:
        app = self.read("zircon_editor/src/ui/retained_host/app.rs")
        assembly = self.read(
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/"
            "state/construction/assembly.rs"
        )

        for source in (app, assembly):
            self.assertIn("plugin_template_generation", source)
            self.assertIn("plugin_template_capabilities", source)
            self.assertNotIn("extension_projection_generation", source)
            self.assertNotIn("extension_projection_capabilities", source)

    def test_retained_app_keeps_plugin_template_sync_in_a_named_owner(self) -> None:
        app = self.read("zircon_editor/src/ui/retained_host/app.rs")
        owner = self.read(
            "zircon_editor/src/ui/retained_host/app/plugin_template_documents.rs"
        )

        self.assertLessEqual(len(app.splitlines()), 800)
        self.assertIn("mod plugin_template_documents;", app)
        self.assertNotIn("fn sync_plugin_template_documents_if_changed", app)
        self.assertIn("fn sync_plugin_template_documents_if_changed", owner)
        self.assertIn("extension_projection_revision", owner)

    def test_viewport_cannot_construct_a_second_settings_authority_in_production(self) -> None:
        construction = self.read(
            "zircon_editor/src/scene/viewport/controller/"
            "scene_viewport_controller_construction.rs"
        )

        self.assertIn(
            "#[cfg(test)]\n    pub(crate) fn new(viewport_size: UVec2) -> Self",
            construction.replace("\r\n", "\n"),
        )
        production = construction.split("#[cfg(test)]", 1)[0]
        self.assertNotIn("SettingsAuthority::with_defaults", production)


if __name__ == "__main__":
    unittest.main()
