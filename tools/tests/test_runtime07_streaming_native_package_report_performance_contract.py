from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/plugin/export_build_plan/native_dynamic_package_plan.rs"


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


class StreamingNativePackageReportPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.report = function_body(cls.source, "pub(super) fn native_dynamic_package_report_template(")
        cls.abi = function_body(cls.source, "pub(super) fn append_native_dynamic_abi_contract_toml(")

    def test_report_uses_string_format_writer(self) -> None:
        self.assertIn("use std::fmt::Write as _;", self.source)
        self.assertIn("writeln!", self.report)
        self.assertIn("writeln!", self.abi)

    def test_report_does_not_allocate_intermediate_format_strings(self) -> None:
        for body in (self.report, self.abi):
            self.assertNotIn("push_str(&format!", body)
            self.assertNotIn("push_str(\n        &format!", body)

    def test_rust_regression_covers_complete_toml_contract(self) -> None:
        self.assertIn("streaming_report_preserves_toml_contract", self.source)
        self.assertIn("format_version = 1", self.source)
        self.assertIn("bridge_method_table", self.source)


if __name__ == "__main__":
    unittest.main()
