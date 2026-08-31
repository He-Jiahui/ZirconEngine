from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ENTITY_PATH = ROOT / "zircon_runtime/src/core/framework/scene/entity_path.rs"
MODULE_VALIDATION = (
    ROOT / "zircon_runtime/src/plugin/extension_registry/validation/runtime_core.rs"
)


def rust_block(source: str, signature: str) -> str:
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
    raise AssertionError(f"unterminated Rust block: {signature}")


class RuntimePathModuleValidationPerformanceContractTests(unittest.TestCase):
    def test_entity_path_preallocates_without_a_second_input_scan(self) -> None:
        source = ENTITY_PATH.read_text(encoding="utf-8")
        body = rust_block(source, "pub fn parse(path: &str)")

        self.assertIn("Vec::with_capacity(path.len().saturating_div(2).max(1))", body)
        self.assertEqual(body.count("path.split('/')"), 1)
        self.assertIn("segments.extend(", body)
        self.assertNotIn("path.matches", body)

    def test_entity_path_keeps_the_trim_filter_and_owned_segment_pipeline(self) -> None:
        body = rust_block(
            ENTITY_PATH.read_text(encoding="utf-8"), "pub fn parse(path: &str)"
        )

        self.assertIn(".map(str::trim)", body)
        self.assertIn(".filter(|segment| !segment.is_empty())", body)
        self.assertIn(".map(ToOwned::to_owned)", body)
        self.assertIn("Self::new(segments)", body)

    def test_module_field_caches_one_trimmed_slice(self) -> None:
        source = MODULE_VALIDATION.read_text(encoding="utf-8")
        body = rust_block(source, "fn module_field_is_valid(")

        self.assertEqual(body.count("value.trim()"), 1)
        self.assertIn("let trimmed = value.trim()", body)
        self.assertIn("!trimmed.is_empty()", body)
        self.assertIn("trimmed.len() == value.len()", body)

    def test_module_error_path_reuses_the_boolean_helper(self) -> None:
        source = MODULE_VALIDATION.read_text(encoding="utf-8")
        body = rust_block(source, "fn validate_module_field(")

        self.assertIn("if !module_field_is_valid(value)", body)
        self.assertNotIn("value.trim()", body)
        self.assertIn("must be non-empty and trimmed", body)


if __name__ == "__main__":
    unittest.main()
