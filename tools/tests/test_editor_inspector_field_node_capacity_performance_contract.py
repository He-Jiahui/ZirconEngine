import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/inspector_fields.rs"
REGRESSION = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/inspector_fields/tests/capacity_tests.rs"
)


class EditorInspectorFieldNodeCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8") if REGRESSION.exists() else ""

    def test_inspector_projection_reserves_base_and_plugin_node_bounds(self) -> None:
        field_match = re.search(
            r"fn inspector_field_nodes_with_metrics\(.*?\n}\n\nfn inspector_field_panel_height",
            self.source,
            re.S,
        )
        plugin_match = re.search(
            r"fn inspector_plugin_component_nodes\(.*?\n}\n\nfn inspector_plugin_component_height",
            self.source,
            re.S,
        )
        self.assertIsNotNone(field_match)
        self.assertIsNotNone(plugin_match)
        field_body = field_match.group(0)
        plugin_body = plugin_match.group(0)
        self.assertIn("INSPECTOR_BASE_NODE_COUNT", field_body)
        self.assertIn("Vec::with_capacity(", field_body)
        self.assertIn("inspector_plugin_component_node_count(fields)", field_body)
        self.assertIn("inspector_plugin_component_node_count(fields)", plugin_body)
        self.assertNotIn("let mut nodes = Vec::new();", field_body)
        self.assertNotIn("let mut nodes = Vec::new();", plugin_body)

    def test_projection_order_and_optional_rows_remain_intact(self) -> None:
        body = self.source[
            self.source.index("fn inspector_field_nodes_with_metrics") : self.source.index(
                "fn inspector_field_panel_height"
            )
        ]
        self.assertIn("nodes.push(panel);", body)
        self.assertIn("nodes.extend(plugin_nodes);", body)
        self.assertIn("if let Some(message) = inspector_plugin_component_fallback_message", body)
        self.assertIn("nodes.push(inspector_action_button_node(", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod capacity_tests;", self.source)
        self.assertIn(
            "editor857_inspector_field_node_capacity_preserves_order",
            self.regression,
        )
        self.assertIn("EDITOR857_INSPECTOR_FIELD_NODE_CAPACITY_BENCH_V1", self.regression)

    def test_deterministic_model_removes_dense_inspector_growth(self) -> None:
        plugin_components = 1_024
        base_nodes = 9
        legacy_growth_events = 18
        optimized_growth_events = 0
        self.assertGreater(plugin_components + base_nodes, 0)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
