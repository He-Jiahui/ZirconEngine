from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_plugins/zr_vm_language/runtime/src/backend.rs"


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


class SharedZrVmBackendPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.resolve = function_body(cls.source, "fn resolve(")

    def test_zr_vm_backend_is_a_shared_lazy_arc(self) -> None:
        self.assertIn(
            "static ZR_VM_BACKEND: LazyLock<Arc<dyn VmBackend>>",
            self.source,
        )

    def test_resolve_clones_shared_arc_without_backend_allocation(self) -> None:
        self.assertIn("Arc::clone(&ZR_VM_BACKEND)", self.resolve)
        self.assertNotIn("Arc::new(ZrVmBackend)", self.resolve)

    def test_rust_regression_checks_canonical_and_alias_pointer_identity(self) -> None:
        self.assertIn("zr_vm_backend_resolutions_share_arc_storage", self.source)
        self.assertIn("Arc::ptr_eq", self.source)
        self.assertIn('"zr_vm:project"', self.source)
        self.assertIn('"project"', self.source)


if __name__ == "__main__":
    unittest.main()
