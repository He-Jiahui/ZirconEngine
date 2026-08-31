import pathlib
import unittest

from tools.ui_navigation_semantics_pressure import run, run_retained_domains


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/surface/navigation_index.rs"
SEMANTICS_SOURCE = ROOT / "zircon_runtime/src/ui/surface/navigation_index/semantics.rs"


class RuntimeUiNavigationSemanticsPressureContractTests(unittest.TestCase):
    def test_pressure_model_preserves_pointer_and_focus_semantics(self) -> None:
        result = run(candidate_count=128, input_update_count=128)
        self.assertTrue(result["semantic_gate_matches"])
        self.assertEqual(0, result["pointer_only_rebuild_count"])
        self.assertTrue(result["focus_change_detected"])
        self.assertGreater(result["scan_reduction_ratio"], 100.0)

    def test_navigation_source_keeps_conservative_unknown_cases(self) -> None:
        source = SOURCE.read_text(encoding="utf-8") + SEMANTICS_SOURCE.read_text(encoding="utf-8")
        self.assertIn("changed_node_ids.is_empty()", source)
        self.assertIn("self.build_error.is_some()", source)
        self.assertIn("let Some(node) = tree.nodes.get(node_id) else", source)

    def test_retained_domain_model_bounds_stable_text_and_style_work(self) -> None:
        result = run_retained_domains(
            surface_node_count=16384,
            candidate_count=4096,
            frame_count=4096,
            changed_nodes_per_frame=1,
            ancestor_depth=8,
        )
        self.assertTrue(result["operation_counts_only"])
        self.assertEqual(0, result["stable_update_navigation_rebuild_count"])
        self.assertEqual(4096, result["avoided_navigation_rebuild_count"])
        self.assertEqual(32768, result["new_ancestor_node_lookups"])
        self.assertGreater(result["tree_visit_reduction_ratio"], 1800.0)
        self.assertEqual(0, result["event_path_tree_scan_count"])

    def test_retained_gate_has_no_global_scan_or_per_node_path_allocation(self) -> None:
        source = SEMANTICS_SOURCE.read_text(encoding="utf-8")
        self.assertNotIn("tree.nodes.iter()", source)
        self.assertNotIn("let mut path = Vec", source)
        self.assertIn("while let Some(current_id) = current", source)


if __name__ == "__main__":
    unittest.main()
