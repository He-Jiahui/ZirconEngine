from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RUNTIME_HOST = ROOT / "zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs"
PANE_STATE = (
    ROOT
    / "zircon_editor/src/ui/template_runtime/runtime/runtime_host/dynamic_control_state.rs"
)
PANE_PROJECTION = ROOT / "zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs"


class EditorTemplateProjectionCachePerformanceContractTests(unittest.TestCase):
    def test_runtime_caches_stable_document_projection_and_invalidates_on_registration(self):
        source = RUNTIME_HOST.read_text(encoding="utf-8")
        self.assertIn("projection_cache", source)
        self.assertIn("template_instance_cache", source)
        self.assertIn("invalidate_projection_cache", source)
        self.assertIn("project_document_cached", source)
        self.assertIn("template_instance_cached", source)
        self.assertIn("self.projection_cache", source)

    def test_pane_projection_applies_dynamic_attributes_after_cached_base(self):
        source = PANE_STATE.read_text(encoding="utf-8")
        self.assertIn("project_document_cached", source)
        self.assertIn("inject_pane_projection_attributes", source)
        self.assertNotIn("project_pane_body(\n            &self.template_service", source)

    def test_legacy_pane_projection_does_not_instantiate_before_projection(self):
        source = PANE_PROJECTION.read_text(encoding="utf-8")
        self.assertNotIn("template_service\n        .instantiate", source)
        self.assertIn("template_instance_cached", RUNTIME_HOST.read_text(encoding="utf-8"))
        self.assertIn("inject_pane_projection_attributes", PANE_STATE.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
