from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class EditorUiHitIndexCutoverContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_workbench_pointer_routing_does_not_export_legacy_window_route(self) -> None:
        exports = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing.rs"
        )

        self.assertNotIn("route_pointer_to_workbench_window", exports)

    def test_workbench_hit_testing_has_no_linear_scan_fallback(self) -> None:
        entry = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs"
        )
        hit = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/hit.rs"
        )

        self.assertIn("hit_test_workbench_template_nodes_with_index", entry)
        self.assertIn("index.candidate_rows(x, y)", hit)
        self.assertNotIn("hit_test_workbench_window_template_node(", entry)
        self.assertNotIn("fn hit_test_workbench_template_nodes(", hit)
        self.assertNotIn("nodes.iter().rev().find", hit.replace("\n", ""))


if __name__ == "__main__":
    unittest.main()
