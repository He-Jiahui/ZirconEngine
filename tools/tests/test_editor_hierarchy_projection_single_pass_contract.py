from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / (
    "zircon_editor/src/ui/retained/ui/data/hierarchy_projection.rs"
)


class EditorHierarchyProjectionSinglePassContractTests(unittest.TestCase):
    def test_required_scene_projection_tracks_selection_without_a_second_scan(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        projection = source.split("fn hierarchy_template_projection", 1)[1].split(
            "fn to_host_contract_scene_node", 1
        )[0]

        self.assertIn("let mut has_selection = false;", projection)
        self.assertIn("has_selection |= node.selected;", projection)
        self.assertIn("hierarchy_nodes", projection)
        self.assertNotIn("payload.nodes.iter().any(|node| node.selected)", projection)

    def test_scene_projection_reuses_the_collected_rows_for_publication(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        projection = source.split("fn hierarchy_template_projection", 1)[1].split(
            "fn to_host_contract_scene_node", 1
        )[0]

        normalized = re.sub(r"\s+", "", projection)
        self.assertEqual(normalized.count("payload.nodes.iter()"), 1)
        self.assertIn("hierarchy_nodes: model_rc(hierarchy_nodes)", projection)
        self.assertNotIn("payload.nodes.iter().map", projection)


if __name__ == "__main__":
    unittest.main()
