from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
INSTANCE_SOURCE = (
    ROOT
    / "zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs"
)
VALUES_SOURCE = (
    ROOT
    / "zircon_plugins/zr_vm_language/runtime/src/real_backend/values.rs"
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


def function_header(source: str, signature: str) -> str:
    start = source.index(signature)
    return source[start : source.index("{", start)]


class BorrowedZrVmExportIdentityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.instance_source = INSTANCE_SOURCE.read_text(encoding="utf-8")
        cls.values_source = VALUES_SOURCE.read_text(encoding="utf-8")
        cls.call_export = function_body(cls.instance_source, "fn call_export(")
        cls.wrapper_header = function_header(
            cls.values_source,
            "pub(super) fn from_zr_return_value_for_export(",
        )
        cls.value_header = function_header(cls.values_source, "fn from_zr_value(")

    def test_export_call_does_not_materialize_a_diagnostic_identity(self) -> None:
        self.assertNotIn('format!("{module_name}.{export_name}")', self.call_export)
        self.assertNotIn("let export_label", self.call_export)

    def test_export_call_forwards_borrowed_identity_components(self) -> None:
        self.assertIn(
            "from_zr_return_value_for_export(&value, module_name, export_name)",
            self.call_export,
        )

    def test_return_conversion_borrows_module_and_export_names(self) -> None:
        for header in (self.wrapper_header, self.value_header):
            self.assertIn("module_name: &str", header)
            self.assertIn("export_name: &str", header)
            self.assertNotIn("export_label", header)

    def test_complete_identity_is_formatted_only_in_existing_errors(self) -> None:
        self.assertGreaterEqual(
            self.values_source.count("at export {module_name}.{export_name}"),
            5,
        )


if __name__ == "__main__":
    unittest.main()
