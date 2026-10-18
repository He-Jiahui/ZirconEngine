import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_editor/src/ui/template_runtime/runtime/build_session.rs"
REGRESSION = (
    REPO_ROOT / "zircon_editor/src/ui/template_runtime/runtime/build_session/tests/editor798_document_id_capacity_tests.rs"
)


class EditorBuiltinTemplateDocumentIdCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8")

    def test_requested_document_ids_reserve_the_input_bound(self) -> None:
        body = self.source.split("pub(super) fn load_builtin_host_templates_for_document_ids", 1)[1]
        body = body.split("fn load_builtin_host_templates_for_documents", 1)[0]
        self.assertIn(
            "HashSet::with_capacity(document_ids.len())",
            body,
        )
        self.assertIn("requested_document_ids.extend(document_ids.iter().copied())", body)
        self.assertNotIn("collect::<HashSet<_>>()", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"build_session/tests/editor798_document_id_capacity_tests.rs\"]",
            self.source,
        )
        self.assertIn(
            "optimization_batch_20260918_editor798_builtin_template_document_id_capacity_bench",
            self.regression,
        )
        self.assertIn(
            "EDITOR798_BUILTIN_TEMPLATE_DOCUMENT_ID_CAPACITY_BENCH_V1",
            self.regression,
        )

    def test_existing_hash_membership_authority_remains(self) -> None:
        production = self.source.split("#[cfg(test)]", 1)[0]
        self.assertIn("Option<&HashSet<&str>>", production)
        self.assertIn("seen_imports.contains(reference)", production)


if __name__ == "__main__":
    unittest.main()
