from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TABLE = ROOT / "zircon_runtime/src/ui/surface/render/collection_rows/table.rs"


class RuntimeCollectionTableMaterializationContractTests(unittest.TestCase):
    def test_table_cells_use_a_fixed_four_slot_authority(self) -> None:
        source = TABLE.read_text(encoding="utf-8")
        start = source.index("fn table_cells(")
        end = source.index("\nfn cell_rect(", start)
        cells = source[start:end]

        self.assertIn("type TableCells = [Option<String>; COLUMN_RATIOS.len()]", source)
        self.assertIn("std::array::from_fn", cells)
        self.assertNotIn("collect::<Vec<_>>()", cells)
        self.assertNotIn("-> Vec<String>", cells)

    def test_table_row_does_not_materialize_columns_beyond_render_capacity(self) -> None:
        source = TABLE.read_text(encoding="utf-8")
        start = source.index("pub(super) fn table_row_commands(")
        end = source.index("\nfn background(", start)
        commands = source[start:end]

        self.assertIn("for (index, cell) in cells.into_iter().enumerate()", commands)
        self.assertNotIn(".take(COLUMN_RATIOS.len())", commands)


if __name__ == "__main__":
    unittest.main()
