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


class SharedZrVmExtensionHostCallerPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.register = function_body(cls.source, "pub(super) fn register_extension_host_module(")

    def test_registration_shares_one_arc_caller_across_callbacks(self) -> None:
        self.assertIn("let caller = Arc::new(host.interface_caller()", self.register)
        self.assertGreaterEqual(self.register.count("Arc::clone(&caller)"), 3)

    def test_registration_does_not_deep_clone_interface_caller(self) -> None:
        self.assertNotIn("let system_caller = caller.clone()", self.register)
        self.assertNotIn("let behavior_caller = caller.clone()", self.register)
        self.assertNotIn("let rpc_caller = caller.clone()", self.register)

    def test_callback_calls_still_borrow_authenticated_caller(self) -> None:
        self.assertGreaterEqual(self.register.count("&system_caller"), 1)
        self.assertGreaterEqual(self.register.count("&behavior_caller"), 1)
        self.assertGreaterEqual(self.register.count("&rpc_caller"), 1)
        self.assertGreaterEqual(self.register.count("&caller"), 1)

    def test_arc_import_is_present_for_shared_caller_ownership(self) -> None:
        self.assertIn("use std::sync::Arc;", self.source)


if __name__ == "__main__":
    unittest.main()
