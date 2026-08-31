from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MODULE = ROOT / (
    "zircon_editor/src/ui/retained_host/ui/workbench_window_projection"
)


class EditorWorkbenchParentIndexPerformanceContractTests(unittest.TestCase):
    def test_status_parent_walk_is_bounded_without_per_node_hash_allocation(self) -> None:
        node_index = (MODULE / "node_index.rs").read_text(encoding="utf-8")
        status = (MODULE / "status_right.rs").read_text(encoding="utf-8")
        compact = "".join(status.split())

        self.assertIn("pub(super) fn node_count(&self) -> usize", node_index)
        self.assertNotIn("HashSet", status)
        self.assertIn("for_in0..node_index.node_count()", compact)


if __name__ == "__main__":
    unittest.main()
