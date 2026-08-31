from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/"
    "importer_classification/capabilities.rs"
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


class SingleAllocationImporterFallbackCapabilityPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(cls.source, "pub(super) fn primary_importer_capability(")

    def test_fallback_preallocates_prefix_plus_slug_length(self) -> None:
        self.assertIn("IMPORTER_CAPABILITY_PREFIX", self.source)
        self.assertIn("IMPORTER_CAPABILITY_PREFIX.len()", self.body)
        self.assertIn("slug.len()", self.body)
        self.assertIn("String::with_capacity(capacity)", self.body)
        self.assertIn("capability.push_str(IMPORTER_CAPABILITY_PREFIX)", self.body)

    def test_fallback_translates_underscores_while_writing_output(self) -> None:
        self.assertIn("for character in slug.chars()", self.body)
        self.assertIn("'_' => capability.push('.')", self.body)
        self.assertIn("character => capability.push(character)", self.body)

    def test_fallback_has_no_intermediate_owned_slug_or_format(self) -> None:
        self.assertNotIn(".replace('_', \".\")", self.body)
        self.assertNotIn("format!", self.body)

    def test_rust_regression_covers_suffix_and_plain_fallback_ids(self) -> None:
        self.assertIn(
            "fallback_importer_capability_writes_slug_into_single_output",
            self.source,
        )
        self.assertIn('"custom_mesh_importer"', self.source)
        self.assertIn('"procedural_cache"', self.source)


if __name__ == "__main__":
    unittest.main()
