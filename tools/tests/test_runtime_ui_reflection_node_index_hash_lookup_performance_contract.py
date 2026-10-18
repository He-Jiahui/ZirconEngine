"""Static contract for the Runtime UI reflection node-index lookup path."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MANAGER = ROOT / "zircon_runtime/src/ui/event_ui/manager/ui_event_manager.rs"
STORE = ROOT / "zircon_runtime/src/ui/event_ui/manager/reflection_store.rs"
TESTS = (
    ROOT / "zircon_runtime/src/ui/event_ui/manager/reflection_store/tests/optimization_tests.rs"
)


class RuntimeUiReflectionNodeIndexHashLookupContract(unittest.TestCase):
    def test_index_uses_hash_lookup_without_changing_ordered_owners(self) -> None:
        source = MANAGER.read_text(encoding="utf-8")
        self.assertIn("use std::collections::{BTreeMap, HashMap};", source)
        self.assertIn(
            "node_index: HashMap<UiNodePath, (UiTreeId, UiNodeId)>", source
        )
        self.assertNotIn(
            "node_index: BTreeMap<UiNodePath, (UiTreeId, UiNodeId)>", source
        )
        self.assertIn("routes_by_id: BTreeMap", source)
        self.assertIn("trees: BTreeMap", source)

    def test_rebuild_retains_capacity_and_preserves_deterministic_source_order(self) -> None:
        source = STORE.read_text(encoding="utf-8")
        self.assertIn(".sum::<usize>()", source)
        self.assertIn("self.node_index.clear();", source)
        self.assertIn("self.node_index.capacity()", source)
        self.assertIn("self.node_index.reserve(additional_capacity);", source)
        self.assertIn("for (tree_id, tree) in &self.trees", source)
        self.assertIn("for (node_id, node) in &tree.nodes", source)

    def test_regressions_cover_capacity_lookup_and_duplicate_path_winner(self) -> None:
        source = TESTS.read_text(encoding="utf-8")
        self.assertIn(
            "runtime11a_node_index_uses_reserved_hash_lookup_capacity", source
        )
        self.assertIn(
            "runtime11a_node_index_hash_lookup_preserves_duplicate_path_winner",
            source,
        )


if __name__ == "__main__":
    unittest.main()
