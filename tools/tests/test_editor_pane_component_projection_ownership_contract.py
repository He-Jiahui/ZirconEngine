from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
CONVERSION = (
    ROOT / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion"
)


class EditorPaneComponentProjectionOwnershipContractTests(unittest.TestCase):
    def test_owned_host_node_moves_component_into_projected_parts(self) -> None:
        source = (
            CONVERSION / "pane_component_projection/host_template_node.rs"
        ).read_text(encoding="utf-8")
        function = source.split("fn host_template_node", 1)[1]

        self.assertIn("let component = node.component;", function)
        self.assertNotIn("node.component.clone()", function)

    def test_content_fallback_matches_converted_control_id_without_clone(self) -> None:
        source = (CONVERSION / "template_runtime_projection.rs").read_text(
            encoding="utf-8"
        )
        function = source.split(
            "fn host_template_node_with_content_fallback", 1
        )[1].split("pub(super) fn builtin_host_runtime", 1)[0]

        self.assertNotIn("control_id.clone()", function)
        self.assertIn("node.control_id.as_str()", function)


if __name__ == "__main__":
    unittest.main()
