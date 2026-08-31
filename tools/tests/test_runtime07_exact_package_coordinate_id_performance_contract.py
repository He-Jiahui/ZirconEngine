from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs"
)


class ExactPackageCoordinateIdPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.package_id_body = cls.source.split(
            "pub fn package_id(&self) -> String {", maxsplit=1
        )[1].split("pub fn asset_roots_or_default", maxsplit=1)[0]

    def test_coordinate_id_preallocates_all_segments_and_separators(self) -> None:
        self.assertIn("self.package_prefix.len()", self.package_id_body)
        self.assertIn("self.package_company.len()", self.package_id_body)
        self.assertIn("self.package_name.len()", self.package_id_body)
        self.assertIn("+ 2", self.package_id_body)
        self.assertIn("String::with_capacity(capacity)", self.package_id_body)

    def test_coordinate_id_appends_borrowed_segments_without_formatter(self) -> None:
        self.assertIn("package_id.push_str(&self.package_prefix)", self.package_id_body)
        self.assertIn("package_id.push('.')", self.package_id_body)
        self.assertIn("package_id.push_str(&self.package_company)", self.package_id_body)
        self.assertIn("package_id.push_str(&self.package_name)", self.package_id_body)
        self.assertNotIn("format!(", self.package_id_body)

    def test_rust_guard_preserves_qualified_and_fallback_ids(self) -> None:
        self.assertIn(
            "exact_package_coordinate_id_preserves_qualified_and_fallback_identity",
            self.source,
        )
        self.assertIn('"org.zircon.weather"', self.source)
        self.assertIn('"legacy_weather"', self.source)
        self.assertIn("return self.id.clone()", self.package_id_body)


if __name__ == "__main__":
    unittest.main()
