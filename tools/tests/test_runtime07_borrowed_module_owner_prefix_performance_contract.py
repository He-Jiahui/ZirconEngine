from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/module_validation/names/owner_prefix.rs"
)


class BorrowedModuleOwnerPrefixPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_success_path_borrows_the_existing_name_and_owner(self) -> None:
        compact_source = "".join(self.source.split())
        self.assertIn("module_name.strip_prefix(owner_id)", compact_source)
        self.assertIn(
            ".is_some_and(|suffix| suffix.starts_with('.'))",
            self.source,
        )

    def test_success_path_does_not_allocate_an_owner_prefix(self) -> None:
        self.assertNotIn('let module_prefix = format!("{owner_id}.")', self.source)
        self.assertNotIn("starts_with(&module_prefix)", self.source)

    def test_rust_guard_preserves_owner_boundary_semantics(self) -> None:
        self.assertIn("borrowed_owner_prefix_preserves_boundary_semantics", self.source)
        self.assertIn(
            'assert!(has_module_owner_prefix("weather.runtime", "weather"));',
            self.source,
        )
        self.assertIn(
            'assert!(!has_module_owner_prefix("weather2.runtime", "weather"));',
            self.source,
        )
        self.assertIn(
            'assert!(!has_module_owner_prefix("weather", "weather"));',
            self.source,
        )
        self.assertIn('assert!(has_module_owner_prefix(".runtime", ""));', self.source)


if __name__ == "__main__":
    unittest.main()
