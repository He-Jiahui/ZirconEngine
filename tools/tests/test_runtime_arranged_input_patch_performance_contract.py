from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ARRANGED = ROOT / "zircon_runtime/src/ui/surface/arranged.rs"


class RuntimeArrangedInputPatchPerformanceContractTests(unittest.TestCase):
    def test_input_patch_does_not_rebuild_or_replace_complete_arranged_nodes(self) -> None:
        source = ARRANGED.read_text(encoding="utf-8")
        start = source.index("pub(crate) fn patch_arranged_tree_input(")
        end = source.index("\nfn collect_tree_descendants(", start)
        function = source[start:end]

        self.assertNotIn("arranged_node_from_tree", function)
        self.assertNotIn("*arranged_tree.nodes.get_mut(index)? = replacement", function)
        self.assertNotIn("Vec::with_capacity", function)
        self.assertGreaterEqual(function.count("for node_id in &affected_node_ids"), 2)
        self.assertIn("changed_node_ids.contains(node_id)", function)

    def test_descendants_remain_in_hit_patch_set_without_owned_structure_clones(self) -> None:
        source = ARRANGED.read_text(encoding="utf-8")
        start = source.index("pub(crate) fn patch_arranged_tree_input(")
        end = source.index("\nfn collect_tree_descendants(", start)
        function = source[start:end]

        self.assertIn("collect_tree_descendants", function)
        self.assertIn("Some(affected_node_ids)", function)
        self.assertNotIn("node_path.clone()", function)
        self.assertNotIn("children.clone()", function)


if __name__ == "__main__":
    unittest.main()
