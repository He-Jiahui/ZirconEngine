from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs"


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


class SharedZrVmHostCapabilitySetPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.register = function_body(cls.source, "pub(super) fn register_host_modules(")
        cls.build = function_body(cls.source, "fn build_native_function(")

    def test_registration_clones_host_capabilities_once(self) -> None:
        self.assertIn("let capabilities = Arc::new(host.capabilities.clone());", self.register)
        self.assertEqual(self.register.count("host.capabilities.clone()"), 1)

    def test_native_functions_capture_arc_capability_storage(self) -> None:
        self.assertIn("capabilities: Arc<CapabilitySet>", self.source)
        self.assertIn("Arc::clone(&capabilities)", self.register)
        self.assertIn("use std::sync::Arc;", self.source)

    def test_function_call_borrows_shared_capability_set(self) -> None:
        self.assertIn(".call(", self.build)
        self.assertIn("ScriptHostArguments::new(&argument_source)", self.build)
        self.assertIn("capabilities.as_ref()", self.build)
        self.assertNotIn("host.capabilities.clone()", self.build)

    def test_capability_capture_preserves_callback_boundary(self) -> None:
        self.assertIn("move |context|", self.build)
        self.assertIn("to_zr_value_for_function(value, &callback_label)", self.build)
        self.assertIn("capabilities.as_ref()", self.build)


if __name__ == "__main__":
    unittest.main()
