from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMMANDS = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_table_rows/commands.rs"
)
CELL_TEXT = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_table_rows/cells/text.rs"
)


class EditorTemplateTablePaintMaterializationContractTests(unittest.TestCase):
    def test_table_rows_reject_empty_or_clipped_geometry_before_cell_materialization(self) -> None:
        source = COMMANDS.read_text(encoding="utf-8")
        workbench_start = source.index("fn push_table_row_commands")
        text_start = source.index("fn push_table_row_text_commands")
        workbench = source[workbench_start:text_start]
        table_text = source[text_start:]

        self.assertLess(
            workbench.index("let Some(clip) = intersect(&rect, clip)"),
            workbench.index("let cells = table_cells(node)"),
        )
        self.assertLess(
            table_text.index("intersect(rect, clip).is_none()"),
            table_text.index("let cells = table_cells(node)"),
        )

    def test_declared_cell_shape_probe_does_not_materialize_archived_cells(self) -> None:
        source = CELL_TEXT.read_text(encoding="utf-8")
        predicate_start = source.index("fn option_cells_look_like_declared_cells")
        split_start = source.index("pub(in ", predicate_start)
        predicate = source[predicate_start:split_start]

        self.assertNotIn("split_archived_table_text", predicate)
        self.assertIn("split_whitespace().nth(3).is_some()", predicate)


if __name__ == "__main__":
    unittest.main()
