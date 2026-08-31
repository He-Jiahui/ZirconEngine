from pathlib import Path
import unittest

from tools.runtime_ui_responsive_width_gate_pressure import run


ROOT = Path(__file__).resolve().parents[2]
SLOT = ROOT / "zircon_runtime/src/ui/layout/pass/slot.rs"
RESPONSIVE = ROOT / "zircon_runtime/src/ui/layout/pass/responsive_mui.rs"
CANDIDATES = ROOT / "zircon_runtime/src/ui/layout/pass/responsive_mui/candidates.rs"
INCREMENTAL = ROOT / "zircon_runtime/src/ui/layout/pass/incremental.rs"


class RuntimeUiResponsiveWidthGateContractTests(unittest.TestCase):
    def test_index_tracks_thresholds_and_incremental_pass_uses_the_gate(self):
        slot = SLOT.read_text(encoding="utf-8")
        responsive = RESPONSIVE.read_text(encoding="utf-8")
        candidates = CANDIDATES.read_text(encoding="utf-8")
        incremental = INCREMENTAL.read_text(encoding="utf-8")

        self.assertIn("responsive_layout_may_change", slot)
        self.assertIn("last_responsive_width", candidates)
        self.assertIn("width_thresholds_by_node", candidates)
        self.assertIn("definitions_by_node", candidates)
        self.assertIn("responsive_definition_for_metadata", candidates)
        self.assertIn('"size"', candidates)
        self.assertIn('"offset"', candidates)
        self.assertIn("previous_definition != next_definition", candidates)
        self.assertIn("width_thresholds_for_metadata", candidates)
        self.assertIn("responsive_layout_may_change(root_size.width)", responsive)
        self.assertIn("previous <= threshold && next > threshold", candidates)
        self.assertIn("apply_mui_responsive_layout_indexed", incremental)

        indexed_body = responsive.split(
            "pub(super) fn apply_mui_responsive_layout_indexed", 1
        )[1].split("pub(super) fn apply_mui_responsive_layout_for_nodes", 1)[0]
        self.assertNotIn("MuiResponsiveCandidates::for_tree", indexed_body)
        self.assertNotIn("tree.nodes.keys()", indexed_body)

    def test_pressure_model_counts_only_responsive_candidate_work(self):
        result = run(
            responsive_candidate_count=10_000,
            resize_step_count=200,
            threshold_crossing_count=2,
        )

        self.assertEqual(result["old_candidate_visits"], 2_000_000)
        self.assertEqual(result["new_candidate_visits"], 30_000)
        self.assertEqual(result["eliminated_candidate_visits"], 1_970_000)
        self.assertAlmostEqual(result["candidate_visit_reduction_ratio"], 66.6666666667)

    def test_model_rejects_invalid_resize_inputs(self):
        with self.assertRaises(ValueError):
            run(0, 1, 1)
        with self.assertRaises(ValueError):
            run(10, 0, 1)
        with self.assertRaises(ValueError):
            run(10, 2, 3)


if __name__ == "__main__":
    unittest.main()
