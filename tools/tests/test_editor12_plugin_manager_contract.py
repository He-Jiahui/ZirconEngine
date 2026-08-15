"""Static owner-boundary checks for the Editor12 plugin manager."""

from pathlib import Path
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_ROOT = REPOSITORY_ROOT / "zircon_editor" / "src" / "core" / "plugin"
PLUGIN_MODULE_DOC = REPOSITORY_ROOT / "docs" / "zircon_editor" / "core" / "plugin.md"
LEGACY_PLUGIN_MODULE_DOC = (
    REPOSITORY_ROOT / "docs" / "zircon_editor" / "core" / "editor_plugin.md"
)
MANAGER_MODULE_ROOT = PLUGIN_ROOT / "manager"
MANAGER_DISCOVERY = MANAGER_MODULE_ROOT / "discovery.rs"
MANAGER_REPLACEMENT_OWNER = MANAGER_MODULE_ROOT / "lifecycle_replacement.rs"
MANAGER_PROJECT_SELECTION = MANAGER_MODULE_ROOT / "project_selection.rs"
MANAGER_PROJECT_REGISTRATION = MANAGER_MODULE_ROOT / "project_registration.rs"
MANAGER_PUBLICATION = MANAGER_MODULE_ROOT / "publication.rs"
MANAGER_STATE = MANAGER_MODULE_ROOT / "state.rs"
MANAGER_SNAPSHOT = MANAGER_MODULE_ROOT / "snapshot.rs"
MANAGER_TESTS = MANAGER_MODULE_ROOT / "tests.rs"
MANAGER_LIFECYCLE_REPLACEMENT = (
    MANAGER_MODULE_ROOT / "tests" / "lifecycle_replacement.rs"
)
MANAGER_LIFECYCLE_STATE = MANAGER_MODULE_ROOT / "tests" / "lifecycle_state.rs"
MANAGER_PROJECT_SELECTION_TEST = (
    MANAGER_MODULE_ROOT / "tests" / "project_selection.rs"
)
NATIVE_REGISTRATION_MANAGER = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_plugins_export"
    / "native_registration"
    / "manager.rs"
)
NATIVE_ENABLEMENT = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_plugins_export"
    / "enablement"
    / "native.rs"
)
FEATURE_ENABLEMENT = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_plugins_export"
    / "enablement"
    / "features.rs"
)
SELECTION_POLICY = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_plugins_export"
    / "enablement"
    / "selection_policy.rs"
)
NATIVE_REGISTRATION_PROJECTION = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_plugins_export"
    / "native_registration"
    / "registration_projection.rs"
)
NATIVE_STATUS = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_plugins_export"
    / "status"
    / "native.rs"
)
PROJECT_STATUS_SNAPSHOT = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_plugins_export"
    / "status"
    / "project_snapshot.rs"
)
RETAINED_PLUGIN_PANE_DATA = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "app"
    / "module_plugin_projection"
    / "pane_data.rs"
)
RETAINED_PLUGIN_STATUS_REPORT = (
    RETAINED_PLUGIN_PANE_DATA.parent / "pane_data" / "report.rs"
)
RETAINED_PLUGIN_STATUS_ROWS = (
    RETAINED_PLUGIN_PANE_DATA.parent / "pane_data" / "view_rows.rs"
)
RETAINED_HOST_APP = (
    REPOSITORY_ROOT / "zircon_editor" / "src" / "ui" / "retained_host" / "app.rs"
)
RETAINED_HOST_STARTUP_ASSEMBLY = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "app"
    / "host_lifecycle"
    / "startup"
    / "state"
    / "construction"
    / "assembly.rs"
)
RETAINED_PLUGIN_PROJECTION = RETAINED_PLUGIN_PANE_DATA.parent.with_suffix(".rs")
RETAINED_PLUGIN_PROJECTION_CACHE = RETAINED_PLUGIN_PANE_DATA.parent / "cache.rs"
RETAINED_PLUGIN_ROWS = RETAINED_PLUGIN_PANE_DATA.parent / "rows.rs"
RETAINED_PLUGIN_FALLBACK_MANIFEST = RETAINED_PLUGIN_ROWS.parent / "rows" / "manifest.rs"
RUNTIME_REGISTRATION_TEST = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "tests"
    / "editor_event"
    / "runtime"
    / "extensions_registration.rs"
)
EDITOR_MANAGER = (
    REPOSITORY_ROOT / "zircon_editor" / "src" / "ui" / "host" / "editor_manager.rs"
)
EDITOR_MANAGER_PROJECT = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_project.rs"
)
EDITOR_MANAGER_STARTUP = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_startup.rs"
)
EDITOR_MANAGER_PLUGIN_EXPORTS = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_plugins_export"
    / "mod.rs"
)
PROJECT_ENABLEMENT = (
    REPOSITORY_ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_plugins_export"
    / "enablement"
    / "project.rs"
)
COMMANDLET_RUNNER = (
    REPOSITORY_ROOT / "zircon_editor" / "src" / "core" / "commandlet" / "runner.rs"
)
EDITOR_EXTENSION = (
    REPOSITORY_ROOT / "zircon_editor" / "src" / "core" / "editor_extension.rs"
)
TEMPLATE_CONTRIBUTIONS = (
    EDITOR_EXTENSION.parent / "editor_extension" / "template_contributions.rs"
)


