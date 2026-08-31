from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/export_build_plan/native_plugin_load_manifest_template.rs"
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


class StreamingNativeLoadManifestPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.template = function_body(
            cls.source,
            "pub(super) fn native_plugin_load_manifest_template(",
        )

    def test_package_rows_stream_into_manifest_output(self) -> None:
        self.assertIn("use std::fmt::Write as _;", self.source)
        self.assertIn("writeln!", self.template)
        self.assertIn("let package_report = format!", self.template)

    def test_package_rows_do_not_allocate_field_format_strings(self) -> None:
        self.assertNotIn("output.push_str(&format!", self.template)
        self.assertEqual(self.template.count("format!("), 1)

    def test_rust_regression_covers_complete_load_manifest(self) -> None:
        self.assertIn("streaming_load_manifest_preserves_toml_contract", self.source)
        self.assertIn("[[plugins]]", self.source)
        self.assertIn("package_report", self.source)


if __name__ == "__main__":
    unittest.main()
