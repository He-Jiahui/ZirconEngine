from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs"
)


class ExactInterfaceImportKeyPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.production = cls.source.split("#[cfg(test)]", maxsplit=1)[0]

    def test_import_key_preallocates_the_exact_output_length(self) -> None:
        self.assertIn(
            "fn interface_import_key(module_name: &str, interface_id: &str) -> String",
            self.source,
        )
        self.assertIn(
            "module_name.len() + 2 + interface_id.len()",
            self.source,
        )
        self.assertIn("String::with_capacity(capacity)", self.source)

    def test_registration_uses_the_exact_key_builder(self) -> None:
        self.assertIn(
            "let key = interface_import_key(module_name, import.interface_id());",
            self.source,
        )
        self.assertNotIn('format!("{module_name}=>{}", import.interface_id())', self.production)

    def test_rust_guard_preserves_key_delimiter_and_order(self) -> None:
        self.assertIn("exact_interface_import_key_preserves_identity", self.source)
        self.assertIn(
            'interface_import_key("weather.runtime", "zr.weather.v1")',
            self.source,
        )
        self.assertIn('"weather.runtime=>zr.weather.v1"', self.source)


if __name__ == "__main__":
    unittest.main()
