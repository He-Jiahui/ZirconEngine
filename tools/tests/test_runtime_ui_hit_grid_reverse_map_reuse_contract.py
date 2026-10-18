from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/tree/hit_test.rs"


class RuntimeUiHitGridReverseMapReuseContractTests(unittest.TestCase):
    def _reindex_body(self) -> str:
        source = SOURCE.read_text(encoding="utf-8")
        return source.split("fn reindex_entry_cells", 1)[1].split(
            "pub fn hit_test", 1
        )[0]

    def test_reindex_retains_per_entry_cell_capacity(self) -> None:
        body = self._reindex_body()

        self.assertIn("self.entry_cells.retain(|node_id, cells|", body)
        self.assertIn("cells.clear();", body)
        self.assertNotIn("self.entry_cells.clear();", body)

    def test_reindex_only_inserts_missing_entry_reverse_maps(self) -> None:
        body = self._reindex_body()

        self.assertIn("let entry_indices = &self.entry_indices;", body)
        self.assertIn("self.entry_cells.entry(entry.node_id).or_default();", body)
        self.assertNotIn("(entry.node_id, Vec::new())", body)


if __name__ == "__main__":
    unittest.main()
