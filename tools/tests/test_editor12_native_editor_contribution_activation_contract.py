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
        self.assertIn("materialize_serialized_contribution_batch", materializer)
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


if __name__ == "__main__":
    unittest.main()
