from __future__ import annotations

import unittest
from pathlib import Path

from tools.runtime_ui_accessibility_extraction_pressure import run


ROOT = Path(__file__).resolve().parents[2]
EXTRACT = ROOT / "zircon_runtime/src/ui/accessibility/extract.rs"
RESOLUTION = ROOT / "zircon_runtime/src/ui/accessibility/extract/resolution.rs"
VISIBILITY = ROOT / "zircon_runtime/src/ui/accessibility/extract/visibility.rs"


class RuntimeUiAccessibilityExtractionPerformanceContract(unittest.TestCase):
    def test_pressure_model_counts_repeated_structural_edge_visits(self) -> None:
        report = run(
            tree_node_count=16_384,
            accessibility_node_count=8_192,
            hidden_relation_target_count=128,
        )

        self.assertEqual(
            report["retired_repeated_traversal"]["structural_edge_visits"],
            402_628_608,
        )
        self.assertEqual(
            report["indexed_extraction"]["structural_edge_visits"],
            32_766,
        )
        self.assertEqual(
            report["delta"]["structural_edge_visit_reduction_ratio"],
            12_288.0,
        )
        self.assertFalse(report["interpretation"]["runtime_cpu_measured"])

    def test_effective_hidden_state_is_precomputed_once(self) -> None:
        extract = EXTRACT.read_text(encoding="utf-8")
        visibility = VISIBILITY.read_text(encoding="utf-8")

        self.assertIn("mod visibility;", extract)
        self.assertEqual(extract.count("EffectiveHiddenIndex::build("), 1)
        self.assertNotIn("fn is_effectively_hidden(", extract)
        self.assertIn("pub(super) struct EffectiveHiddenIndex", visibility)
        self.assertIn("hidden_by_node: BTreeMap<UiNodeId, bool>", visibility)
        self.assertIn("effective_hidden_index_propagates_hidden_ancestors", visibility)

    def test_child_filtering_starts_only_from_published_nodes(self) -> None:
        resolution = RESOLUTION.read_text(encoding="utf-8")
        filter_start = resolution.index("pub(super) fn filter_children(")
        filter_body = resolution[filter_start : resolution.index("fn labelled_by_name(")]

        self.assertIn("let included_node_ids = nodes.keys().copied().collect::<Vec<_>>();", filter_body)
        self.assertIn("for node_id in included_node_ids", filter_body)
        self.assertNotIn("for node in surface.tree.nodes.values()", filter_body)

    def test_hidden_relation_targets_are_cleared_before_vec_publication(self) -> None:
        extract = EXTRACT.read_text(encoding="utf-8")
        clear = extract.index("for hidden_target in &hidden_relation_targets")
        publication = extract.index("nodes: nodes.into_values().collect()")

        self.assertLess(clear, publication)
        self.assertIn("nodes.get_mut(hidden_target)", extract[clear:publication])
        self.assertNotIn("snapshot.nodes.iter_mut().find", extract)


if __name__ == "__main__":
    unittest.main()
