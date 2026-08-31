from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs"
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


class BorrowedZrVmLifecycleEntryModulePerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.optional_header = function_header(cls.source, "fn call_optional_export(")
        cls.lifecycle = function_body(cls.source, "fn call_entry_lifecycle_export(")
        cls.call_export = function_body(cls.source, "fn call_export(")

    def test_lifecycle_call_does_not_clone_the_entry_module(self) -> None:
        self.assertNotIn("self.entry_module.clone()", self.lifecycle)

    def test_optional_export_helper_borrows_only_the_runtime_owner(self) -> None:
        self.assertIn("runtime_owner: &mut ZrVmRuntimeOwner", self.optional_header)
        self.assertNotIn("&mut self", self.optional_header)

    def test_lifecycle_call_borrows_disjoint_instance_fields(self) -> None:
        self.assertIn("&mut self.runtime_owner", self.lifecycle)
        self.assertIn("&self.entry_module", self.lifecycle)

    def test_general_export_uses_the_same_field_scoped_helper(self) -> None:
        self.assertIn("&mut self.runtime_owner", self.call_export)
        self.assertNotIn("self.call_optional_export", self.call_export)


if __name__ == "__main__":
    unittest.main()
