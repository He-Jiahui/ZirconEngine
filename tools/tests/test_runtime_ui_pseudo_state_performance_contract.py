import pathlib
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
STYLE = REPO_ROOT / "zircon_runtime/src/ui/v2/style.rs"


class RuntimeUiPseudoStatePerformanceContractTests(unittest.TestCase):
    def test_descendant_probe_skips_selector_fact_clones_when_index_is_empty(self) -> None:
        source = STYLE.read_text(encoding="utf-8")
        start = source.index("pub(crate) fn node_state_can_affect_descendants")
        end = source.index("pub(crate) fn capture_baseline_from_tree", start)
        method = source[start:end]

        node_validation = method.index("UiTreeError::MissingNode")
        early_return = method.index("self.ancestor_pseudo_segments.is_empty()")
        self.assertLess(node_validation, early_return)
        self.assertNotIn("SelectorPathNode::from_tree_node", method)
        self.assertIn("matches_tree_node_segment_ignoring_state(segment, node)", method)

    def test_descendant_probe_matches_borrowed_tree_metadata(self) -> None:
        source = STYLE.read_text(encoding="utf-8")
        start = source.index("fn matches_tree_node_segment_ignoring_state")
        end = source.index("fn matches_segment_ignoring_state", start)
        helper = source[start:end]

        self.assertIn("node.template_metadata.as_ref()", helper)
        self.assertIn("metadata.component.as_str()", helper)
        self.assertIn("metadata.control_id.as_deref()", helper)
        self.assertIn("metadata.classes.iter()", helper)
        self.assertNotIn("clone()", helper)
        self.assertNotIn("to_owned()", helper)
        self.assertNotIn("to_string()", helper)


if __name__ == "__main__":
    unittest.main()
