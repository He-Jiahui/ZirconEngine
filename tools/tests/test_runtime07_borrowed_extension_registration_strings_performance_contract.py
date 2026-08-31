from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_plugins/zr_vm_language/runtime/src/real_backend/extension_host.rs"


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


class BorrowedExtensionRegistrationStringsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.register = function_body(cls.source, "pub(super) fn register_extension_host_module(")
        cls.string_function = function_body(cls.source, "fn string_function(")
        cls.reader = function_body(
            cls.source,
            "fn with_extension_registration_strings(",
        )

    def test_callbacks_receive_the_borrowed_argument_source(self) -> None:
        self.assertIn(
            "callback: impl Fn(&ScriptHostArguments<'_>, &str) -> Result<zrvm::Value, zrvm::Error>",
            self.source,
        )
        self.assertIn('let label = format!("{VM_HOST_INTERFACE_MODULE}.{name}");', self.string_function)
        self.assertEqual(self.register.count("move |arguments, label|"), 4)
        self.assertNotIn("Fn(Vec<String>)", self.source)
        self.assertNotIn("Fn(&[&str])", self.source)

    def test_registration_uses_borrowed_arguments_directly(self) -> None:
        self.assertIn("with_extension_registration_strings(context, &label, &callback)", self.string_function)
        self.assertNotIn("read_extension_registration_strings_at_business_boundary", self.source)
        self.assertNotIn("let arguments: Vec<String>", self.register)

    def test_reader_keeps_argument_borrows_inside_the_callback(self) -> None:
        self.assertIn("let host_arguments = ScriptHostArguments::new(&source);", self.reader)
        self.assertIn("callback(&host_arguments, label)", self.reader)
        self.assertIn("fn borrow_string<T>(", self.source)
        self.assertNotIn("registered_arguments", self.reader)
        self.assertNotIn("Vec<String>", self.reader)
        self.assertNotIn("Vec::with_capacity", self.reader)
        self.assertIn("visitor(value).map_err(|error| ScriptHostError::new(error.message))", self.source)

    def test_borrow_string_preserves_validation_without_copy_metrics(self) -> None:
        self.assertIn("ScriptHostValueRef::String(value)", self.source)
        self.assertNotIn("ScriptHostHotPathMetrics::record_guest_string_copy", self.reader)
        self.assertIn("value.kind()", self.source)
        self.assertIn("must be a string, received", self.source)
        self.assertNotIn('"zircon.vm.register_', self.source)


if __name__ == "__main__":
    unittest.main()
