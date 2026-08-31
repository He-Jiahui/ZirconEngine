import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class Editor17SettingsMutationCoordinatorContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_context_owns_the_only_product_mutation_coordinator(self) -> None:
        context = self.read("zircon_editor/src/core/context/editor_context.rs")
        builder = self.read("zircon_editor/src/core/context/builder.rs")
        manager = self.read("zircon_editor/src/ui/host/editor_manager.rs")
        settings = self.read("zircon_editor/src/core/settings/mod.rs")

        self.assertIn("SettingsMutationCoordinator", context)
        self.assertIn("settings_mutations: Arc<SettingsMutationCoordinator>", context)
        self.assertIn("pub fn settings_mutations(", context)
        self.assertNotIn("settings_persistence: SettingsPersistenceService", context)
        self.assertIn("SettingsMutationCoordinator::new", builder)
        self.assertIn(".settings_mutations()", manager)
        self.assertNotIn(".settings()\n            .record_command_palette_usage", manager)
        self.assertIn("mod mutation;", settings)

    def test_viewport_only_submits_typed_mutations_to_context_owner(self) -> None:
        controller = self.read(
            "zircon_editor/src/scene/viewport/controller/scene_viewport_controller.rs"
        )
        accessors = self.read(
            "zircon_editor/src/scene/viewport/controller/"
            "scene_viewport_controller_accessors.rs"
        )
        construction = self.read(
            "zircon_editor/src/scene/viewport/controller/"
            "scene_viewport_controller_construction.rs"
        )

        for source in (controller, accessors, construction):
            self.assertNotIn("SettingsStore", source)
            self.assertNotIn("SettingsPersistenceService", source)
            self.assertNotIn("SettingsPersistenceTicket", source)
            self.assertNotIn("settings_persistence_tickets", source)
        self.assertIn("SettingsMutationCoordinator", controller)
        self.assertIn("settings_mutations.set(", accessors)

    def test_project_lifecycle_binds_the_context_coordinator_not_the_viewport(self) -> None:
        construction = self.read(
            "zircon_editor/src/ui/workbench/startup/editor_state_construction.rs"
        )
        project = self.read(
            "zircon_editor/src/ui/workbench/startup/editor_state_project.rs"
        )

        self.assertIn("context.settings_mutations()", construction)
        self.assertIn(".settings_mutations()", project)
        self.assertIn(".bind_project", project)
        self.assertIn("settings_mutations().clear_project", project)
        self.assertNotIn("configure_project_settings", construction + project)
        self.assertNotIn("clear_project_settings", project)

    def test_persistence_coalesces_by_physical_document_not_setting_key(self) -> None:
        persistence = self.read("zircon_editor/src/core/settings/persistence.rs")
        target = persistence.split("fn persistence_target", 1)[1].split(
            "fn scope_name", 1
        )[0]

        self.assertIn('"settings:{}:{}"', target)
        self.assertNotIn("key.as_str()", target)
        self.assertIn("let lane_key = Arc::clone(&request.target);", persistence)

    def test_mutation_receipt_preserves_apply_and_durability_state(self) -> None:
        mutation = self.read("zircon_editor/src/core/settings/mutation.rs")

        for disposition in (
            "Unchanged",
            "SessionApplied",
            "PersistentQueued",
            "AppliedPendingAdmission",
        ):
            self.assertIn(disposition, mutation)
        self.assertIn("ProjectNotBound", mutation)
        self.assertIn("UserSourceUnavailable", mutation)
        self.assertIn("OperationInProgress", mutation)
        self.assertIn("retry_pending", mutation)
        self.assertIn("pub fn clear(", mutation)
        self.assertIn("record_command_palette_usage", mutation)
        self.assertIn("RetryRejected", mutation)
        self.assertIn("admission_error", mutation)
        self.assertIn("MAX_PENDING_SETTINGS_DOCUMENTS", mutation)
        self.assertNotIn('expect("persistent mutations resolve', mutation)
        self.assertNotIn("VecDeque<SettingsPersistenceTicket>", mutation)


if __name__ == "__main__":
    unittest.main()
