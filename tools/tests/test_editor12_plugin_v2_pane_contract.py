"""Static contract tests for generic Editor12 plugin V2 panes."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EDITOR = ROOT / "zircon_editor" / "src"


class PluginV2PaneContractTests(unittest.TestCase):
    def test_extension_views_bind_their_declared_template_without_a_plugin_branch(self) -> None:
        descriptor = (EDITOR / "core" / "editor_extension" / "view_descriptor.rs").read_text(
            encoding="utf-8"
        )
        registry = (EDITOR / "core" / "editor_extension.rs").read_text(encoding="utf-8")
        bridge = (EDITOR / "ui" / "host" / "editor_extension_views.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("with_ui_template_id", descriptor)
        self.assertIn("ui_template_id", descriptor)
        self.assertIn("bind_matching_ui_templates_to_views", registry)
        self.assertIn("PanePayloadKind::TemplateV2", bridge)
        self.assertIn("PaneRouteNamespace::Template", bridge)
        self.assertNotIn("navigation", bridge.lower())

    def test_plugin_owned_template_data_is_typed_and_projects_through_the_generic_pane(self) -> None:
        descriptor = (EDITOR / "core" / "editor_extension" / "view_descriptor.rs").read_text(
            encoding="utf-8"
        )
        registry = (EDITOR / "core" / "editor_extension.rs").read_text(encoding="utf-8")
        contributions = (
            EDITOR
            / "core"
            / "editor_extension"
            / "template_contributions.rs"
        ).read_text(encoding="utf-8")
        payload = (
            EDITOR
            / "ui"
            / "layouts"
            / "windows"
            / "workbench_host_window"
            / "pane_payload.rs"
        ).read_text(encoding="utf-8")
        projection = (
            EDITOR / "ui" / "template_runtime" / "runtime" / "pane_payload_projection.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("EditorUiTemplatePaneDataSource", descriptor)
        self.assertIn("EditorUiTemplatePaneDataSnapshot", descriptor)
        self.assertIn("component_patches: Vec<UiComponentProjectionPatch>", descriptor)
        self.assertIn("component_patches: Vec::new()", descriptor)
        self.assertIn("with_component_patch", descriptor)
        self.assertIn("mod template_contributions;", registry)
        self.assertIn("register_ui_template_pane_data_source", contributions)
        self.assertIn("TemplateV2PanePayload", payload)
        self.assertIn("UiValue", payload)
        self.assertIn("UiComponentProjectionPatch", payload)
        self.assertIn("template_data", projection)
        self.assertIn("ui_value_to_toml", projection)
        self.assertIn("inject_template_v2_component_patches", projection)
        self.assertIn("apply_component_projection_patch", projection)
        self.assertNotIn("navigation", payload.lower())
        self.assertNotIn("navigation", projection.lower())

    def test_plugin_template_sync_prepares_all_owner_candidates_before_publishing(self) -> None:
        app = (EDITOR / "ui" / "retained_host" / "app.rs").read_text(encoding="utf-8")
        plugin_documents = (
            EDITOR / "ui" / "template_runtime" / "runtime" / "plugin_documents.rs"
        ).read_text(encoding="utf-8")
        plugin_document_tests = (
            EDITOR
            / "ui"
            / "template_runtime"
            / "runtime"
            / "plugin_documents"
            / "tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("sync_plugin_v2_template_descriptor_sets", plugin_documents)
        self.assertIn("compile_plugin_v2_document_sources", plugin_documents)
        self.assertIn("replace_compiled_plugin_v2_document_batch", plugin_documents)
        self.assertIn('#[path = "plugin_documents/tests.rs"]', plugin_documents)
        self.assertIn(
            "batch_template_sync_keeps_last_good_documents_when_any_candidate_fails",
            plugin_document_tests,
        )
        self.assertIn(
            "sync_plugin_v2_template_descriptor_sets(&templates_by_owner)", app
        )
        self.assertNotIn("self.plugin_template_owners.difference", app)


if __name__ == "__main__":
    unittest.main()
