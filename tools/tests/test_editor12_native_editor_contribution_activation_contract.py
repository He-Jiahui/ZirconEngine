"""Static contract tests for Editor12 native contribution host activation."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
NATIVE_REGISTRATION = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_plugins_export"
    / "native_registration"
)
PLUGIN_ROOT = ROOT / "zircon_editor" / "src" / "core" / "plugin"
PLAY_NATIVE_ACTIVATION = (
    ROOT / "zircon_editor" / "src" / "core" / "play" / "plugin_activation" / "native.rs"
)
LIVE_HOST_ROOT = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "app"
    / "module_plugin_actions"
    / "live_host"
)
EDITOR_PLUGIN_EXPORT = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_plugins_export"
    / "mod.rs"
)


class NativeEditorContributionActivationContractTests(unittest.TestCase):
    def test_native_registration_materializes_verified_batches_into_package_registries(
        self,
    ) -> None:
        manager = (NATIVE_REGISTRATION / "manager.rs").read_text(encoding="utf-8")
        materializer = (NATIVE_REGISTRATION / "native_contribution.rs").read_text(
            encoding="utf-8"
        )
        projection = (NATIVE_REGISTRATION / "registration_projection.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("materialize_native_editor_contributions", manager)
        self.assertIn("NativePluginLoadReport", materializer)
        self.assertIn("editor_entry_report", materializer)
        self.assertIn("editor_contribution_batch", materializer)
        self.assertIn(
            "materialize_serialized_native_contribution_batch",
            materializer,
        )
        self.assertIn("run_editor_plugin_boundary", materializer)
        self.assertIn("EditorExtensionRegistry::default()", materializer)
        self.assertIn("registration.extensions = EditorExtensionRegistry::default()", materializer)
        self.assertIn("extensions: EditorExtensionRegistry", projection)

    def test_host_uses_the_single_plugin_isolation_boundary(self) -> None:
        plugin_mod = (PLUGIN_ROOT / "mod.rs").read_text(encoding="utf-8")

        self.assertIn("run_editor_plugin_boundary", plugin_mod)
        self.assertIn("EditorPluginBoundaryFailure", plugin_mod)

    def test_selected_native_failures_remain_faulted_plugin_reports(self) -> None:
        manager = (NATIVE_REGISTRATION / "manager.rs").read_text(encoding="utf-8")

        self.assertIn(
            "selected_native_load_failure_remains_visible_to_the_plugin_manager",
            manager,
        )
        self.assertIn("report_unusable_native_entry", manager)
        self.assertIn("native editor entry is unavailable", manager)
        self.assertNotIn("require_usable_native_entry", manager)

    def test_project_native_loads_share_the_project_plugin_directory(self) -> None:
        plugin_module = (PLUGIN_ROOT / "mod.rs").read_text(encoding="utf-8")
        activation = PLAY_NATIVE_ACTIVATION.read_text(encoding="utf-8")
        backend = (LIVE_HOST_ROOT / "native_backend.rs").read_text(encoding="utf-8")
        watch = (LIVE_HOST_ROOT / "development_watch.rs").read_text(encoding="utf-8")
        export = EDITOR_PLUGIN_EXPORT.read_text(encoding="utf-8")

        self.assertIn("pub(crate) fn project_native_plugin_directory", plugin_module)
        self.assertIn('join("zircon_plugins")', plugin_module)
        for source in (activation, backend, watch, export):
            self.assertIn("project_native_plugin_directory", source)
        self.assertIn("resolver(project_root)", activation)
        self.assertIn("resolver(request.project_root)", backend)
        self.assertIn("resolver(&self.key.project_root)", watch)


if __name__ == "__main__":
    unittest.main()
