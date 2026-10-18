import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PRODUCTION = ROOT / "zircon_editor/src/ui/material_editor/projection.rs"
LOWER = ROOT / "zircon_editor/src/ui/material_editor/projection/tests/row_capacity_tests.rs"


class EditorMaterialProjectionRowCapacityContract(unittest.TestCase):
    def _production_section(self, name: str, next_name: str) -> str:
        source = PRODUCTION.read_text(encoding="utf-8")
        start = source.index(f"fn {name}")
        end = source.index(f"fn {next_name}", start)
        return source[start:end]

    def test_property_rows_reserve_schema_and_override_bound(self):
        section = self._production_section(
            "project_property_rows", "project_texture_slot_rows"
        )
        self.assertIn("let mut rows = Vec::with_capacity(", section)
        self.assertIn("shader.property_schema.len()", section)
        self.assertIn("material.property_overrides().len()", section)
        self.assertNotIn("let mut rows = Vec::new();", section)
        self.assertIn("rows.push(MaterialEditorPropertyRow", section)

    def test_texture_rows_reserve_schema_and_override_bound(self):
        section = self._production_section(
            "project_texture_slot_rows", "project_diagnostic_rows"
        )
        self.assertIn("let mut rows = Vec::with_capacity(", section)
        self.assertIn("shader.texture_slots.len()", section)
        self.assertIn("material.texture_slots.len()", section)
        self.assertNotIn("let mut rows = Vec::new();", section)
        self.assertIn("rows.push(MaterialEditorTextureSlotRow", section)

    def test_lower_capacity_regression_and_release_marker_are_wired(self):
        source = LOWER.read_text(encoding="utf-8")
        self.assertIn("EDITOR856_MATERIAL_PROJECTION_ROW_CAPACITY_BENCH_V1", source)
        self.assertIn("property_and_texture_row_capacity_preserves_order", source)
        self.assertIn("row_capacity_source_contract", source)

    def test_deterministic_row_model_removes_geometric_growth(self):
        row_count = 8_192

        def growth_events(reserved):
            capacity = reserved
            length = 0
            growths = 0
            for _ in range(row_count):
                if length == capacity:
                    capacity = max(4, capacity * 2)
                    growths += 1
                length += 1
            return growths

        self.assertEqual(growth_events(0), 12)
        self.assertEqual(growth_events(row_count), 0)


if __name__ == "__main__":
    unittest.main()
