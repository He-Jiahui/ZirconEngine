import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class EditorAssetReferenceProjectionCapacityPerformanceContractTests(unittest.TestCase):
    def test_reference_projections_reserve_known_source_bounds(self):
        source = (
            ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/details.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "Vec::with_capacity(record.direct_references.len())",
            source,
        )
        self.assertIn(
            "Vec::with_capacity(referencer_uuids.len())",
            source,
        )

    def test_reference_projection_keeps_stable_sort_and_unknown_fallback(self):
        source = (
            ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/details.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("direct_references.sort_by(reference_order);", source)
        self.assertIn("referenced_by.sort_by(reference_order);", source)
        self.assertIn("known_project_asset: false", source)
        self.assertIn("display_name_for_locator", source)

    def test_reference_projection_reuses_the_single_referencer_collection(self):
        source = (
            ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/catalog_generation/details.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "let referencer_uuids = runtime_registry.get_referencers_by_uuid(record.asset_uuid);",
            source,
        )
        self.assertIn(
            "for source_uuid in referencer_uuids",
            source.split("let referencer_uuids", 1)[1].split("referenced_by.sort_by", 1)[0],
        )


if __name__ == "__main__":
    unittest.main()
