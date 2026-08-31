from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_completion/owner_selection.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function: {signature}")


class CompletedOwnerFeatureSelectionFastPathPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_existing_feature_index_is_borrowed_before_catalog_projection(self) -> None:
        body = function_body(self.source, "fn complete_owner_feature_selection(")
        index_lookup = body.index("feature_indices.get(feature.id.as_str()).copied()")
        projection = body.index("project_selection_from_feature_manifest(feature)")
        self.assertLess(index_lookup, projection)
        self.assertNotIn("feature_indices.get(&catalog_selection.id)", body)
        self.assertIn("let existing_index =", body)
        self.assertIn("owner_feature_selection_is_complete", body)

    def test_completion_guard_matches_only_fields_the_merge_can_fill(self) -> None:
        body = function_body(self.source, "fn owner_feature_selection_is_complete(")
        self.assertIn("selection.runtime_crate.is_some()", body)
        self.assertIn("selection.editor_crate.is_some()", body)
        self.assertIn("!selection.target_modes.is_empty()", body)
        self.assertIn("provider_package_id.is_none()", body)
        self.assertIn("selection.provider_package_id.is_some()", body)

    def test_rust_guards_cover_complete_and_provider_incomplete_paths(self) -> None:
        self.assertIn(
            "completed_owner_feature_selection_skips_catalog_projection_requirements",
            self.source,
        )
        self.assertIn(
            "provider_requirement_prevents_incomplete_fast_path",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
