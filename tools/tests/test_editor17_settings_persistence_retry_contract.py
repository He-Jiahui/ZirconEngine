import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class Editor17SettingsPersistenceRetryContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_runtime_retry_stays_on_the_settings_mutation_coordinator(self) -> None:
        action_ids = self.read("zircon_editor/src/ui/settings/action_ids.rs")
        runtime = self.read(
            "zircon_editor/src/ui/host/editor_event_runtime_access/settings_mutation.rs"
        )

        self.assertIn("SETTINGS_RETRY_PERSISTENCE_ACTION_ID", action_ids)
        self.assertIn("retry_settings_persistence", runtime)
        self.assertIn("SettingsScope::Project", runtime)
        self.assertIn("SettingsScope::User", runtime)
        self.assertIn(".settings_mutations()", runtime)
        self.assertIn(".retry_pending(scope)", runtime)

    def test_named_health_projection_selects_project_before_user_without_scanning_settings(self) -> None:
        projection = self.read(
            "zircon_editor/src/ui/settings/persistence_health_projection.rs"
        )

        self.assertIn("SettingsPersistenceHealthProjection", projection)
        self.assertIn("snapshot.project()", projection)
        self.assertIn("snapshot.user()", projection)
        self.assertLess(projection.index("snapshot.project()"), projection.index("snapshot.user()"))
        self.assertIn("status().is_retryable()", projection)
        self.assertNotIn("resolved_settings", projection)
        self.assertNotIn("diagnostics()", projection)

    def test_zui_bridge_and_pane_carry_one_health_generation_and_retry_scope(self) -> None:
        zui = self.read(
            "zircon_editor/assets/ui/editor/components/workbench/floating/workbench_preferences.zui"
        )
        bridge = self.read(
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/settings_window.rs"
        )
        pane = self.read(
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/settings_window/mod.rs"
        )
        node = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/data/template_nodes/node.rs"
        )

        for name in [
            "settings_persistence_health_generation",
            "settings_persistence_retry_scope",
            "settings_persistence_status_text",
        ]:
            self.assertIn(name, zui)
            self.assertIn(name.upper(), bridge)
            self.assertIn(name.upper(), pane)
        self.assertIn("prepare_settings_persistence_health", bridge)
        self.assertIn("settings_persistence_retry_scope", node)

    def test_retry_is_a_title_bar_icon_action_with_direct_hit_testing(self) -> None:
        geometry = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/settings_window_geometry.rs"
        )
        hit = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows/settings.rs"
        )
        paint = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_settings_window/persistence_health.rs"
        )

        self.assertIn("persistence_retry", geometry)
        self.assertIn("SETTINGS_RETRY_PERSISTENCE_ACTION_ID", hit)
        self.assertIn("settings_persistence_retry_scope", hit)
        self.assertIn('"refresh-outline"', paint)
        self.assertIn("settings_persistence_status_text", paint)
        self.assertTrue(
            (
                ROOT
                / "zircon_editor/assets/icons/ionicons/refresh-outline.svg"
            ).is_file()
        )

    def test_notification_change_prepares_health_and_one_retained_refresh(self) -> None:
        notifications = self.read(
            "zircon_editor/src/ui/retained_host/app/workbench_notifications.rs"
        )
        option = self.read(
            "zircon_editor/src/ui/retained_host/app/pane_surface_actions/workbench_surface/option.rs"
        )

        self.assertIn("capture_settings_persistence_health_projection", notifications)
        self.assertIn("prepare_settings_persistence_health", notifications)
        self.assertIn("notification_changed || health_changed", notifications)
        self.assertNotIn("diagnostics()", notifications)
        self.assertEqual(notifications.count("refresh_prepared_state_change()"), 1)
        self.assertIn("SETTINGS_RETRY_PERSISTENCE_ACTION_ID", option)
        self.assertIn("retry_settings_persistence", option)
        self.assertIn("prepare_settings_persistence_health", option)


if __name__ == "__main__":
    unittest.main()
