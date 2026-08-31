from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/registration_report/validation/interfaces.rs"
)


class PreallocatedRegistrationInterfaceSetsPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_exported_interface_set_uses_iterator_capacity_hint(self) -> None:
        self.assertIn("let exported_interface_rows = extensions.plugin_interfaces();", self.source)
        self.assertIn("exported_interface_rows.size_hint()", self.source)
        self.assertIn("let mut exported_interfaces = HashSet::with_capacity", self.source)

    def test_imported_interface_set_uses_iterator_capacity_hint(self) -> None:
        self.assertIn(
            "let imported_interface_rows = extensions.plugin_interface_imports();",
            self.source,
        )
        self.assertIn("imported_interface_rows.size_hint()", self.source)
        self.assertIn("let mut imported_interfaces = HashSet::with_capacity", self.source)
        self.assertNotIn("collect::<HashSet<_>>()", self.source)

    def test_rust_guard_retains_registration_validation_contract(self) -> None:
        self.assertIn(
            "preallocated_interface_sets_preserve_registration_validation_contract",
            self.source,
        )
        self.assertIn("undeclared_interface_import_is_rejected", self.source)
        self.assertIn("declared_interface_import_must_be_registered", self.source)


if __name__ == "__main__":
    unittest.main()
