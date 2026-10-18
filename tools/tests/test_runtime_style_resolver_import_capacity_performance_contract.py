import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PRODUCTION = (
    ROOT
    / "zircon_runtime/src/ui/template/asset/compiler/ui_style_resolver.rs"
)
LOWER = (
    ROOT / "zircon_runtime/src/ui/template/asset/compiler/ui_style_resolver/tests/import_capacity_tests.rs"
)


class RuntimeStyleResolverImportCapacityContract(unittest.TestCase):
    def test_production_reserves_imported_style_resolution_bound(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        self.assertIn(
            "let mut imported_styles = Vec::with_capacity(document.imports.styles.len());",
            source,
        )
        self.assertIn("for reference in &document.imports.styles", source)
        self.assertIn("imported_styles.push(", source)
        self.assertNotIn("collect::<Result<Vec<_>, _>>()?", source)

    def test_resolution_keeps_single_lookup_and_error_order(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        start = source.index("let mut imported_styles")
        end = source.index("let imported_stylesheet_count", start)
        resolution = source[start:end]
        self.assertEqual(resolution.count(".get(reference)"), 1)
        self.assertIn("ok_or_else(||", resolution)
        self.assertIn("UiAssetError::UnknownImport", resolution)

    def test_lower_capacity_regression_and_release_marker_are_wired(self):
        source = LOWER.read_text(encoding="utf-8")
        self.assertIn("RUNTIME851_STYLE_IMPORT_RESOLUTION_CAPACITY_BENCH_V1", source)
        self.assertIn("import_capacity_source_contract", source)
        self.assertIn("import_capacity_preserves_resolution_shape", source)

    def test_deterministic_import_model_removes_geometric_growth(self):
        import_count = 64

        def growth_events(reserved):
            capacity = reserved
            length = 0
            growths = 0
            for _ in range(import_count):
                if length == capacity:
                    capacity = max(4, capacity * 2)
                    growths += 1
                length += 1
            return growths

        self.assertEqual(growth_events(0), 5)
        self.assertEqual(growth_events(import_count), 0)


if __name__ == "__main__":
    unittest.main()
