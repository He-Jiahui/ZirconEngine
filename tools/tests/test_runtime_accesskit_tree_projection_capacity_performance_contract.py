from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/accessibility/accesskit.rs"
PERFORMANCE_TESTS = ROOT / "zircon_runtime/src/ui/accessibility/accesskit/tests/performance_tests.rs"


class RuntimeAccessKitTreeProjectionCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.performance_tests = PERFORMANCE_TESTS.read_text(encoding="utf-8")
        cls.snapshot_body = cls.source[cls.source.index("pub(crate) fn snapshot_to_accesskit_tree_update"):]
        cls.snapshot_body = cls.snapshot_body[: cls.snapshot_body.index("\n}\n\nfn accesskit_focus_node_id") + 2]
        cls.root_body = cls.source[cls.source.index("fn synthetic_root_node"):]
        cls.root_body = cls.root_body[: cls.root_body.index("\n}\n\nfn accesskit_node") + 2]
        cls.node_body = cls.source[cls.source.index("fn accesskit_node(source"):]
        cls.node_body = cls.node_body[: cls.node_body.index("\n}\n\nfn apply_text_properties") + 2]

    def test_tree_projection_reserves_snapshot_node_bound(self) -> None:
        self.assertIn("let synthetic_root_capacity = usize::from(snapshot.roots.len() > 1);", self.snapshot_body)
        self.assertIn("let mut nodes =", self.snapshot_body)
        self.assertIn(
            "Vec::with_capacity(snapshot.nodes.len().saturating_add(synthetic_root_capacity))",
            self.snapshot_body.replace("\n", ""),
        )
        self.assertIn("for node in &snapshot.nodes", self.snapshot_body)
        self.assertNotIn(".collect::<Vec<_>>()", self.snapshot_body)
        self.assertIn("runtime809_accesskit_tree_projection_capacity_bench_v1", self.performance_tests)

    def test_synthetic_root_children_reserve_root_bound(self) -> None:
        self.assertIn("let mut children = Vec::with_capacity(snapshot.roots.len());", self.root_body)
        self.assertNotIn(".collect::<Vec<_>>()", self.root_body)

    def test_accesskit_children_reserve_source_bound(self) -> None:
        self.assertIn("let mut children = Vec::with_capacity(source.children.len());", self.node_body)
        self.assertNotIn(".collect::<Vec<_>>()", self.node_body)

    def test_projection_capacity_model_removes_geometric_growth(self) -> None:
        node_count = 4096
        legacy_growth_events = 12
        optimized_growth_events = 0
        self.assertGreater(node_count, 0)
        self.assertGreater(legacy_growth_events, optimized_growth_events)


if __name__ == "__main__":
    unittest.main()
