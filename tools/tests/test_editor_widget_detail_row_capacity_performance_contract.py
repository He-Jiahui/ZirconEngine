import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PRODUCTION = (
    ROOT
    / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/ui_asset_detail_fields/widget.rs"
)
LOWER = (
    ROOT / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/ui_asset_detail_fields/widget/tests/capacity_tests.rs"
)


class EditorWidgetDetailRowCapacityContract(unittest.TestCase):
    def test_production_reserves_known_widget_detail_row_bound(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        self.assertIn("WIDGET_DETAIL_ROW_MAX_COUNT", source)
        self.assertIn(
            "Vec::with_capacity(widget_detail_row_capacity(data, prop_state_rows))",
            source,
        )
        self.assertIn("fn widget_detail_row_capacity", source)
        self.assertIn("fn prop_state_row_is_actionable", source)

    def test_capacity_helper_accounts_for_fixed_and_bounded_dynamic_rows(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        self.assertIn("PROP_STATE_ROW_LIMIT", source)
        self.assertIn(".take(PROP_STATE_ROW_LIMIT)", source)
        self.assertIn(".filter(|row| prop_state_row_is_actionable(row))", source)
        self.assertIn("inspector_can_edit_control_id", source)
        self.assertIn("inspector_can_edit_text_prop", source)
        self.assertIn("inspector_can_edit_component_root_class_policy", source)

    def test_lower_capacity_regression_and_release_marker_are_wired(self):
        source = LOWER.read_text(encoding="utf-8")
        self.assertIn("EDITOR850_WIDGET_DETAIL_ROW_CAPACITY_BENCH_V1", source)
        self.assertIn("capacity_preserves_widget_detail_rows", source)
        self.assertIn("widget_capacity_counts_only_emitted_rows", source)

    def test_deterministic_widget_detail_model_removes_geometric_growth(self):
        row_count = 9

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

        self.assertEqual(growth_events(0), 3)
        self.assertEqual(growth_events(row_count), 0)


if __name__ == "__main__":
    unittest.main()
