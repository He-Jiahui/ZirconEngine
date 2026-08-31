import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class Editor17SettingsPersistenceHealthContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_runtime_terminal_observer_feeds_the_settings_health_owner(self) -> None:
        persistence = self.read("zircon_editor/src/core/settings/persistence.rs")

        self.assertIn("submit_observed", persistence)
        self.assertIn("retry_observed", persistence)
        self.assertIn("admission.observe_terminal", persistence)

    def test_health_is_a_named_child_owner_with_stale_observation_rejection(self) -> None:
        mutation = self.read("zircon_editor/src/core/settings/mutation.rs")
        health = self.read("zircon_editor/src/core/settings/mutation/health.rs")

        self.assertIn("mod health;", mutation)
        self.assertIn("SettingsPersistenceHealthSnapshot", health)
        self.assertIn("SettingsPersistenceHealthSubscriber", health)
        self.assertIn("active_observation", health)
        self.assertIn("observation.document", health)
        self.assertIn("observation.id", health)

    def test_coordinator_exposes_a_snapshot_and_change_subscriber_not_lane_diagnostics(self) -> None:
        mutation = self.read("zircon_editor/src/core/settings/mutation.rs")
        settings_module = self.read("zircon_editor/src/core/settings/mod.rs")
        settings_window = self.read(
            "zircon_editor/src/ui/retained_host/app/settings_window_actions.rs"
        )

        self.assertIn("persistence_health_snapshot", mutation)
        self.assertIn("configure_persistence_health_subscriber", mutation)
        self.assertIn("SettingsPersistenceHealthSnapshot", settings_module)
        self.assertNotIn(".diagnostics()", settings_window)

    def test_context_subscriber_projects_only_actionable_failures_to_notifications(self) -> None:
        builder = self.read("zircon_editor/src/core/context/builder.rs")
        context = self.read("zircon_editor/src/core/context/editor_context.rs")
        projection = self.read(
            "zircon_editor/src/core/context/builder/settings_persistence_health.rs"
        )
        english = self.read("zircon_editor/assets/i18n/en.toml")
        chinese = self.read("zircon_editor/assets/i18n/zh-CN.toml")

        self.assertIn("EditorSettingsPersistenceHealthSubscriber", builder)
        self.assertIn("configure_persistence_health_subscriber", builder)
        self.assertIn("SettingsPersistenceHealthSubscriber", projection)
        self.assertIn("SettingsPersistenceHealthStatus::PendingAdmission", projection)
        self.assertIn("BoundedKeyedIoTerminal::Failed", projection)
        self.assertIn("publish_toast", projection)
        self.assertNotIn("diagnostics()", projection)
        self.assertIn("Arc<EditorNotificationService>", context)
        for key in [
            "editor.notification.settings_persistence_failed.title",
            "editor.notification.settings_persistence_pending.message",
            "editor.notification.settings_persistence_failed.message",
        ]:
            self.assertIn(key, english)
            self.assertIn(key, chinese)

    def test_health_behaviour_regressions_cover_durable_and_deferred_states(self) -> None:
        tests = self.read("zircon_editor/src/core/settings/tests/mutation.rs")

        self.assertIn("coordinator_publishes_durable_health_from_worker_terminal", tests)
        self.assertIn("coordinator_projects_deferred_admission_as_retryable_health", tests)


if __name__ == "__main__":
    unittest.main()
