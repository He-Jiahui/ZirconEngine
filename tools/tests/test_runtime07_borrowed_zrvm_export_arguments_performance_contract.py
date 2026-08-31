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


class BorrowedZrVmExportArgumentsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.instance_source = INSTANCE_SOURCE.read_text(encoding="utf-8")
        cls.values_source = VALUES_SOURCE.read_text(encoding="utf-8")
        cls.call_export = function_body(cls.instance_source, "fn call_export(")
        cls.lower_arguments = function_body(
            cls.instance_source,
            "pub(super) fn lower_zr_arguments(",
        )
        cls.lower_arguments_header = function_header(
            cls.instance_source,
            "pub(super) fn lower_zr_arguments(",
        )
        cls.lower_value_header = function_header(
            cls.values_source,
            "pub(super) fn to_zr_value(",
        )
        cls.return_lowering = function_body(
            cls.values_source,
            "pub(super) fn to_zr_value_for_function(",
        )

    def test_export_call_does_not_clone_owned_host_arguments(self) -> None:
        self.assertNotIn(".cloned()", self.call_export)
        self.assertNotIn("arguments.to_vec()", self.call_export)

    def test_export_call_lowers_each_borrowed_argument_directly(self) -> None:
        self.assertIn(
            "lower_zr_arguments(&mut lowered_arguments, arguments)",
            self.call_export,
        )
        self.assertIn("arguments: &[ScriptHostValue]", self.lower_arguments_header)
        self.assertIn("for argument in arguments", self.lower_arguments)
        self.assertIn("to_zr_value(argument)", self.lower_arguments)

    def test_value_lowering_accepts_a_borrowed_host_value(self) -> None:
        self.assertIn("value: &ScriptHostValue", self.lower_value_header)

    def test_owned_host_returns_reuse_the_borrowed_lowering_helper(self) -> None:
        self.assertIn("to_zr_value(&value)", self.return_lowering)


if __name__ == "__main__":
    unittest.main()
