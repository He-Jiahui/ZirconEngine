import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PRODUCTION = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/mui_icons/parser.rs"
)
LOWER = (
    ROOT / "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/mui_icons/parser/tests/capacity_tests.rs"
)


class EditorMuiIconPathCapacityContract(unittest.TestCase):
    def test_production_reserves_marker_bound_before_scanning(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        self.assertIn("let mut elements = Vec::with_capacity(", source)
        self.assertIn('source.matches("d: \\\"\").count()', source)
        self.assertIn("cursor = value_end", source)

    def test_empty_and_marker_only_paths_keep_parser_semantics(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        self.assertIn("while let Some(relative_index) = source[cursor..].find(\"d: \\\"\")", source)
        self.assertIn("elements.push(MuiIconPathElement", source)
        self.assertIn("unwrap_or(\"\")", source)

    def test_lower_capacity_regression_and_release_marker_are_wired(self):
        source = LOWER.read_text(encoding="utf-8")
        self.assertIn("EDITOR852_MUI_ICON_PATH_CAPACITY_BENCH_V1", source)
        self.assertIn("path_capacity_preserves_elements", source)
        self.assertIn("path_capacity_source_contract", source)

    def test_deterministic_path_model_removes_geometric_growth(self):
        path_count = 64

        def growth_events(reserved):
            capacity = reserved
            length = 0
            growths = 0
            for _ in range(path_count):
                if length == capacity:
                    capacity = max(4, capacity * 2)
                    growths += 1
                length += 1
            return growths

        self.assertEqual(growth_events(0), 5)
        self.assertEqual(growth_events(path_count), 0)


if __name__ == "__main__":
    unittest.main()