class Editor12PluginManagerContractTests(unittest.TestCase):
    def test_plugin_module_documentation_hard_cuts_the_legacy_root_path(self) -> None:
        self.assertTrue(PLUGIN_MODULE_DOC.is_file())
        self.assertFalse(LEGACY_PLUGIN_MODULE_DOC.exists())

        document = PLUGIN_MODULE_DOC.read_text(encoding="utf-8")
        self.assertIn("EditorPluginManager", document)
        self.assertIn("dispatch_lifecycle_event_to_active", document)
        self.assertNotIn("zircon_editor/src/core/editor_plugin.rs", document)
        self.assertNotIn("zircon_editor/src/core/editor_plugin_catalog_gen.rs", document)

    def test_manager_separates_discovery_state_snapshot_and_tests_from_orchestration(self) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        state = MANAGER_STATE.read_text(encoding="utf-8")

        for leaf in (
            "discovery.rs",
            "publication.rs",
            "project_registration.rs",
            "state.rs",
            "snapshot.rs",
            "tests.rs",
        ):
            self.assertTrue(
                (MANAGER_MODULE_ROOT / leaf).is_file(),
                f"plugin manager must own the {leaf} responsibility in a leaf module",
            )
        self.assertIn("mod discovery;", manager)
        self.assertIn("mod lifecycle_replacement;", manager)
        self.assertIn("mod project_registration;", manager)
        self.assertIn("mod project_selection;", manager)
        self.assertIn("mod publication;", manager)
        self.assertIn("mod state;", manager)
        self.assertIn("mod snapshot;", manager)
        self.assertIn("#[cfg(test)]\nmod tests;", manager)
        self.assertLessEqual(len(manager.splitlines()), 900)
        self.assertTrue(MANAGER_REPLACEMENT_OWNER.is_file())

        tests = MANAGER_TESTS.read_text(encoding="utf-8")
        snapshot_publication = (
            MANAGER_MODULE_ROOT / "tests" / "snapshot_publication.rs"
        )
        self.assertTrue(snapshot_publication.is_file())
        self.assertTrue(MANAGER_LIFECYCLE_REPLACEMENT.is_file())
        self.assertTrue(MANAGER_LIFECYCLE_STATE.is_file())
        self.assertTrue(MANAGER_PROJECT_SELECTION_TEST.is_file())
        self.assertIn("mod snapshot_publication;", tests)
        self.assertIn("mod lifecycle_replacement;", tests)
        self.assertIn("mod lifecycle_state;", tests)
        self.assertIn("mod project_selection;", tests)
        self.assertIn("mod project_registration;", tests)
        self.assertLessEqual(len(tests.splitlines()), 800)

    def test_project_manifest_enablement_is_one_manager_owned_publication(self) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        project_selection = MANAGER_PROJECT_SELECTION.read_text(encoding="utf-8")
        state = MANAGER_STATE.read_text(encoding="utf-8")
        tests = MANAGER_PROJECT_SELECTION_TEST.read_text(encoding="utf-8")
        document = PLUGIN_MODULE_DOC.read_text(encoding="utf-8")

        self.assertTrue(MANAGER_PROJECT_SELECTION.is_file())
        self.assertIn("pub fn apply_project_manifest", manager)
        self.assertIn("project_selection::apply_project_manifest", manager)
        self.assertIn("pub(super) fn apply_project_manifest", project_selection)
        self.assertIn("fn editor_package_enablement", project_selection)
        self.assertIn("RuntimeTargetMode::EditorHost", project_selection)
        self.assertIn("validate_enablement_request", project_selection)
        self.assertIn("manager.publish_manager_snapshot", project_selection)
        self.assertIn("DuplicateProjectSelection", state)
        self.assertIn("## Project Selection", document)
        self.assertIn("EditorPluginManager::apply_project_manifest", document)
        self.assertIn(
            "project_manifest_applies_editor_enablement_in_one_snapshot_generation", tests
        )
        self.assertIn(
            "project_manifest_rejects_duplicate_editor_selection_without_mutating", tests
        )
        self.assertIn(
            "invalid_project_manifest_does_not_dispatch_partial_lifecycle_callbacks", tests
        )

    def test_project_open_applies_completed_plugin_manifest_before_document_events(
        self,
    ) -> None:
        project = EDITOR_MANAGER_PROJECT.read_text(encoding="utf-8")
        startup = EDITOR_MANAGER_STARTUP.read_text(encoding="utf-8")
        plugin_exports = EDITOR_MANAGER_PLUGIN_EXPORTS.read_text(encoding="utf-8")
        open_project = project.split("pub fn open_project", 1)[1].split(
            "pub fn close_project", 1
        )[0]

        self.assertIn("fn apply_project_plugin_manifest", project)
        self.assertIn("project_root: &Path", project)
        self.assertIn("NativePluginLoader.load_discovered_editor", project)
        self.assertIn(
            "complete_project_plugin_manifest_with_native_report(manifest, &native_report)",
            project,
        )
        self.assertIn(
            "selected_native_editor_plugin_registration_reports_from_load_report",
            project,
        )
        self.assertIn("publish_project_registration_reports(native_reports)", project)
        self.assertIn("clear_project_registration_reports", project)
        self.assertIn(".apply_project_manifest(&completed.plugins)", project)
        self.assertIn("self.publish_project_plugin_status(", project)
        self.assertIn("fn apply_project_plugin_manifest_or_close", project)
        self.assertIn("self.host.close_project()", project)
        self.assertLess(
            open_project.index("apply_project_plugin_manifest_or_close"),
            open_project.index("publish_document_messages"),
        )
        self.assertEqual(
            startup.count("self.publish_document_startup_session(&session)?;"), 3
        )
        self.assertIn("pub fn plugin_panel_source", plugin_exports)
        self.assertIn("EditorPluginPanelSource::from_manager(&self.plugin_manager)", plugin_exports)

    def test_project_native_discovery_uses_the_manager_scoped_publication_boundary(
        self,
    ) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        publication = MANAGER_PUBLICATION.read_text(encoding="utf-8")
        project_registration = MANAGER_PROJECT_REGISTRATION.read_text(encoding="utf-8")
        catalog = (PLUGIN_ROOT / "catalog.rs").read_text(encoding="utf-8")
        registration = (PLUGIN_ROOT / "registration.rs").read_text(encoding="utf-8")
        editor_manager = EDITOR_MANAGER.read_text(encoding="utf-8")
        tests = (
            MANAGER_MODULE_ROOT / "tests" / "project_registration.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("mod publication;", manager)
        self.assertIn("mod project_registration;", manager)
        self.assertIn("publish_catalog_with_indexed_discoveries", publication)
        self.assertIn("lifecycle_mutation", publication)
        self.assertIn("publish_project_registration_reports", project_registration)
        self.assertIn("clear_project_registration_reports", project_registration)
        self.assertIn("EditorPluginSource::Project", project_registration)
        self.assertIn("replace_project_registration_reports", catalog)
        self.assertIn("record_host_lifecycle_event", catalog)
        self.assertIn("record_host_lifecycle_event", registration)
        self.assertIn("EditorPluginManager::builtin(", editor_manager)
        self.assertNotIn("plugin_manager: &'static EditorPluginManager", editor_manager)
        self.assertIn(
            "project_native_reports_replace_only_project_rows_and_record_host_lifecycle",
            tests,
        )

    def test_project_plugin_status_is_published_once_and_retained_reads_are_snapshot_only(
        self,
    ) -> None:
        native_status = NATIVE_STATUS.read_text(encoding="utf-8")
        project_status_snapshot = PROJECT_STATUS_SNAPSHOT.read_text(encoding="utf-8")
        editor_manager = EDITOR_MANAGER.read_text(encoding="utf-8")
        project_manager = EDITOR_MANAGER_PROJECT.read_text(encoding="utf-8")
        retained_pane_data = RETAINED_PLUGIN_PANE_DATA.read_text(encoding="utf-8")
        retained_report = RETAINED_PLUGIN_STATUS_REPORT.read_text(encoding="utf-8")
        retained_rows = RETAINED_PLUGIN_STATUS_ROWS.read_text(encoding="utf-8")
        retained_host = RETAINED_HOST_APP.read_text(encoding="utf-8")
        retained_startup = RETAINED_HOST_STARTUP_ASSEMBLY.read_text(encoding="utf-8")
        retained_projection = RETAINED_PLUGIN_PROJECTION.read_text(encoding="utf-8")
        retained_cache = RETAINED_PLUGIN_PROJECTION_CACHE.read_text(encoding="utf-8")
        retained_rows_owner = RETAINED_PLUGIN_ROWS.read_text(encoding="utf-8")
        native_registration = NATIVE_REGISTRATION_MANAGER.read_text(encoding="utf-8")
        native_enablement = NATIVE_ENABLEMENT.read_text(encoding="utf-8")
        capabilities = (
            REPOSITORY_ROOT
            / "zircon_editor"
            / "src"
            / "ui"
            / "host"
            / "editor_manager_plugins_export"
            / "enablement"
            / "capabilities.rs"
        ).read_text(encoding="utf-8")
        feature_enablement = FEATURE_ENABLEMENT.read_text(encoding="utf-8")
        selection_policy = SELECTION_POLICY.read_text(encoding="utf-8")
        project_enablement = PROJECT_ENABLEMENT.read_text(encoding="utf-8")

        self.assertNotIn("load_discovered_all", native_status)
        self.assertNotIn("NativePluginLoader.discover", native_status)
        self.assertIn("native_plugin_status_report_from_load_report", native_status)
        self.assertIn("Arc<EditorPluginStatusReport>", project_status_snapshot)
        self.assertIn("Mutex<Option<Arc<ProjectPluginStatusSnapshot>>>", editor_manager)
        self.assertIn(
            "builtin_plugin_status: Mutex<Arc<ProjectPluginStatusSnapshot>>",
            editor_manager,
        )
        self.assertIn("pub(crate) fn published_plugin_status_report", editor_manager)
        self.assertIn("refresh_builtin_plugin_status", editor_manager)
        self.assertIn("publish_project_plugin_status", project_manager)
        self.assertIn("clear_project_plugin_status", project_manager)
        self.assertIn("load_discovered_editor", project_manager)
        self.assertIn(
            "selected_native_editor_plugin_registration_reports_from_load_report",
            native_registration,
        )
        self.assertIn("published_plugin_status_report", retained_report)
        self.assertNotIn("project_plugin_status_report", retained_report)
        self.assertNotIn("fallback_project_manifest", retained_report)
        self.assertNotIn("ProjectManifest::load", retained_report)
        self.assertNotIn("project_root_path", retained_report)
        self.assertNotIn("mod manifest;", retained_rows_owner)
        self.assertNotIn("fallback_project_manifest", retained_rows_owner)
        self.assertFalse(RETAINED_PLUGIN_FALLBACK_MANIFEST.exists())
        self.assertIn("Arc<EditorPluginStatusReport>", retained_report)
        self.assertIn("module_plugin_status_rows(status_report)", retained_pane_data)
        self.assertIn("report: &EditorPluginStatusReport", retained_rows)
        self.assertIn("module_plugin_projection_cache", retained_host)
        self.assertIn("ModulePluginPaneProjectionCache", retained_host)
        self.assertIn("module_plugin_projection_cache: Default::default()", retained_startup)
        self.assertIn("mod cache;", retained_projection)
        self.assertIn("Arc::ptr_eq", retained_cache)
        self.assertIn("pub(in crate::ui::retained_host::app) fn get_or_build", retained_cache)
        self.assertIn("FnOnce(&EditorPluginStatusReport)", retained_cache)
        self.assertIn(
            "one_thousand_stable_projection_reads_reuse_one_generation_at_1_100_1000_plugin_scales",
            retained_cache,
        )
        self.assertIn("stable_projection_clone_bytes", retained_cache)
        self.assertIn("EDITOR12_PLUGIN_PANE_STABLE_READ", retained_cache)
        self.assertIn(".get_or_build(&status_report", retained_pane_data)
        self.assertNotIn(".cached(&status_report)", retained_pane_data)
        self.assertIn(
            "publish_project_plugin_status_from_load_report", native_enablement
        )
        self.assertIn(
            "publish_project_plugin_status_from_load_report", feature_enablement
        )
        self.assertIn(
            "publish_project_plugin_status_from_load_report", selection_policy
        )
        self.assertIn(
            "native_aware_runtime_plugin_catalog_from_load_report", feature_enablement
        )
        self.assertIn("manifest.plugins.set_enabled(selection.clone());", project_enablement)
        self.assertIn("self.publish_project_plugin_status", project_enablement)
        self.assertIn("self.plugin_status_report(manifest)", project_enablement)
        self.assertIn(
            "fn set_project_plugin_enabled_unpublished", project_enablement
        )
        self.assertIn(
            "pub(in crate::ui::host) fn update_editor_plugin_state_unpublished",
            editor_manager,
        )
        self.assertIn(
            "pub(in crate::ui::host) fn set_editor_plugin_enabled_unpublished",
            capabilities,
        )
        project_unpublished_enablement = project_enablement.split(
            "fn set_project_plugin_enabled_unpublished", 1
        )[1]
        self.assertIn(
            "self.update_editor_plugin_state_unpublished(plugin_id, enabled)?;",
            project_unpublished_enablement,
        )
        self.assertIn(
            "self.set_editor_plugin_enabled_unpublished(plugin_id, enabled)?",
            project_unpublished_enablement,
        )
        self.assertNotIn(
            "self.update_editor_plugin_state(plugin_id, enabled)?;",
            project_unpublished_enablement,
        )
        self.assertNotIn(
            "self.set_editor_plugin_enabled(plugin_id, enabled)?",
            project_unpublished_enablement,
        )
        native_builtin_enablement = native_enablement.split(
            "if self\n            .runtime_plugin_catalog()",
            1,
        )[1].split("let native_projection", 1)[0]
        self.assertIn(
            "set_project_plugin_enabled_unpublished", native_builtin_enablement
        )
        self.assertNotIn(
            "set_project_plugin_enabled(manifest, plugin_id, enabled)",
            native_builtin_enablement,
        )
        self.assertLess(
            native_builtin_enablement.index("set_project_plugin_enabled_unpublished"),
            native_builtin_enablement.index(
                "publish_project_plugin_status_from_load_report"
            ),
        )
        native_external_enablement = native_enablement.split(
            "let native_projection", 1
        )[1].split("let diagnostics", 1)[0]
        self.assertLess(
            native_external_enablement.index("self.set_editor_capabilities_enabled"),
            native_external_enablement.index(
                "manifest.plugins.set_enabled(selection.clone());"
            ),
        )

    def test_lifecycle_regression_leaves_cover_managed_transitions_and_new_instances(
        self,
    ) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        state_test = MANAGER_LIFECYCLE_STATE.read_text(encoding="utf-8")
        replacement_test = MANAGER_LIFECYCLE_REPLACEMENT.read_text(encoding="utf-8")
        replacement_owner = MANAGER_REPLACEMENT_OWNER.read_text(encoding="utf-8")
        native_projection = NATIVE_REGISTRATION_PROJECTION.read_text(encoding="utf-8")
        runtime_registration = RUNTIME_REGISTRATION_TEST.read_text(encoding="utf-8")

        self.assertIn("pub fn transition_state", manager)
        self.assertIn("pub fn validate_enablement", manager)
        self.assertIn("ManagedLifecycleTransitionRequired", state_test)
        self.assertIn("state_machine_accepts_only_lifecycle_edges", state_test)
        self.assertIn(
            "replacing_an_active_package_retires_the_old_instance_before_hot_reload",
            replacement_test,
        )
        self.assertIn("replacing_a_faulted_package_dispatches_lifecycle", replacement_test)
        self.assertIn("replacing_an_active_package_retires_the_old_instance_before_hot_reload", replacement_test)
        self.assertIn(
            "replacement_retries_failed_unload_before_activating_candidate",
            replacement_test,
        )
        self.assertIn(
            "the next replacement must retry the unfinished old-instance unload first",
            replacement_test,
        )
        self.assertIn("EditorPluginLifecycleStage::Loaded", replacement_test)
        self.assertIn("EditorPluginLifecycleStage::Enabled", replacement_test)
        self.assertIn("EditorPluginLifecycleStage::Disabled", replacement_test)
        self.assertIn("EditorPluginLifecycleStage::Unloaded", replacement_test)
        self.assertIn("EditorPluginLifecycleStage::HotReloaded", replacement_test)
        self.assertIn("retire_replaced_active_entries", replacement_owner)
        self.assertIn("replaced_live_package_ids", replacement_owner)
        self.assertIn("instance_requires_retirement", replacement_owner)
        self.assertIn("lifecycle_stage_succeeded", replacement_owner)
        self.assertIn("LifecycleCleanupFailed", MANAGER_DISCOVERY.read_text(encoding="utf-8"))
        self.assertEqual(native_projection.count("successful_lifecycle_stages:"), 1)
        self.assertEqual(native_projection.count("failed_lifecycle_stages:"), 1)
        self.assertEqual(runtime_registration.count("successful_lifecycle_stages:"), 2)
        self.assertEqual(runtime_registration.count("failed_lifecycle_stages:"), 2)

    def test_loading_phase_has_its_own_core_module(self) -> None:
        plugin_module = (PLUGIN_ROOT / "mod.rs").read_text(encoding="utf-8")
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        phases = (PLUGIN_ROOT / "phases.rs").read_text(encoding="utf-8")

        self.assertIn("mod phases;", plugin_module)
        self.assertIn("pub use phases::EditorPluginLoadingPhase;", plugin_module)
        self.assertIn("pub enum EditorPluginLoadingPhase", phases)
        self.assertIn("PreWorkbench", phases)
        self.assertIn("PostWorkbench", phases)
        self.assertNotIn("pub enum EditorPluginLoadingPhase", manager)

    def test_plugin_registration_uses_one_panic_isolation_boundary(self) -> None:
        plugin_module = (PLUGIN_ROOT / "mod.rs").read_text(encoding="utf-8")
        isolation = (PLUGIN_ROOT / "isolation.rs").read_text(encoding="utf-8")
        registration = (PLUGIN_ROOT / "registration.rs").read_text(encoding="utf-8")
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        state = MANAGER_STATE.read_text(encoding="utf-8")
        replacement_owner = MANAGER_REPLACEMENT_OWNER.read_text(encoding="utf-8")

        self.assertIn("mod isolation;", plugin_module)
        self.assertIn("pub fn run_editor_plugin_boundary", isolation)
        self.assertIn("catch_unwind", isolation)
        self.assertIn("AssertUnwindSafe", isolation)
        self.assertIn("candidate_extensions", registration)
        self.assertIn("extensions = candidate_extensions;", registration)
        self.assertIn("run_editor_plugin_boundary", registration)
        self.assertIn("EditorPluginState::Faulted", manager)
        self.assertIn("pub enum EditorPluginState", state)

    def test_plugin_state_machine_exposes_only_legal_lifecycle_edges(self) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        state = MANAGER_STATE.read_text(encoding="utf-8")

        self.assertIn("Revoking", state)
        self.assertIn("pub fn can_transition_to", state)
        self.assertIn("Self::Discovered =>", state)
        self.assertIn("Self::Validated =>", state)
        self.assertIn("Self::Loading =>", state)
        self.assertIn("Self::Active =>", state)
        self.assertIn("Self::Revoking =>", state)
        self.assertIn("Self::Disabled =>", state)
        self.assertIn("Self::Faulted =>", state)
        self.assertIn("pub fn transition_state", manager)

    def test_external_lifecycle_events_publish_once_for_the_active_plugin_set(self) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        tests = MANAGER_TESTS.read_text(encoding="utf-8")
        broadcast = (
            MANAGER_MODULE_ROOT / "tests" / "lifecycle_broadcast.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("mod lifecycle_broadcast;", tests)
        self.assertIn("pub fn dispatch_lifecycle_event_to_active", manager)
        self.assertIn("entry.state == EditorPluginState::Active", manager)
        self.assertIn("external_lifecycle_broadcast_updates_active_plugins_in_one_generation", broadcast)
        self.assertIn("external_lifecycle_broadcast_faults_only_the_callback_that_fails", broadcast)
        self.assertIn("external_lifecycle_broadcast_rejects_manager_owned_activation_stages", broadcast)
        self.assertIn("after.generation(), before.generation() + 1", broadcast)
        self.assertIn("EditorPluginLifecycleStage::Loaded", broadcast)
        self.assertIn("EditorPluginLifecycleStage::Enabled", broadcast)
        self.assertIn("EditorPluginLifecycleStage::Disabled", broadcast)
        self.assertIn("let next_report", broadcast)
        self.assertIn('"plugin.broadcast.healthy"', broadcast)

    def test_manager_accepts_one_explicit_discovery_input_per_package(self) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        discovery = MANAGER_DISCOVERY.read_text(encoding="utf-8")

        self.assertIn("pub struct EditorPluginDiscovery", discovery)
        self.assertIn("pub(crate) fn new_with_discoveries", manager)
        self.assertIn("DuplicateDiscovery", discovery)
        self.assertIn("UnknownPackage", discovery)
        self.assertIn("EditorPluginSource::Project", discovery)
        self.assertIn("EditorPluginLoadingPhase::PreWorkbench", discovery)

    def test_core_manager_is_the_catalog_store_owner(self) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        publication = MANAGER_PUBLICATION.read_text(encoding="utf-8")
        snapshot = MANAGER_SNAPSHOT.read_text(encoding="utf-8")
        catalog = (PLUGIN_ROOT / "catalog.rs").read_text(encoding="utf-8")
        plugin_module = (PLUGIN_ROOT / "mod.rs").read_text(encoding="utf-8")
        library = (REPOSITORY_ROOT / "zircon_editor" / "src" / "lib.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("pub struct EditorPluginManager", manager)
        self.assertIn("catalog_store: EditorPluginCatalogStore", manager)
        self.assertIn("pub struct EditorPluginManagerSnapshot", snapshot)
        self.assertIn("RwLock<Arc<EditorPluginManagerSnapshot>>", manager)
        self.assertIn("pub fn catalog_snapshot", manager)
        self.assertIn("pub fn state_snapshot", manager)
        self.assertIn("pub fn set_enabled", manager)
        self.assertIn("pub fn from_plugins", manager)
        self.assertIn("pub fn from_descriptors", manager)
        self.assertIn("pub(crate) fn new", manager)
        self.assertIn("pub(crate) fn publish_catalog", publication)
        self.assertIn("pub(crate) struct EditorPluginCatalog", catalog)
        self.assertIn("pub(crate) use catalog::EditorPluginCatalog;", plugin_module)
        self.assertNotIn("EditorPluginCatalog", library)
        self.assertIn("previous_by_package", manager)
        self.assertNotIn(
            ".find(|entry| entry.package_id == package.id)", manager
        )

    def test_catalog_replacement_uses_sorted_retraction_lookup(self) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        retraction = manager.split("fn active_package_retracted", 1)[1].split(
            "fn has_failed_disabled_lifecycle", 1
        )[0]

        self.assertIn("binary_search_by", retraction)
        self.assertNotIn(
            ".find(|entry| entry.package_id == active.package_id)", retraction
        )

    def test_editor_manager_delegates_catalog_reads_to_core_manager(self) -> None:
        plugin_module = (PLUGIN_ROOT / "mod.rs").read_text(encoding="utf-8")
        editor_manager = EDITOR_MANAGER.read_text(encoding="utf-8")
        project_manager = EDITOR_MANAGER_PROJECT.read_text(encoding="utf-8")
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        catalog = (PLUGIN_ROOT / "catalog.rs").read_text(encoding="utf-8")
        registration = (PLUGIN_ROOT / "registration.rs").read_text(
            encoding="utf-8"
        )
        project_selection_test = MANAGER_PROJECT_SELECTION_TEST.read_text(
            encoding="utf-8"
        )
        project_registration_test = (
            MANAGER_MODULE_ROOT / "tests" / "project_registration.rs"
        ).read_text(encoding="utf-8")
        project_registration = MANAGER_PROJECT_REGISTRATION.read_text(
            encoding="utf-8"
        )
        native_registration = NATIVE_REGISTRATION_MANAGER.read_text(
            encoding="utf-8"
        )
        project_enablement = PROJECT_ENABLEMENT.read_text(encoding="utf-8")
        commandlet_runner = COMMANDLET_RUNNER.read_text(encoding="utf-8")
        plugin_exports = (
            REPOSITORY_ROOT
            / "zircon_editor"
            / "src"
            / "ui"
            / "host"
            / "editor_manager_plugins_export"
            / "mod.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub use manager::EditorPluginManager", plugin_module)
        self.assertIn("plugin_manager: EditorPluginManager", editor_manager)
        self.assertNotIn("plugin_catalog: EditorPluginCatalogStore", editor_manager)
        self.assertIn("EditorPluginManager::builtin(", editor_manager)
        self.assertNotIn("EditorPluginManager::builtin_shared()", editor_manager)
        self.assertIn("EditorPluginManager::builtin_shared()", commandlet_runner)
        self.assertNotIn("EditorPluginDescriptor::builtin_catalog_projection", commandlet_runner)
        self.assertNotIn("pub fn editor_plugin_capabilities", plugin_exports)
        self.assertIn("pub(crate) fn update_editor_plugin_state", editor_manager)
        self.assertNotIn("pub(crate) fn set_editor_plugin_enabled", editor_manager)
        self.assertIn(
            "self.set_editor_plugin_enabled_unpublished(plugin_id, enabled)?",
            project_enablement,
        )
        self.assertNotIn("editor_catalog.package_manifests()", project_enablement)
        self.assertNotIn("pub fn enable_project_plugin", project_enablement)
        self.assertNotIn("pub fn disable_project_plugin", project_enablement)
        self.assertIn("publish_project_registration_reports", project_registration)
        self.assertIn("clear_project_registration_reports", project_registration)
        self.assertIn("replace_project_registration_reports", catalog)
        self.assertIn("record_host_lifecycle_event", registration)
        self.assertIn(
            "selected_native_editor_plugin_registration_reports", project_manager
        )
        self.assertIn("publish_project_registration_reports", project_manager)
        self.assertIn("clear_project_registration_reports", project_manager)
        self.assertIn(
            "project_native_reports_replace_only_project_rows_and_record_host_lifecycle",
            project_registration_test,
        )
        self.assertIn("selected_native_editor_plugin_registration_reports", native_registration)

    def test_plugin_enablement_has_one_public_facade_and_updates_manager_state(self) -> None:
        editor_manager = EDITOR_MANAGER.read_text(encoding="utf-8")
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        publication = MANAGER_PUBLICATION.read_text(encoding="utf-8")
        state = MANAGER_STATE.read_text(encoding="utf-8")
        capabilities = (
            REPOSITORY_ROOT
            / "zircon_editor"
            / "src"
            / "ui"
            / "host"
            / "editor_manager_plugins_export"
            / "enablement"
            / "capabilities.rs"
        ).read_text(encoding="utf-8")

        self.assertEqual(capabilities.count("pub fn set_editor_plugin_enabled("), 1)
        self.assertIn("self.plugin_manager", editor_manager)
        self.assertIn(".set_enabled(plugin_id, enabled)", editor_manager)
        self.assertIn("pub(crate) fn validate_editor_plugin_state", editor_manager)
        self.assertIn("pub fn validate_enablement", manager)
        self.assertIn("InvalidEnablement", state)
        public_enablement = capabilities.split("pub fn set_editor_plugin_enabled", 1)[1].split(
            "pub(in crate::ui::host) fn set_editor_plugin_enabled_unpublished", 1
        )[0]
        plugin_enablement = capabilities.split(
            "pub(in crate::ui::host) fn set_editor_plugin_enabled_unpublished", 1
        )[1]
        self.assertIn(
            "self.set_editor_plugin_enabled_unpublished(plugin_id, enabled)?",
            public_enablement,
        )
        self.assertIn("self.refresh_builtin_plugin_status();", public_enablement)
        self.assertLess(
            public_enablement.index(
                "self.set_editor_plugin_enabled_unpublished(plugin_id, enabled)?"
            ),
            public_enablement.index("self.refresh_builtin_plugin_status();"),
        )
        self.assertIn(
            "self.update_editor_plugin_state_unpublished(plugin_id, enabled)",
            plugin_enablement,
        )
        self.assertLess(
            plugin_enablement.index("self.set_editor_capabilities_with_previous"),
            plugin_enablement.index(
                "self.update_editor_plugin_state_unpublished(plugin_id, enabled)"
            ),
        )

    def test_enablement_preflights_and_rolls_back_before_manifest_publish(self) -> None:
        capabilities = (
            REPOSITORY_ROOT
            / "zircon_editor"
            / "src"
            / "ui"
            / "host"
            / "editor_manager_plugins_export"
            / "enablement"
            / "capabilities.rs"
        ).read_text(encoding="utf-8")
        project_enablement = PROJECT_ENABLEMENT.read_text(encoding="utf-8")
        plugin_enablement = capabilities.split(
            "pub(in crate::ui::host) fn set_editor_plugin_enabled_unpublished", 1
        )[1]

        self.assertIn("set_editor_capabilities_with_previous", capabilities)
        self.assertIn("restore_editor_capabilities", capabilities)
        self.assertIn("self.validate_editor_plugin_state(plugin_id, enabled)?;", plugin_enablement)
        self.assertLess(
            plugin_enablement.index("self.validate_editor_plugin_state(plugin_id, enabled)?;"),
            plugin_enablement.index("self.set_editor_capabilities_with_previous"),
        )
        self.assertLess(
            plugin_enablement.index("self.set_editor_capabilities_with_previous"),
            plugin_enablement.index(
                "self.update_editor_plugin_state_unpublished(plugin_id, enabled)"
            ),
        )
        self.assertIn(
            "self.restore_editor_capabilities(&core, &previous_capabilities)",
            plugin_enablement,
        )
        self.assertLess(
            project_enablement.index(
                "self.set_editor_plugin_enabled_unpublished(plugin_id, enabled)?"
            ),
            project_enablement.index("manifest.plugins.set_enabled(selection.clone());"),
        )

    def test_capability_enablement_serializes_the_config_transaction_and_keeps_its_core(self) -> None:
        editor_manager = EDITOR_MANAGER.read_text(encoding="utf-8")
        capabilities = (
            REPOSITORY_ROOT
            / "zircon_editor"
            / "src"
            / "ui"
            / "host"
            / "editor_manager_plugins_export"
            / "enablement"
            / "capabilities.rs"
        ).read_text(encoding="utf-8")
        ui_host = (
            REPOSITORY_ROOT / "zircon_editor" / "src" / "ui" / "host" / "editor_ui_host.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("capability_updates: Mutex<()>", editor_manager)
        self.assertIn("fn lock_editor_capability_updates", editor_manager)
        self.assertEqual(capabilities.count("self.lock_editor_capability_updates();"), 2)
        self.assertIn("Result<(CoreHandle, EditorCapabilitySnapshot, Vec<String>), String>", capabilities)
        self.assertIn("fn restore_editor_capabilities(", capabilities)
        self.assertIn("core: &CoreHandle", capabilities)
        self.assertNotIn("self.host.runtime_core()", capabilities.split("fn restore_editor_capabilities", 1)[1])
        self.assertIn("refresh_capabilities_from_core", capabilities)
        self.assertIn("pub(super) fn refresh_capabilities_from_core", ui_host)

    def test_loading_phase_publishes_one_manager_owned_active_extension_view(self) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        snapshot = MANAGER_SNAPSHOT.read_text(encoding="utf-8")
        state = MANAGER_STATE.read_text(encoding="utf-8")
        catalog_snapshot = (PLUGIN_ROOT / "catalog_snapshot.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("active_extensions: Arc<EditorExtensionCatalogReport>", snapshot)
        self.assertIn("reached_loading_phase: Option<", snapshot)
        self.assertIn("pub fn advance_loading_phase", manager)
        self.assertIn("InvalidLoadingPhaseAdvance", state)
        self.assertIn("fn build_active_extensions", manager)
        self.assertIn("state: EditorPluginState::Validated", snapshot)
        self.assertIn("pub fn active_extensions", snapshot)
        self.assertIn("pub(crate) fn registrations", catalog_snapshot)
        self.assertNotIn(
            "loading_phase <=",
            (PLUGIN_ROOT / "panel_source.rs").read_text(encoding="utf-8"),
        )

    def test_template_root_stays_owner_bound_across_hot_replacement(self) -> None:
        extension = EDITOR_EXTENSION.read_text(encoding="utf-8")
        contributions = TEMPLATE_CONTRIBUTIONS.read_text(encoding="utf-8")
        registration = contributions.split("pub fn register_ui_template", 1)[1].split(
            "pub fn register_ui_template_pane_data_source", 1
        )[0]
        replacement = contributions.split(
            "pub fn replace_ui_template_contributions", 1
        )[1].split("pub fn ui_templates", 1)[0]

        self.assertIn("mod template_contributions;", extension)
        self.assertIn("pub use template_contributions::EditorUiTemplateDescriptor;", extension)
        self.assertIn("ui_template_root: Option<PathBuf>", extension)
        self.assertIn("self.ui_template_root = Some(root.clone());", contributions)
        self.assertIn("if descriptor.plugin_root.is_none()", registration)
        self.assertIn(
            "descriptor.plugin_root = self.ui_template_root.clone();", registration
        )
        self.assertIn(".or_else(|| self.ui_template_root.clone())", replacement)
        self.assertIn(
            "template_replacement_binds_new_ids_to_the_host_plugin_root", contributions
        )

    def test_extension_registry_keeps_contribution_descriptor_models_in_a_leaf_owner(self) -> None:
        extension = EDITOR_EXTENSION.read_text(encoding="utf-8")
        descriptors = (
            EDITOR_EXTENSION.parent
            / "editor_extension"
            / "contribution_descriptors.rs"
        ).read_text(encoding="utf-8")
        inspector = (
            EDITOR_EXTENSION.parent / "extension" / "inspector.rs"
        ).read_text(encoding="utf-8")
        materialization = (
            PLUGIN_ROOT / "extension_materialization.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("mod contribution_descriptors;", extension)
        self.assertIn("pub use contribution_descriptors::{", extension)
        self.assertNotIn("pub struct DrawerDescriptor", extension)
        self.assertNotIn("pub struct AssetImporterDescriptor", extension)
        self.assertIn("pub struct DrawerDescriptor", descriptors)
        self.assertIn("pub struct EditorMenuItemDescriptor", descriptors)
        self.assertNotIn("ComponentDrawerDescriptor", descriptors)
        self.assertIn("pub struct AssetImporterDescriptor", descriptors)
        self.assertIn("fn validate_asset_importer", descriptors)
        self.assertIn("pub struct InspectorCustomizationDescriptor", inspector)
        self.assertIn("registry.register_inspector_customization", materialization)

    def test_dynamic_control_updates_clear_stale_table_selection(self) -> None:
        action_registry = (
            REPOSITORY_ROOT
            / "zircon_editor"
            / "src"
            / "ui"
            / "template_runtime"
            / "runtime"
            / "template_action_registry.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("fn clear_stale_table_selection", action_registry)
        self.assertIn(
            "clear_stale_table_selection(control_attributes);", action_registry
        )
        self.assertIn(
            "dynamic_row_updates_clear_a_stale_table_selection", action_registry
        )

    def test_loading_phase_is_an_unbypassable_publish_invariant(self) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        snapshot = MANAGER_SNAPSHOT.read_text(encoding="utf-8")
        state = MANAGER_STATE.read_text(encoding="utf-8")
        extension_report = (
            PLUGIN_ROOT / "extension_catalog_report.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("active_manager_generation: Option<u64>", extension_report)
        self.assertIn(
            "report.active_manager_generation = Some(manager_generation);", manager
        )
        self.assertIn("LoadingPhaseUnavailable", state)
        self.assertIn("normalize_entries_for_loading_phase", manager)
        transition = manager.split("pub fn transition_state", 1)[1].split(
            "pub fn advance_loading_phase", 1
        )[0]
        self.assertIn("is_phase_gated_state", transition)
        self.assertIn("phase_is_reached", transition)

    def test_manager_owns_phase_lifecycle_dispatch(self) -> None:
        manager = (PLUGIN_ROOT / "manager.rs").read_text(encoding="utf-8")
        publication = MANAGER_PUBLICATION.read_text(encoding="utf-8")
        discovery = MANAGER_DISCOVERY.read_text(encoding="utf-8")
        state = MANAGER_STATE.read_text(encoding="utf-8")
        replacement_owner = MANAGER_REPLACEMENT_OWNER.read_text(encoding="utf-8")
        lifecycle_replacement = MANAGER_LIFECYCLE_REPLACEMENT.read_text(
            encoding="utf-8"
        )
        registration = (PLUGIN_ROOT / "registration.rs").read_text(
            encoding="utf-8"
        )
        catalog = (PLUGIN_ROOT / "catalog.rs").read_text(encoding="utf-8")
        catalog_snapshot = (PLUGIN_ROOT / "catalog_snapshot.rs").read_text(
            encoding="utf-8"
        )
        event_registration = (
            REPOSITORY_ROOT
            / "zircon_editor"
            / "src/tests/editor_event/runtime/extensions_registration.rs"
        ).read_text(encoding="utf-8")
        native_registration_projection = (
            REPOSITORY_ROOT
            / "zircon_editor"
            / "src/ui/host/editor_manager_plugins_export/native_registration"
            / "registration_projection.rs"
        ).read_text(encoding="utf-8")
        registration_build = registration.split("pub fn from_plugin", 1)[1].split(
            "pub fn record_lifecycle_event", 1
        )[0]
        transition = manager.split("pub fn transition_state", 1)[1].split(
            "pub fn advance_loading_phase", 1
        )[0]
        enablement = manager.split("fn validate_enablement", 1)[1].split(
            "fn lifecycle_stage", 1
        )[0]

        self.assertIn("fn activate_eligible_entries", manager)
        self.assertIn("pub fn dispatch_lifecycle_event", manager)
        self.assertIn("EditorPluginLifecycleStage::Loaded", manager)
        self.assertIn("EditorPluginLifecycleStage::Enabled", manager)
        self.assertNotIn("record_lifecycle_stage(", registration_build)
        self.assertIn("pub(super) fn record_lifecycle_event", registration)
        self.assertIn("successful_lifecycle_stages", registration)
        self.assertIn("failed_lifecycle_stages", registration)
        self.assertIn("lifecycle_stage_succeeded", catalog)
        self.assertIn("lifecycle_stage_failed", catalog)
        self.assertIn("BUILTIN_EDITOR_PLUGIN_MANAGER_INIT", manager)
        self.assertIn("initialize_once", manager)
        self.assertIn("PhaseRetractionRequiresDisable", publication)
        self.assertIn("reset_replaced_active_entries", replacement_owner)
        self.assertIn("DisabledLifecycleRetryRequired", manager)
        self.assertIn(
            "EditorPluginState::Faulted | EditorPluginState::Active | EditorPluginState::Revoking",
            replacement_owner,
        )
        discovery_errors = discovery.split("pub enum EditorPluginDiscoveryError", 1)[1].split(
            "impl fmt::Display for EditorPluginDiscoveryError", 1
        )[0]
        transition_errors = state.split("pub enum EditorPluginTransitionError", 1)[1].split(
            "impl fmt::Display for EditorPluginTransitionError", 1
        )[0]
        self.assertIn("DisabledLifecycleRetryRequired", discovery_errors)
        self.assertIn("DisabledLifecycleRetryRequired", transition_errors)
        self.assertIn("has_failed_disabled_lifecycle", transition)
        self.assertIn("has_failed_disabled_lifecycle", enablement)
        self.assertIn("next_state == EditorPluginState::Validated", transition)
        self.assertIn("ManagedLifecycleTransitionRequired", transition)
        self.assertIn("if has_failed_disabled_lifecycle", enablement)
        self.assertIn("DisabledLifecycleRetryRequired", enablement)
        self.assertNotIn("replacement_bypasses_failed_disabled_lifecycle", replacement_owner)
        self.assertIn(
            "EditorPluginState::Faulted | EditorPluginState::Active | EditorPluginState::Revoking",
            replacement_owner,
        )
        active_replacement = lifecycle_replacement.split(
            "fn replacing_an_active_package_retires_the_old_instance_before_hot_reload",
            1,
        )[1].split("#[test]", 1)[0]
        faulted_replacement = lifecycle_replacement.split(
            "fn replacing_a_faulted_package_dispatches_lifecycle_for_the_new_plugin_instance",
            1,
        )[1].split("#[test]", 1)[0]
        for replacement in (active_replacement, faulted_replacement):
            self.assertIn("Arc::clone(&replacement_plugin)", replacement)
            self.assertIn("Some(EditorPluginState::Active)", replacement)
            self.assertIn("EditorPluginLifecycleStage::Loaded", replacement)
            self.assertIn("EditorPluginLifecycleStage::Enabled", replacement)
        self.assertNotIn("materialized_extensions", catalog)
        self.assertNotIn("pub fn editor_extensions", catalog)
        self.assertNotIn("extensions: Arc<EditorExtensionCatalogReport>", catalog_snapshot)
        self.assertNotIn("pub fn editor_extensions", catalog_snapshot)
        for registration_literal in (
            event_registration,
            native_registration_projection,
        ):
            self.assertIn(
                "successful_lifecycle_stages: Vec::new(),", registration_literal
            )
            self.assertIn("failed_lifecycle_stages: Vec::new(),", registration_literal)


if __name__ == "__main__":
    unittest.main()
