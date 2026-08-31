from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_plugins/zr_vm_language/runtime/src/real_backend/values.rs"


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


class LazyZrVmExportValueLabelPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.wrapper = function_body(
            cls.source,
            "pub(super) fn from_zr_return_value_for_export(",
        )
        cls.value_header = function_header(cls.source, "fn from_zr_value(")
        cls.byte_header = function_header(cls.source, "fn from_zr_byte_array(")

    def test_success_path_does_not_format_an_export_prefix(self) -> None:
        self.assertNotIn('format!("export ', self.wrapper)

    def test_wrapper_passes_the_borrowed_export_label_directly(self) -> None:
        self.assertIn("from_zr_value(value, module_name, export_name)", self.wrapper)
        self.assertIn("module_name: &str", self.value_header)
        self.assertIn("export_name: &str", self.value_header)

    def test_export_prefix_is_materialized_only_inside_error_messages(self) -> None:
        self.assertGreaterEqual(
            self.source.count("at export {module_name}.{export_name}"),
            5,
        )

    def test_byte_array_errors_reuse_the_same_borrowed_label(self) -> None:
        self.assertIn("module_name: &str", self.byte_header)
        self.assertIn("export_name: &str", self.byte_header)
        self.assertNotIn("export_label: &str", self.byte_header)
        self.assertNotIn("value_label: &str", self.byte_header)


if __name__ == "__main__":
    unittest.main()
