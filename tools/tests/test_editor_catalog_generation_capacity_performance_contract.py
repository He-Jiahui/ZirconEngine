import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class EditorCatalogGenerationCapacityPerformanceContractTests(unittest.TestCase):
    def test_catalog_projection_reserves_each_record_backed_output(self):
        source = (
            ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/build.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("let record_count = records.len();", source)
        self.assertIn("Vec::with_capacity(record_count)", source)
        self.assertGreaterEqual(source.count("Vec::with_capacity(record_count)"), 3)

    def test_catalog_projection_keeps_sorted_records_and_details_contract(self):
        source = (
            ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/build.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "records.sort_by(|left, right| left.locator.cmp(&right.locator));",
            source,
        )
        self.assertIn("build_details_generation(", source)
        self.assertIn("Arc::clone(", source)
        self.assertIn("Arc::new((*record).clone())", source)

    def test_catalog_projection_does_not_collect_record_backed_vectors_without_capacity(self):
        source = (
            ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/build.rs"
        ).read_text(encoding="utf-8")
        implementation = source.split("#[cfg(test)]", 1)[0]
        self.assertNotIn(".collect::<Vec<_>>()", implementation)


if __name__ == "__main__":
    unittest.main()
