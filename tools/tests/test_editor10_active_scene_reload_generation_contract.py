import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
APP = ROOT / "zircon_editor/src/ui/retained_host/app.rs"
WORKSPACE = ROOT / "zircon_editor/src/ui/retained_host/app/assets/workspace.rs"
RELOAD_CONFLICT = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/assets/workspace/active_scene_reload_conflict.rs"
)
ASSET_RUNTIME_ACCESS = ROOT / "zircon_editor/src/ui/retained_host/app/asset_runtime_access.rs"
ASSET_REFRESH = ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh.rs"
ASSET_REFRESH_APPLY = ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh/apply.rs"
ASSET_REFRESH_EVENTS = ROOT / "zircon_editor/src/ui/retained_host/app/assets/refresh/events.rs"
BACKEND_REFRESH = ROOT / "zircon_editor/src/ui/retained_host/app/backend_refresh.rs"
CORE_SCENE_RELOAD = ROOT / "zircon_editor/src/core/document/scene_reload.rs"
CORE_SCENE_LOAD_JOB = ROOT / "zircon_editor/src/core/project/scene_load_job.rs"
HOST_SCENE_SUBMISSION = ROOT / "zircon_editor/src/ui/host/editor_scene_document_submission.rs"
HOST_TICK = ROOT / "zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs"
HOST_ASSEMBLY = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/state/construction/assembly.rs"
)
PROJECT_CLOSE = ROOT / "zircon_editor/src/ui/retained_host/app/project_close.rs"
RUNTIME_GENERATION = (
    ROOT
    / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs"
)
RUNTIME_LEVEL_LIFECYCLE = (
    ROOT / "zircon_runtime/src/scene/module/level_manager_lifecycle.rs"
)
AUTHORING_WORLD = ROOT / "zircon_editor/src/core/editing/authoring_world.rs"
PROJECT_CLOSE_SOURCE = ROOT / "zircon_editor/src/ui/retained_host/app/project_close.rs"
EDITOR_SAVE_BATCH = ROOT / "zircon_editor/src/ui/host/editor_save_batch.rs"
DOCUMENT_SAVE = ROOT / "zircon_editor/src/ui/retained_host/app/document_save.rs"
CLOSE_PROMPT_TESTS = (
    ROOT / "zircon_editor/src/ui/retained_host/app/tests/close_prompt.rs"
)


def reload_active_scene_source() -> str:
    source = WORKSPACE.read_text(encoding="utf-8")
    start = source.index("fn request_active_scene_reload")
    end = source.index("fn import_model_into_project", start)
    return source[start:end]


