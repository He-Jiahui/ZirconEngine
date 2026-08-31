import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class EditorSettingsWindowProductWiringContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_settings_command_routes_through_one_explicit_host_effect(self) -> None:
        commands = self.read("zircon_editor/src/core/commands/defaults.rs")
        event_types = self.read("zircon_editor/src/core/editor_event/types.rs")
        dispatch = self.read(
            "zircon_editor/src/ui/host/editor_event_execution/dispatch.rs"
        )
        event_bridge = self.read("zircon_editor/src/ui/retained_host/event_bridge.rs")
        side_effects = self.read(
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/"
            "dispatch_effects/side_effects.rs"
        )

        self.assertIn('"editor.settings.open"', commands)
        self.assertIn('"Edit/Editor Settings"', commands)
        self.assertIn("OpenSettingsWindow", event_types)
        self.assertIn("SettingsWindowOpenRequested", event_types)
        self.assertIn("EditorEventEffect::SettingsWindowOpenRequested", dispatch)
        self.assertIn("open_settings_window_requested", event_bridge)
        self.assertIn("open_workbench_settings_window", side_effects)

    def test_product_projection_reads_only_current_authorities_on_open(self) -> None:
        access = self.read(
            "zircon_editor/src/ui/host/editor_event_runtime_access/"
            "settings_projection.rs"
        )
        action = self.read(
            "zircon_editor/src/ui/retained_host/app/settings_window_actions.rs"
        )

        self.assertIn("self.context().settings().snapshot()", access)
        self.assertIn("inner.contributions.snapshot()", access)
        self.assertIn("inner.manager.capability_snapshot()", access)
        self.assertIn("self.context().i18n()", access)
        self.assertIn("SettingsWindowProjection::capture", access)
        self.assertNotIn("SettingsAuthority::with_defaults", access)
        self.assertIn("capture_settings_window_projection", action)
        self.assertIn("WorkbenchSettingsOpenState::from_projection", action)

    def test_preferences_component_is_mounted_with_dynamic_catalog_props(self) -> None:
        window = self.read(
            "zircon_editor/assets/ui/editor/windows/workbench_window.zui"
        )
        preferences = self.read(
            "zircon_editor/assets/ui/editor/components/workbench/floating/"
            "workbench_preferences.zui"
        )
        bridge = self.read(
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
            "workbench/settings_window.rs"
        )

        self.assertIn("workbench_preferences.zui#WorkbenchPreferences", window)
        self.assertIn('{ node = "settings_window" }', window)
        for prop in ("categories", "settings", "plugin_pages", "settings_generation"):
            with self.subTest(prop=prop):
                self.assertIn(prop, preferences)
                self.assertIn(prop.upper(), bridge)
        self.assertIn("SettingsWindowProjection", bridge)
        self.assertIn("mutate_control_property", bridge)

    def test_preferences_use_typed_projection_and_bounded_native_painting(self) -> None:
        preferences = self.read(
            "zircon_editor/assets/ui/editor/components/workbench/floating/"
            "workbench_preferences.zui"
        )
        projection = self.read(
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
            "pane_component_projection/settings_window/mod.rs"
        )
        painter = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_settings_window/commands.rs"
        )
        dispatch = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
            "template_nodes/specialized/secondary.rs"
        )
        hit_test = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
            "template_node/popup_rows/settings.rs"
        )
        selection = self.read(
            "zircon_editor/src/ui/retained_host/app/pane_surface_actions/"
            "workbench_surface/option.rs"
        )
        actions = self.read("zircon_editor/src/ui/settings/action_ids.rs")

        self.assertNotIn("WorkbenchPreferencesGeneral", preferences)
        self.assertNotIn("WorkbenchPreferencesLayout", preferences)
        self.assertIn("projected_settings_window_data", projection)
        self.assertIn('component_role != "settings-window"', projection)
        self.assertIn("settings_window_visible_rows", painter)
        self.assertIn("SETTINGS_WINDOW_PAINT_OVERSCAN_ROWS", painter)
        self.assertIn("node.settings_entries.get(row)", painter)
        self.assertNotIn("0..row_count", painter)
        self.assertIn("push_settings_window_commands", dispatch)
        self.assertIn("SettingsWindowLayout", hit_test)
        self.assertIn("node.settings_categories", hit_test)
        self.assertIn(".get(row)", hit_test)
        self.assertIn("SETTINGS_CATEGORY_CHANGED_ACTION_ID", hit_test)
        self.assertIn("editor.settings.category_changed", actions)
        self.assertIn("select_settings_category", selection)


if __name__ == "__main__":
    unittest.main()
