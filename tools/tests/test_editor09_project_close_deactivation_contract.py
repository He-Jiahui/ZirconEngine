import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]


class Editor09ProjectCloseDeactivationContractTests(unittest.TestCase):
    def test_project_deactivation_is_an_infallible_local_retirement(self) -> None:
        api = (
            REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/api.rs"
        ).read_text(encoding="utf-8")
        bridge = (
            REPO_ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/preview_trait_bridge.rs"
        ).read_text(encoding="utf-8")
        project_access = (
            REPO_ROOT / "zircon_editor/src/ui/host/project_access.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("fn deactivate_runtime_project(&self) -> bool;", api)
        self.assertNotIn(
            "fn deactivate_runtime_project(&self) -> Result<bool, CoreError>", api
        )
        self.assertIn("fn deactivate_runtime_project(&self) -> bool", bridge)
        self.assertIn("Deactivate: FnOnce() -> bool", project_access)
        self.assertNotIn("DeactivateError", project_access)
        self.assertNotIn(
            '"deactivate_editor_asset_projection",\n        deactivate_projection()',
            project_access,
        )

    def test_catalog_and_source_sync_identities_never_saturate_or_wrap(self) -> None:
        generation = (
            REPO_ROOT / "zircon_editor/src/ui/host/editor_asset_manager/generation.rs"
        ).read_text(encoding="utf-8")
        state = (
            REPO_ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/editor_asset_state.rs"
        ).read_text(encoding="utf-8")
        deactivation = (
            REPO_ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/project_deactivation.rs"
        ).read_text(encoding="utf-8")
        sync = (
            REPO_ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs"
        ).read_text(encoding="utf-8")
        preview = (
            REPO_ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/request_preview_refresh.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("fn next_catalog_identity(&self)", generation)
        self.assertIn("fn next_publish_epoch(&self)", generation)
        self.assertIn("fn advance_source_sync_epoch(", state)
        self.assertIn("next_catalog_identity()", deactivation)
        self.assertIn("advance_source_sync_epoch()", deactivation)
        self.assertIn("next_catalog_identity()", sync)
        self.assertIn("advance_source_sync_epoch()", sync)
        self.assertIn("next_publish_epoch()", preview)
        for source in (deactivation, sync, preview):
            self.assertNotIn("saturating_add(1)", source)
            self.assertNotIn("source_sync_epoch.fetch_add", source)

    def test_runtime_project_sync_commits_through_the_runtime_generation_fence(self) -> None:
        manager = (
            REPO_ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/mod.rs"
        ).read_text(encoding="utf-8")
        sync = (
            REPO_ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("current_project_generation_snapshot()", manager)
        self.assertNotIn("current_project_manager()", manager)
        self.assertIn("sync_from_runtime_project_generation", manager)
        self.assertIn("ProjectAssetGenerationToken", sync)
        self.assertIn("commit_if_project_generation", sync)
        self.assertIn("runtime_project_generation_superseded_count", sync)
        self.assertNotIn("pub fn sync_from_project", sync)
        terminal = sync.split("enum CatalogCommitAttempt", maxsplit=1)[1]
        self.assertLess(
            terminal.index("build_catalog_generation("),
            terminal.index("let commit = ||"),
        )
        self.assertIn("drop(source_commit_guard);", terminal)

    def test_runtime_catalog_generation_sequence_never_wraps_into_an_old_token(self) -> None:
        runtime_generation = (
            REPO_ROOT
            / "zircon_runtime/src/asset/project/catalog_input_generation.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("fn advance_sequence(sequence: &AtomicU64)", runtime_generation)
        self.assertIn(".fetch_update(", runtime_generation)
        self.assertIn("current.checked_add(1)", runtime_generation)
        self.assertNotIn(
            "NEXT_CATALOG_INPUT_GENERATION.fetch_add", runtime_generation
        )


if __name__ == "__main__":
    unittest.main()
