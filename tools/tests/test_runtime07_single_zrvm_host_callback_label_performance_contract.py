from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs"
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


class SingleZrVmHostCallbackLabelPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.validate = function_body(
            cls.source,
            "pub(super) fn validate_native_function_arity(",
        )
        cls.build = function_body(cls.source, "fn build_native_function(")

    def test_valid_arity_path_does_not_materialize_a_label(self) -> None:
        self.assertNotIn("let label = native_function_label", self.validate)

    def test_arity_errors_format_the_label_only_on_failure(self) -> None:
        self.assertGreaterEqual(
            self.validate.count("zr_vm function {module_name}.{}"),
            4,
        )

    def test_callback_setup_does_not_clone_name_or_label_strings(self) -> None:
        self.assertNotIn("function.name.clone()", self.build)
        self.assertNotIn("label.clone()", self.build)

    def test_callback_label_is_built_once_after_arity_validation(self) -> None:
        validation = self.build.index("validate_native_function_arity(module_name, function)")
        callback_label = self.build.index(
            "let callback_label = native_function_label(module_name, &function.name);"
        )
        self.assertLess(validation, callback_label)


if __name__ == "__main__":
    unittest.main()
