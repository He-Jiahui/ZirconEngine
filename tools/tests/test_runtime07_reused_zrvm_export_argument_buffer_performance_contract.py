from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
INSTANCE_SOURCE = (
    ROOT
    / "zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs"
)
OWNER_SOURCE = (
    ROOT
    / "zircon_plugins/zr_vm_language/runtime/src/real_backend/runtime_owner.rs"
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


class ReusedZrVmExportArgumentBufferPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.instance_source = INSTANCE_SOURCE.read_text(encoding="utf-8")
        cls.owner_source = OWNER_SOURCE.read_text(encoding="utf-8")
        cls.call_export = function_body(cls.instance_source, "fn call_export(")

    def test_runtime_owner_retains_only_the_reusable_value_capacity(self) -> None:
        self.assertIn("lowered_arguments: Vec<zrvm::Value>", self.owner_source)
        self.assertIn("fn take_lowered_arguments(", self.owner_source)
        self.assertIn("fn recycle_lowered_arguments(", self.owner_source)

    def test_export_call_does_not_collect_a_fresh_argument_vector(self) -> None:
        self.assertNotIn("collect::<Result<Vec", self.call_export)
        self.assertIn("take_lowered_arguments(&guard)", self.call_export)
        self.assertIn("lower_zr_arguments(&mut lowered_arguments, arguments)", self.call_export)

    def test_lowering_clears_and_reuses_capacity_for_borrowed_arguments(self) -> None:
        self.assertIn("fn lower_zr_arguments(", self.instance_source)
        self.assertIn("lowered_arguments.clear()", self.instance_source)
        self.assertIn("lowered_arguments.reserve(arguments.len())", self.instance_source)
        self.assertIn("to_zr_value(argument)", self.instance_source)

    def test_buffer_is_recycled_before_the_call_result_is_unwrapped(self) -> None:
        for required in (
            "let call_result = Self::call_optional_export(",
            "recycle_lowered_arguments(&guard, lowered_arguments)",
            "let Some(value) = call_result?",
        ):
            self.assertIn(required, self.call_export)
        self.assertLess(
            self.call_export.index("recycle_lowered_arguments"),
            self.call_export.index("let Some(value) = call_result?"),
        )


if __name__ == "__main__":
    unittest.main()
