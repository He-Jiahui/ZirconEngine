from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
REGISTER = ROOT / "zircon_runtime/src/plugin/extension_registry/register.rs"
RUNTIME_CORE = (
    ROOT
    / "zircon_runtime/src/plugin/extension_registry/register/runtime_core.rs"
)


class ExactRuntimeOwnerKeyPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.register = REGISTER.read_text(encoding="utf-8")
        cls.runtime_core = RUNTIME_CORE.read_text(encoding="utf-8")

    def test_runtime_owner_key_preallocates_the_exact_output_length(self) -> None:
        self.assertIn("fn runtime_owner_key(plugin_id: &str) -> String", self.register)
        self.assertIn("plugin_id.len() + \".runtime\".len()", self.register)
        self.assertIn("String::with_capacity(capacity)", self.register)
        self.assertNotIn('format!("{plugin_id}.runtime")', self.register)

    def test_owner_and_module_registration_share_the_runtime_owner_path(self) -> None:
        self.assertIn(
            "self.intern_plugin_module(runtime_owner_key(plugin_id))",
            self.register,
        )
        self.assertIn(
            "let owner = self.intern_runtime_owner(&descriptor.name)?;",
            self.runtime_core,
        )
        self.assertNotIn('format!("{}.runtime", descriptor.name)', self.runtime_core)

    def test_rust_guard_preserves_runtime_owner_identity(self) -> None:
        self.assertIn("exact_runtime_owner_key_preserves_identity", self.register)
        self.assertIn('runtime_owner_key("rendering")', self.register)
        self.assertIn('"rendering.runtime"', self.register)


if __name__ == "__main__":
    unittest.main()
