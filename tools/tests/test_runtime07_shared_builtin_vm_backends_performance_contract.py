from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT / "zircon_runtime/src/script/vm/backend/builtin_vm_backend_family.rs"
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


class SharedBuiltinVmBackendsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.resolve = function_body(cls.source, "fn resolve(")

    def test_builtin_backends_are_shared_lazy_arc_instances(self) -> None:
        self.assertIn("static MOCK_BACKEND: LazyLock<Arc<dyn VmBackend>>", self.source)
        self.assertIn(
            "static UNAVAILABLE_BACKEND: LazyLock<Arc<dyn VmBackend>>",
            self.source,
        )

    def test_resolve_clones_shared_arcs_without_allocating_backends(self) -> None:
        self.assertIn("Arc::clone(&MOCK_BACKEND)", self.resolve)
        self.assertIn("Arc::clone(&UNAVAILABLE_BACKEND)", self.resolve)
        self.assertNotIn("Arc::new(MockVmBackend)", self.resolve)
        self.assertNotIn("Arc::new(UnavailableVmBackend)", self.resolve)

    def test_rust_regression_checks_canonical_and_alias_pointer_identity(self) -> None:
        self.assertIn("builtin_backend_resolutions_share_arc_storage", self.source)
        self.assertIn("Arc::ptr_eq", self.source)
        self.assertIn('"builtin:mock"', self.source)
        self.assertIn('"unavailable"', self.source)


if __name__ == "__main__":
    unittest.main()
