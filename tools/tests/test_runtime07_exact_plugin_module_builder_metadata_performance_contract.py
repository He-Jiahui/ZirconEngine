from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_plugins/plugin_sdk/src/manifest/plugin_module_builder.rs"


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


class ExactPluginModuleBuilderMetadataPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.joiner = function_body(cls.source, "fn join_module_metadata(")

    def test_joiner_preallocates_the_exact_combined_length(self) -> None:
        self.assertIn("parts.iter().map(|part| part.len()).sum()", self.joiner)
        self.assertIn("String::with_capacity(capacity)", self.joiner)
        self.assertIn("joined.push_str(part)", self.joiner)

    def test_module_constructors_use_the_exact_joiner(self) -> None:
        for signature, suffix in (
            ("pub fn runtime(", ".runtime"),
            ("pub fn editor(", ".editor"),
            ("pub fn native(", ".native"),
            ("pub fn vm(", ".vm"),
        ):
            with self.subTest(signature=signature):
                body = function_body(self.source, signature)
                self.assertIn("join_module_metadata", body)
                self.assertIn(f'"{suffix}"', body)
                self.assertNotIn("format!", body)

    def test_default_description_uses_the_same_exact_joiner(self) -> None:
        body = function_body(self.source, "pub fn new(")
        self.assertIn(
            'description: join_module_metadata(&["Plugin module ", &name])',
            body,
        )
        self.assertNotIn("format!", body)

    def test_rust_regression_covers_all_builtin_module_kinds(self) -> None:
        self.assertIn(
            "exact_module_metadata_preserves_builtin_names_and_descriptions",
            self.source,
        )
        for suffix in ("runtime", "editor", "native", "vm"):
            self.assertIn(f'"weather.{suffix}"', self.source)


if __name__ == "__main__":
    unittest.main()
