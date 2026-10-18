import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class EditorCatalogProjectionInputCapacityPerformanceContractTests(unittest.TestCase):
    def test_full_catalog_projection_reserves_runtime_asset_capacity(self):
        source = (
            ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "let catalog_capacity = candidate_index.len();",
            source,
        )
        self.assertIn(
            "HashMap::with_capacity(catalog_capacity)",
            source,
        )
        self.assertGreaterEqual(
            source.count("HashMap::with_capacity(catalog_capacity)"),
            2,
        )

    def test_record_projection_reserves_diagnostic_entries(self):
        source = (
            ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/record_projection.rs"
        ).read_text(encoding="utf-8")
        compact = "".join(source.split())
        self.assertIn("Vec::with_capacity(", compact)
        self.assertIn("metadata.diagnostics.len()", compact)
        self.assertIn(
            "saturating_add(catalog_input.reference_repairs().len())",
            compact,
        )

    def test_catalog_projection_capacity_keeps_single_pass_and_projection_semantics(self):
        source = (
            ROOT
            / "zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs"
        ).read_text(encoding="utf-8")
        implementation = source.split("#[cfg(test)]", 1)[0]
        self.assertIn("project_full_catalog(", implementation)
        self.assertIn("catalog_by_uuid.insert(record.asset_uuid, record);", implementation)
        self.assertIn("uuid_by_locator.insert(record.locator.clone(), record.asset_uuid);", implementation)
        self.assertNotIn("catalog_input_generation.records().count()", implementation)


if __name__ == "__main__":
    unittest.main()
