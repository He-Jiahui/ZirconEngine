"""Static contract tests for Editor12 retained plugin-template synchronization."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
APP = ROOT / "zircon_editor" / "src" / "ui" / "retained_host" / "app.rs"
TICK = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "retained_host"
    / "app"
    / "host_lifecycle"
    / "tick.rs"
)
CATALOG = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "template_runtime"
    / "runtime"
    / "plugin_documents.rs"
)


class RetainedPluginTemplateSyncContractTests(unittest.TestCase):
    def test_retained_host_syncs_the_registered_plugin_owner_set_into_its_long_lived_runtime(
        self,
    ) -> None:
        source = APP.read_text(encoding="utf-8")

        self.assertIn("sync_plugin_template_documents_if_changed", source)
        self.assertIn("sync_plugin_v2_template_descriptor_sets", source)
        self.assertIn("register_editor_plugin_registration", source)
        self.assertNotIn("mod plugin_template_documents;", source)
        self.assertNotIn("PluginTemplateDocumentResolver", source)

    def test_retained_tick_consumes_runtime_template_changes_before_presentation_work(
        self,
    ) -> None:
        app = APP.read_text(encoding="utf-8")
        tick = TICK.read_text(encoding="utf-8")

        self.assertIn("self.runtime.plugin_template_revision()", app)
        self.assertIn("self.runtime.enabled_plugin_template_descriptors()", app)
        self.assertIn("self.mark_presentation_dirty()", app)

        runtime_events = tick.index("self.runtime.pump_runtime_event_consumers()")
        template_sync = tick.index("self.sync_plugin_template_documents_if_changed()")
        presentation = tick.index("self.sync_pending_play_decisions();")
        self.assertLess(runtime_events, template_sync)
        self.assertLess(template_sync, presentation)

    def test_catalog_resolves_owned_plugin_uris_and_replaces_by_generation(self) -> None:
        source = CATALOG.read_text(encoding="utf-8")
        app = APP.read_text(encoding="utf-8")

        self.assertIn("sync_plugin_v2_template_descriptor_sets", app)
        self.assertIn("replace_compiled_plugin_v2_document_batch", source)
        self.assertIn("plugins://", source)
        self.assertIn('segment == ".."', source)
        self.assertIn("plugin_root", source)
        self.assertNotIn("navigation", source.lower())


if __name__ == "__main__":
    unittest.main()
