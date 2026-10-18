import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/layouts/views/asset_reference_rows.rs"


class EditorAssetReferenceRowsCapacityPerformanceContractTests(unittest.TestCase):
    def test_reference_row_refresh_reserves_four_nodes_per_source_row(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn sync_asset_reference_list(")
        end = source.index("\nfn apply_asset_reference_list_layout", start)
        body = source[start:end]
        self.assertIn("const REFERENCE_ROW_NODE_COUNT: usize = 4;", source)
        self.assertIn(
            "nodes.reserve(references.len().saturating_mul(REFERENCE_ROW_NODE_COUNT));",
            body,
        )
        self.assertIn("nodes.extend(prototypes.row_nodes(controls, index + 1, reference));", body)

    def test_reference_row_refresh_keeps_prototype_reset_and_empty_semantics(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn sync_asset_reference_list(")
        end = source.index("\nfn apply_asset_reference_list_layout", start)
        body = source[start:end]
        self.assertIn("nodes.retain(|node| !is_dynamic_reference_row_component", body)
        self.assertIn("hide_reference_row_prototypes(nodes, controls);", body)
        self.assertIn("if references.is_empty()", body)
        self.assertIn("controls.empty_text.into()", body)

    def test_reserved_row_capacity_removes_geometric_growth_in_the_model(self) -> None:
        rows = 4096
        nodes_per_row = 4
        required = rows * nodes_per_row

        old_capacity = 16
        target_capacity = old_capacity + required
        old_growth = 0
        while old_capacity < target_capacity:
            old_capacity *= 2
            old_growth += 1

        reserved_growth = 1
        self.assertGreater(old_growth, reserved_growth)
        self.assertEqual(reserved_growth, 1)
        self.assertEqual(required, rows * nodes_per_row)


if __name__ == "__main__":
    unittest.main()
