from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/plugin/export_build_plan/cargo_manifest_template.rs"


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


class StreamingCargoManifestPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.template = function_body(
            cls.source,
            "pub(super) fn cargo_manifest_template(",
        )

    def test_linked_crate_rows_stream_into_manifest_output(self) -> None:
        self.assertIn("use std::fmt::Write as _;", self.source)
        self.assertIn("writeln!", self.template)
        self.assertNotIn("contents.push_str(&format!", self.template)

    def test_package_name_does_not_build_a_full_temporary_string(self) -> None:
        self.assertNotIn("sanitize_package_name(&format!", self.template)
        self.assertIn("String::with_capacity", self.source)

    def test_rust_regression_covers_complete_cargo_manifest(self) -> None:
        self.assertIn("streaming_cargo_manifest_preserves_contract", self.source)
        self.assertIn("[dependencies]", self.source)
        self.assertIn("zircon_plugin_test_runtime", self.source)


if __name__ == "__main__":
    unittest.main()
