from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
REDUCER = ROOT / "zircon_runtime/src/ui/component/state_reducer/tree_view.rs"
SUPPORT = (
    ROOT
    / "zircon_runtime/src/ui/surface/surface/default_interactions/tree_view_support.rs"
)


class RuntimeTreeSelectionIndexPerformanceContractTests(unittest.TestCase):
    def test_component_tree_order_uses_borrowed_ids_and_hash_deduplication(self) -> None:
        source = REDUCER.read_text(encoding="utf-8")
        start = source.index("fn ordered_node_ids")
        end = source.index("\nfn current_tree_index", start)
        ordered = source[start:end]

        self.assertIn("Vec<&'a str>", ordered)
        self.assertIn("HashSet::new()", ordered)
        self.assertIn("collect_tree_node_ids(value, &mut node_ids, &mut seen)", ordered)

    def test_component_range_selection_builds_disabled_membership_once(self) -> None:
        source = REDUCER.read_text(encoding="utf-8")
        start = source.index("fn range_selected_node_ids")
        end = source.index("\nfn selected_control_property", start)
        selection = source[start:end]

        self.assertIn("disabled_option_ids(state)", selection)
        self.assertIn("disabled_ids.contains", selection)
        self.assertNotIn("option_is_disabled", selection)
        self.assertNotIn("push_unique", selection)

    def test_surface_tree_order_and_range_selection_share_the_linear_contract(self) -> None:
        source = SUPPORT.read_text(encoding="utf-8")
        ids_start = source.index("pub(super) fn tree_node_ids")
        ids_end = source.index("\npub(super) fn tree_nodes_property", ids_start)
        ids = source[ids_start:ids_end]
        range_start = source.index("pub(super) fn range_selected_ids")
        range_end = source.index("\npub(super) fn toggled_selected_ids", range_start)
        selection = source[range_start:range_end]

        self.assertIn("Vec<&'a str>", ids)
        self.assertIn("HashSet::new()", ids)
        self.assertIn("disabled_option_ids(metadata)", selection)
        self.assertNotIn("tree_option_is_disabled", selection)
        self.assertNotIn("push_unique", selection)


if __name__ == "__main__":
    unittest.main()
