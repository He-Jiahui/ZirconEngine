from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_report/diagnostic.rs"
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


class StreamingFeatureBlockDiagnosticPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_feature_block_diagnostic_uses_one_exact_output_buffer(self) -> None:
        body = function_body(self.source, "pub fn to_diagnostic(&self)")
        self.assertIn("String::with_capacity(feature_block_diagnostic_capacity", body)
        self.assertIn("write!", body)
        self.assertNotIn("Vec::new()", body)
        self.assertNotIn(".to_string()", body)

    def test_detail_and_list_rows_stream_without_joined_intermediates(self) -> None:
        body = function_body(self.source, "pub fn to_diagnostic(&self)")
        self.assertIn("append_feature_block_detail", body)
        self.assertIn("append_feature_block_list", body)
        self.assertNotIn(".join(", body)
        self.assertNotIn("collect::<Vec", body)

    def test_rust_regression_preserves_complete_diagnostic(self) -> None:
        self.assertIn(
            "streaming_feature_block_diagnostic_preserves_contract",
            self.source,
        )
        self.assertIn(
            "required feature rendering.shadow is blocked: feature is not declared by the plugin catalog; owner dependency is missing",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
