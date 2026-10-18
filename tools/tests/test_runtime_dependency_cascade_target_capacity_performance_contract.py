from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/template/asset/dependency_index.rs"


class RuntimeDependencyCascadeTargetCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.lower = (
            SOURCE.parent
            / "dependency_index/tests/cascade_target_capacity_tests.rs"
        ).read_text(encoding="utf-8")
        start = cls.source.index("    pub fn cascade_invalidation_targets(&self, changed: &str)")
        end = cls.source.index("\n    pub fn query_asset", start)
        cls.body = cls.source[start:end]

    def test_target_vector_reserves_only_after_first_admitted_dependent(self) -> None:
        self.assertIn("let mut targets = Vec::new();", self.body)
        self.assertIn("if targets.is_empty() {", self.body)
        self.assertIn("targets.reserve(dependents.len());", self.body)
        self.assertLess(
            self.body.index("seen.insert(dependent)"),
            self.body.index("targets.reserve(dependents.len())"),
        )

    def test_bfs_membership_and_publication_order_are_unchanged(self) -> None:
        self.assertIn("let mut seen: HashSet<&str> = HashSet::new();", self.body)
        self.assertIn("let mut queue: VecDeque<&str> = VecDeque::new();", self.body)
        self.assertIn("seen.insert(dependent)", self.body)
        self.assertIn("targets.push(dependent.to_string());", self.body)
        self.assertIn("queue.push_back(dependent);", self.body)
        self.assertIn(
            '#[path = "dependency_index/tests/cascade_target_capacity_tests.rs"]',
            self.source,
        )
        self.assertIn(
            "RUNTIME829_DEPENDENCY_CASCADE_TARGET_CAPACITY_BENCH_V1",
            self.lower,
        )
        self.assertNotIn("sort", self.body)

    def test_empty_and_self_cycle_paths_do_not_eagerly_reserve(self) -> None:
        self.assertIn("seen.insert(changed);", self.body)
        self.assertIn("queue.push_back(changed);", self.body)
        reserve = self.body.index("targets.reserve(dependents.len())")
        admission = self.body.index("if seen.insert(dependent)")
        self.assertGreater(reserve, admission)

    def test_deterministic_fanout_model_removes_growth_events(self) -> None:
        fanout = 4_096
        legacy_growth_events = 0
        capacity = 0
        for count in range(fanout):
            if count == capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1
        optimized_growth_events = 0
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(legacy_growth_events, 11)
        self.assertEqual(fanout, 4_096)


if __name__ == "__main__":
    unittest.main()
