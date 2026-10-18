import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class EditorCatalogDetailsCowCapacityPerformanceContractTests(unittest.TestCase):
    def setUp(self):
        self.source = (
            ROOT / "zircon_editor/src/ui/host/editor_asset_manager/generation.rs"
        ).read_text(encoding="utf-8")
        self.implementation = self.source.split("#[cfg(test)]", 1)[0]

    def test_single_row_updates_share_details_slice_when_slot_is_unmaterialized(self):
        self.assertIn(
            "Arc::clone(&self.details_by_asset_index)",
            self.implementation,
        )
        self.assertIn(
            "let details = if self.details_by_asset_index[index].is_some()",
            self.implementation,
        )

    def test_materialized_details_still_copy_and_replace_only_the_target_slot(self):
        compact = "".join(self.implementation.split())
        self.assertIn(
            "letmutdetails=self.details_by_asset_index.iter().cloned().collect::<Vec<_>>();",
            compact,
        )
        self.assertIn(
            "details[index]=Some(Arc::new(current_details.with_asset(",
            compact,
        )

    def test_catalog_update_keeps_asset_and_catalog_record_replacement_semantics(self):
        self.assertIn("assets[index] = Arc::clone(&updated);", self.implementation)
        self.assertIn("catalog_records[index] = Some(Arc::new(updated));", self.implementation)
        self.assertIn("details_by_asset_index: details", self.implementation)

    def test_batched_updates_copy_details_only_when_a_target_is_materialized(self):
        compact = "".join(self.implementation.split())
        self.assertIn(
            ".any(|(index,_)|self.details_by_asset_index[*index].is_some())",
            compact,
        )
        self.assertIn("letdetails_by_asset_index=details.map(Into::into)", compact)


if __name__ == "__main__":
    unittest.main()
