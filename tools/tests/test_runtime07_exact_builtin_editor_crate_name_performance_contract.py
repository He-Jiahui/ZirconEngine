from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/plugin/package_manifest/builtin_catalog.rs"


class ExactBuiltinEditorCrateNamePerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.production = cls.source.split("#[cfg(test)]", maxsplit=1)[0]

    def test_crate_name_preallocates_all_borrowed_parts(self) -> None:
        self.assertIn(
            '"zircon_plugin_".len() + package_id.len() + "_editor".len()',
            self.source,
        )
        self.assertIn("String::with_capacity(capacity)", self.source)
        self.assertIn('crate_name.push_str("zircon_plugin_")', self.source)
        self.assertIn("crate_name.push_str(package_id)", self.source)
        self.assertIn('crate_name.push_str("_editor")', self.source)

    def test_catalog_map_uses_exact_helper_without_formatter_growth(self) -> None:
        self.assertIn(
            "exact_builtin_editor_crate_name(descriptor.package_id())",
            self.source,
        )
        self.assertNotIn("format!(", self.production)

    def test_rust_guard_preserves_editor_crate_names(self) -> None:
        self.assertIn(
            "exact_builtin_editor_crate_names_preserve_package_identity",
            self.source,
        )
        self.assertIn('"zircon_plugin_net_editor"', self.source)
        self.assertIn(
            '"zircon_plugin_rendering_deferred_editor"',
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
