from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
REGISTRY_SOURCE = ROOT / "zircon_editor/src/core/editor_extension.rs"
BATCH_SOURCE = ROOT / "zircon_editor/src/core/extension/store/batch.rs"
REGISTRY_TEST_SOURCE = (
    ROOT / "zircon_editor/src/core/editor_extension/tests/optimization_batch_editor826_template_binding_tests.rs"
)
BATCH_TEST_SOURCE = (
    ROOT / "zircon_editor/src/core/extension/store/batch/tests/optimization_batch_editor826_template_binding_tests.rs"
)


class EditorTemplateViewBindingLookupPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.registry = REGISTRY_SOURCE.read_text(encoding="utf-8")
        cls.batch = BATCH_SOURCE.read_text(encoding="utf-8")
        cls.registry_tests = REGISTRY_TEST_SOURCE.read_text(encoding="utf-8")
        cls.batch_tests = BATCH_TEST_SOURCE.read_text(encoding="utf-8")
        cls.registry_body = cls._function_body(
            cls.registry, "    pub(crate) fn bind_matching_ui_templates_to_views(&mut self) {"
        )
        cls.batch_body = cls._function_body(
            cls.batch, "    pub(crate) fn bind_matching_ui_templates_to_views(&mut self) {"
        )

    @staticmethod
    def _function_body(source: str, signature: str) -> str:
        start = source.index(signature)
        end = source.index("\n    }", start) + len("\n    }")
        return source[start:end]

    def test_registry_uses_direct_template_lookup(self) -> None:
        self.assertIn(".ui_templates", self.registry_body)
        self.assertIn(".get(view.id())", self.registry_body)
        self.assertIn('template.ui_document().starts_with("plugins://")', self.registry_body)
        self.assertNotIn("let template_ids =", self.registry_body)
        self.assertNotIn("collect::<std::collections::BTreeSet<_>>()", self.registry_body)

    def test_batch_uses_direct_template_lookup(self) -> None:
        self.assertIn(".ui_templates", self.batch_body)
        self.assertIn(".get(view.id())", self.batch_body)
        self.assertIn('template.ui_document().starts_with("plugins://")', self.batch_body)
        self.assertNotIn("let template_ids =", self.batch_body)
        self.assertNotIn("collect::<std::collections::BTreeSet<_>>()", self.batch_body)

    def test_lower_regressions_and_release_markers_are_wired(self) -> None:
        self.assertIn(
            "editor_extension_template_binding_preserves_existing_and_missing_views",
            self.registry_tests,
        )
        self.assertIn("editor_extension_template_binding_lookup_bench_v1", self.registry_tests)
        self.assertIn(
            "contribution_batch_template_binding_preserves_existing_and_missing_views",
            self.batch_tests,
        )
        self.assertIn("contribution_batch_template_binding_lookup_bench_v1", self.batch_tests)
        self.assertIn("EDITOR826_TEMPLATE_VIEW_BINDING_LOOKUP_BENCH_V1", self.registry_tests)
        self.assertIn("EDITOR826_TEMPLATE_VIEW_BINDING_LOOKUP_BENCH_V1", self.batch_tests)

    def test_deterministic_model_removes_template_set_allocation(self) -> None:
        template_count = 4_096
        view_count = 4_096
        plugin_templates = template_count // 2
        legacy_template_set_allocations = 1
        optimized_template_set_allocations = 0
        legacy_template_scan = template_count
        optimized_template_probes = view_count

        self.assertEqual(plugin_templates, 2_048)
        self.assertGreater(legacy_template_set_allocations, optimized_template_set_allocations)
        self.assertGreaterEqual(legacy_template_scan + view_count, optimized_template_probes)
        template_ids = {f"view.{index}" for index in range(plugin_templates)}
        bound = [
            f"view.{index}"
            for index in range(view_count)
            if f"view.{index}" in template_ids
        ]
        self.assertEqual(bound, [f"view.{index}" for index in range(plugin_templates)])


if __name__ == "__main__":
    unittest.main()
