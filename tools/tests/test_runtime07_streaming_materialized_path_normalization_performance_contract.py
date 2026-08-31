from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/plugin/export_build_plan/materialize/paths.rs"


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


class StreamingMaterializedPathNormalizationPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.validation = function_body(
            cls.source,
            "pub(super) fn validated_materialized_relative_path(",
        )

    def test_normalization_preallocates_one_output_string(self) -> None:
        self.assertIn(
            "String::with_capacity(relative_path.len())",
            self.validation,
        )
        self.assertIn("normalized.push('/')", self.validation)
        self.assertIn("normalized.push_str(component)", self.validation)

    def test_normalization_does_not_collect_components(self) -> None:
        self.assertNotIn("let mut normalized = Vec::new()", self.validation)
        self.assertNotIn("normalized.join", self.validation)
        self.assertNotIn("collect::<Vec", self.validation)

    def test_rust_regressions_cover_output_and_rejections(self) -> None:
        self.assertIn("streaming_normalization_preserves_portable_paths", self.source)
        self.assertIn("streaming_normalization_preserves_path_rejections", self.source)


if __name__ == "__main__":
    unittest.main()
