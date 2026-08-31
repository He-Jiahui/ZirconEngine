from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/script/vm/host/builtin_host_modules.rs"


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


class PreallocatedBuiltinHostModuleHandlesPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.register = function_body(cls.source, "pub fn register_builtin_host_modules(")

    def test_complete_install_capacity_matches_all_builtin_modules(self) -> None:
        self.assertIn("const BUILTIN_HOST_MODULE_HANDLE_CAPACITY: usize = 6;", self.source)
        self.assertEqual(self.register.count("handles.push("), 6)

    def test_handle_collector_uses_exact_preallocation(self) -> None:
        self.assertIn(
            "Vec::with_capacity(BUILTIN_HOST_MODULE_HANDLE_CAPACITY)",
            self.register,
        )
        self.assertNotIn("Vec::new()", self.register)

    def test_rust_regression_locks_complete_install_capacity(self) -> None:
        self.assertIn("builtin_host_module_handle_capacity_covers_complete_install", self.source)
        self.assertIn("BUILTIN_HOST_MODULE_HANDLE_CAPACITY", self.source)


if __name__ == "__main__":
    unittest.main()