class ActiveSceneReloadGenerationContract(unittest.TestCase):
    def test_refresh_targets_the_lifecycle_active_scene_not_the_manifest_default(self) -> None:
        refresh = ASSET_REFRESH.read_text(encoding="utf-8")
        backend = BACKEND_REFRESH.read_text(encoding="utf-8")
        workspace = WORKSPACE.read_text(encoding="utf-8")

        self.assertIn("active_scene_identity", refresh)
        self.assertIn("active_scene_uri", backend)
        self.assertIn("reload_active_scene", backend)
        self.assertIn("active_scene_identity", workspace)
        self.assertNotIn("project.manifest().default_scene", workspace)
        self.assertNotIn("default_scene_uri", backend)
        self.assertNotIn("reload_default_scene", backend)

    def test_reload_commit_is_owned_by_the_document_lifecycle_coordinator(self) -> None:
        self.assertTrue(CORE_SCENE_RELOAD.is_file())
        coordinator = CORE_SCENE_RELOAD.read_text(encoding="utf-8")
        workspace = WORKSPACE.read_text(encoding="utf-8")
        host = HOST_SCENE_SUBMISSION.read_text(encoding="utf-8")

        self.assertIn("pub struct SceneDocumentReloadCoordinator", coordinator)
        self.assertIn("active_scene_identity_while_routed", coordinator)
        self.assertIn("prepare_active_scene_reload", coordinator)
        self.assertIn("install_active_scene_reload", coordinator)
        self.assertIn("commit_prepared_active_scene_reload", workspace)
        self.assertIn("SceneTransitionDirty", host)
        self.assertIn("commit_if_project_generation", host)
        self.assertNotIn("replace_world(", workspace)

    def test_reload_reuses_the_active_project_generation(self) -> None:
        source = reload_active_scene_source()
        host = HOST_SCENE_SUBMISSION.read_text(encoding="utf-8")

        self.assertIn("current_project_generation_snapshot()", source)
        self.assertIn("project.paths().root()", source)
        self.assertIn("active_scene_identity_for_session", source)
        self.assertIn("commit_if_project_generation", host)
        self.assertIn("ProjectGenerationCommitOutcome::Committed", host)
        self.assertIn("ProjectGenerationCommitOutcome::Superseded", host)
        self.assertIn("check_project_generation(&pending.generation)", source)
        self.assertNotIn("replace_world(", source)
        self.assertIn("reload_active_scene_world", host)

    def test_reload_identity_rejects_a_to_b_to_a_aba(self) -> None:
        lifecycle = (ROOT / "zircon_editor/src/core/document/lifecycle.rs").read_text(
            encoding="utf-8"
        )
        tests = (
            ROOT / "zircon_editor/src/core/document/scene_reload_tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("activation_revision: u64", lifecycle)
        self.assertIn("active_scene_activation_revision", lifecycle)
        self.assertIn("next_scene_activation_revision", lifecycle)
        self.assertIn("SceneActivationRevisionExhausted", lifecycle)
        self.assertGreaterEqual(tests.count('"res://scenes/main.scene.toml"'), 2)
        self.assertIn("SceneDocumentReloadOutcome::Superseded", tests)

    def test_reload_never_reopens_or_rescans_the_project(self) -> None:
        source = reload_active_scene_source()

        self.assertNotIn("current_project()", source)
        self.assertNotIn("ProjectManager::open", source)
        self.assertNotIn("scan_and_import", source)

    def test_runtime_generation_owner_exposes_a_typed_conditional_commit(self) -> None:
        runtime = RUNTIME_GENERATION.read_text(encoding="utf-8")
        access = ASSET_RUNTIME_ACCESS.read_text(encoding="utf-8")

        self.assertIn("pub struct ProjectAssetGenerationSnapshot", runtime)
        self.assertIn("pub struct ProjectAssetGenerationToken", runtime)
        self.assertIn("pub enum ProjectGenerationCommitOutcome", runtime)
        self.assertIn("pub enum ProjectGenerationMatch", runtime)
        self.assertIn("pub fn current_project_generation_snapshot", runtime)
        self.assertIn("pub fn check_project_generation", runtime)
        self.assertIn("pub fn commit_if_project_generation", runtime)
        self.assertIn("let generation = self.project_generation_read();", runtime)
        self.assertIn("commit()", runtime)
        self.assertIn("project_asset_manager: ManagerServiceHandle<ProjectAssetManager>", access)

    def test_same_project_supersession_is_coalesced_through_the_refresh_accumulator(self) -> None:
        workspace = reload_active_scene_source()
        refresh = ASSET_REFRESH.read_text(encoding="utf-8")
        apply = ASSET_REFRESH_APPLY.read_text(encoding="utf-8")
        events = ASSET_REFRESH_EVENTS.read_text(encoding="utf-8")

        self.assertIn("newer_same_project_generation: true", workspace)
        self.assertIn("newer_same_project_generation: false", workspace)
        self.assertIn(
            "newer_same_project_generation: true,\n"
            "            } => Ok(ActiveSceneReloadOutcome::Superseded)",
            workspace,
        )
        self.assertIn(
            "newer_same_project_generation: false,\n"
            "            } => Ok(ActiveSceneReloadOutcome::Discarded)",
            workspace,
        )
        self.assertIn("events.active_scene_reload_requested", refresh)
        self.assertIn("request_active_scene_reload", events)
        self.assertIn("active_scene_reload_requested: bool", events)
        self.assertIn("self.events.active_scene_reload_requested = true", events)
        self.assertIn("self.request_active_scene_reload()?", apply)
        self.assertIn(
            "matches!(&completion, Ok(ActiveSceneReloadOutcome::Superseded))",
            workspace,
        )
        self.assertIn("if reload_requested || superseded", workspace)
        self.assertIn("self.queue_active_scene_reload_retry();", workspace)
        self.assertEqual(
            workspace.count(".request_active_scene_reload(std::time::Instant::now())"),
            1,
        )

    def test_scene_io_and_deserialization_are_owned_by_an_interactive_job(self) -> None:
        self.assertTrue(CORE_SCENE_LOAD_JOB.is_file())
        job = CORE_SCENE_LOAD_JOB.read_text(encoding="utf-8")
        workspace = WORKSPACE.read_text(encoding="utf-8")

        self.assertIn("impl EditorJob for ProjectSceneLoadJob", job)
        self.assertIn("type Output = ProjectSceneDocument", job)
        self.assertIn("ProjectAuthority::default()", job)
        self.assertIn(".open_scene(&self.project, self.request)", job)
        self.assertIn("JobCategory::Index", job)
        self.assertIn("JobPriority::Interactive", job)
        self.assertGreaterEqual(job.count("context.check_cancelled()?"), 2)
        self.assertIn("submit_scene_open", workspace)
        self.assertNotIn("Scene::load_scene_from_uri", workspace)
        self.assertIn("document.into_world()", workspace)
        self.assertNotIn("document.world().clone()", workspace)

    def test_retained_host_coalesces_polls_and_cancels_the_scene_load_ticket(self) -> None:
        app = APP.read_text(encoding="utf-8")
        workspace = WORKSPACE.read_text(encoding="utf-8")
        tick = HOST_TICK.read_text(encoding="utf-8")
        assembly = HOST_ASSEMBLY.read_text(encoding="utf-8")
        project_close = PROJECT_CLOSE.read_text(encoding="utf-8")

        self.assertIn("pending_active_scene_reload: Option<assets::PendingActiveSceneReload>", app)
        self.assertIn("active_scene_reload_conflict: Option<assets::ActiveSceneReloadConflict>", app)
        self.assertIn("pending_active_scene_reload: None", assembly)
        self.assertIn("active_scene_reload_conflict: None", assembly)
        self.assertIn("self.poll_active_scene_reload();", tick)
        self.assertLess(
            tick.index("self.poll_active_scene_reload();"),
            tick.index("self.refresh_project_assets()"),
        )
        self.assertIn("reload_requested = true", workspace)
        self.assertIn("request_active_scene_reload", workspace)
        self.assertIn("commit_prepared_active_scene_reload", workspace)
        self.assertIn("cancel_pending_active_scene_reload", project_close)

    def test_retry_close_and_level_publication_preserve_reload_work(self) -> None:
        app = APP.read_text(encoding="utf-8")
        assembly = HOST_ASSEMBLY.read_text(encoding="utf-8")
        workspace_file = WORKSPACE.read_text(encoding="utf-8")
        workspace = reload_active_scene_source()
        project_close = PROJECT_CLOSE_SOURCE.read_text(encoding="utf-8")
        level_lifecycle = RUNTIME_LEVEL_LIFECYCLE.read_text(encoding="utf-8")
        authoring_world = AUTHORING_WORLD.read_text(encoding="utf-8")

        self.assertIn("JobSubmitError::AdmissionEntryLimitExceeded", workspace)
        self.assertIn("JobSubmitError::AdmissionByteLimitExceeded", workspace)
        self.assertIn("JobSubmitError::OldestPendingAgeExceeded", workspace)
        self.assertIn("ACTIVE_SCENE_RELOAD_ADMISSION_RETRY_LIMIT", workspace)
        self.assertIn("ActiveSceneReloadAdmissionState", workspace)
        self.assertIn("consecutive_failures", workspace)
        self.assertIn("active_scene_reload_retry_delay", workspace_file)
        self.assertIn("next_active_scene_reload_admission_retry", workspace_file)
        self.assertIn(
            "admission_retry_backs_off_three_times_then_terminates", workspace_file
        )
        self.assertIn("checked_add", workspace)
        self.assertIn("admission retry limit", workspace)
        self.assertIn("retry_not_before: Option<Instant>", workspace_file)
        self.assertIn("retry_not_before: None", workspace)
        refresh = ASSET_REFRESH.read_text(encoding="utf-8")
        self.assertIn("active_scene_reload_admission_retry_deadline()", refresh)
        self.assertIn("asset_maintenance_frame_update(", refresh)
        self.assertIn(
            "empty_refresh_preserves_active_scene_retry_deadline", refresh
        )
        self.assertIn("active_scene_reload_admission:", app)
        self.assertIn("active_scene_reload_admission: None", assembly)
        close_start = project_close.index("fn commit_project_close(")
        close = project_close[close_start:]
        self.assertLess(
            close.index("self.editor_manager.commit_project_close()"),
            close.index("self.cancel_pending_active_scene_reload()"),
        )
        self.assertIn("pub struct PreparedLevel", level_lifecycle)
        self.assertIn("pub struct PreparedLevelPublication", level_lifecycle)
        self.assertIn("if self.committed", level_lifecycle)
        self.assertIn("AuthoringWorldSeed::Prepared(level)", authoring_world)
        self.assertIn("let publication = level.publish();", authoring_world)
        self.assertIn("publication.commit();", authoring_world)

    def test_saved_conflict_retries_only_for_the_same_active_scene(self) -> None:
        workspace = reload_active_scene_source() + RELOAD_CONFLICT.read_text(encoding="utf-8")

        self.assertIn("fn reconcile_active_scene_reload_conflict", workspace)
        self.assertIn("active_scene_identity_for_session()", workspace)
        self.assertIn("!= Some(&conflict.identity)", workspace)
        self.assertIn("self.dirty_project_scene_generation()", workspace)
        self.assertIn("Ok(None)", workspace)
        self.assertIn("self.active_scene_reload_conflict.take()", workspace)
        self.assertIn("dismiss_active_scene_reload_conflict_decision", workspace)
        self.assertIn("self.queue_active_scene_reload_retry();", workspace)

    def test_dirty_reload_conflict_has_an_explicit_generation_bound_decision_flow(self) -> None:
        app = APP.read_text(encoding="utf-8")
        assembly = HOST_ASSEMBLY.read_text(encoding="utf-8")
        workspace = WORKSPACE.read_text(encoding="utf-8")
        conflict = RELOAD_CONFLICT.read_text(encoding="utf-8")
        host = HOST_SCENE_SUBMISSION.read_text(encoding="utf-8")
        english = (ROOT / "zircon_editor/assets/i18n/en.toml").read_text(
            encoding="utf-8"
        )
        chinese = (ROOT / "zircon_editor/assets/i18n/zh-CN.toml").read_text(
            encoding="utf-8"
        )

        self.assertIn("generation: ProjectAssetGenerationToken", conflict)
        self.assertIn("enum ActiveSceneReloadConflictState", conflict)
        self.assertIn("AwaitingDecision", conflict)
        self.assertIn("DiscardRequested", conflict)
        self.assertIn("Cancelled", conflict)
        self.assertIn("DecisionNotification::new", conflict)
        self.assertIn("DecisionOption::new", conflict)
        self.assertIn('DecisionOptionId::parse("save")', conflict)
        self.assertIn('DecisionOptionId::parse("discard")', conflict)
        self.assertIn('DecisionOptionId::parse("keep_editing")', conflict)
        self.assertIn(".with_default_option(save_option)", conflict)
        self.assertIn(".with_cancel_option(keep_editing_option)", conflict)
        self.assertIn(".with_display_subject", conflict)
        self.assertIn("MAX_DECISION_DISPLAY_SUBJECT_BYTES", conflict)
        self.assertIn("active_scene_reload_display_subject", conflict)
        self.assertIn("is_char_boundary", conflict)
        self.assertIn("notifications().decisions()", conflict)
        self.assertIn(".snapshot()", conflict)
        self.assertIn("ActiveSceneReloadDecisionLookup::Missing", conflict)

        self.assertIn("active_scene_reload_decision_sequence: u64", app)
        self.assertIn("active_scene_reload_decision_sequence: 0", assembly)
        self.assertIn("self.save_project_scene()", conflict)
        self.assertIn("self.dirty_project_scene_generation()", conflict)
        self.assertNotIn("begin_dirty_document_save", conflict)
        self.assertNotIn("poll_dirty_document_save", conflict)
        self.assertNotIn("PendingActiveSceneReloadConflictSave", conflict)

        self.assertIn("enum PreparedActiveSceneReloadDirtyPolicy", host)
        self.assertIn("\n    Reject,", host)
        self.assertIn("PreparedActiveSceneReloadDirtyPolicy::Discard", host)
        self.assertIn("dirty_policy", workspace)
        self.assertIn("pending.dirty_policy", workspace)
        self.assertIn("if self.dirty_policy == PreparedActiveSceneReloadDirtyPolicy::Discard", host)

        for catalog in (english, chinese):
            self.assertIn("editor.scene.reload_conflict.title", catalog)
            self.assertIn("editor.scene.reload_conflict.message", catalog)
            self.assertIn("editor.scene.reload_conflict.save", catalog)
            self.assertIn("editor.scene.reload_conflict.discard", catalog)
            self.assertIn("editor.scene.reload_conflict.keep_editing", catalog)

    def test_dirty_save_competition_has_one_explicit_owner_and_deferred_callers(self) -> None:
        coordinator = EDITOR_SAVE_BATCH.read_text(encoding="utf-8")
        conflict = RELOAD_CONFLICT.read_text(encoding="utf-8")
        save_all = DOCUMENT_SAVE.read_text(encoding="utf-8")
        project_close = PROJECT_CLOSE_SOURCE.read_text(encoding="utf-8")
        app = APP.read_text(encoding="utf-8")
        assembly = HOST_ASSEMBLY.read_text(encoding="utf-8")
        behavior = CLOSE_PROMPT_TESTS.read_text(encoding="utf-8")

        self.assertIn("enum DirtyDocumentSaveOwner", coordinator)
        self.assertIn("SaveAll", coordinator)
        self.assertIn("ClosePrompt", coordinator)
        self.assertIn("Busy { owner: DirtyDocumentSaveOwner }", coordinator)
        self.assertIn("OwnerMismatch", coordinator)
        self.assertIn("dirty_document_save_owner", coordinator)
        self.assertIn("wrong_owner_poll_cannot_consume_a_real_completed_save_batch", coordinator)
        self.assertIn("self.save_project_scene()", conflict)
        self.assertNotIn("DirtyDocumentSaveOwner", conflict)
        self.assertIn("queued_document_save_all", save_all)
        self.assertIn("DirtyDocumentSaveOwner::SaveAll", save_all)
        self.assertIn("DirtyDocumentSaveStart::Busy", save_all)
        self.assertIn("queued_document_save_all: bool", app)
        self.assertIn("queued_document_save_all: false", assembly)
        self.assertIn("RetainedProjectCloseError::PendingDocumentSave", project_close)
        close_start = project_close.index("fn commit_project_close(")
        manager_close = project_close.index(
            "self.editor_manager.commit_project_close()", close_start
        )
        owner_guard = project_close.index("dirty_document_save_owner()", close_start)
        self.assertLess(owner_guard, manager_close)
        self.assertIn(
            "save_all_queues_behind_close_prompt_save_then_acquires_the_released_owner",
            behavior,
        )
        self.assertIn(
            "project_close_cannot_teardown_until_close_prompt_save_terminalizes",
            behavior,
        )


if __name__ == "__main__":
    unittest.main()
