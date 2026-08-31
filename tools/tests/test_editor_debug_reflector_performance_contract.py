from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class EditorDebugReflectorPerformanceContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_selected_node_validation_does_not_build_a_full_node_index(self) -> None:
        model = self.read("zircon_editor/src/ui/workbench/debug_reflector/model.rs")

        self.assertNotIn("BTreeSet", model)
        self.assertIn(".any(|node| node.node_id == selected_node)", model)

    def test_payload_projection_borrows_existing_reflector_text_rows(self) -> None:
        projection = self.read(
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
            "runtime_diagnostics.rs"
        )
        function = projection.split("fn runtime_debug_reflector_nodes(", 1)[1].split(
            "fn runtime_debug_reflector_nodes_from_model(", 1
        )[0]

        self.assertNotIn(".map(ToString::to_string)", function)
        for field in (
            "&payload.ui_debug_reflector_details",
            "&payload.ui_debug_reflector_sections",
            "&payload.ui_debug_reflector_nodes",
        ):
            self.assertIn(field, function)


if __name__ == "__main__":
    unittest.main()
