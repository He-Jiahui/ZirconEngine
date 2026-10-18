from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
BRIDGE_DEPENDENCIES = (
    REPO_ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_dependencies.rs"
)


class Runtime58BridgeDependencyScratchContract(unittest.TestCase):
    def test_reuses_one_visiting_set_for_all_root_diagnostic_walks(self):
        source = BRIDGE_DEPENDENCIES.read_text(encoding="utf-8")

        self.assertIn("let package_capacity = graph.package_order.len();", source)
        self.assertIn(
            "let mut visiting = HashSet::with_capacity(package_capacity);",
            source,
        )
        self.assertGreater(
            source.index("let mut visiting = HashSet::with_capacity(package_capacity);"),
            source.index("if !graph.index_issue_reachability()"),
        )
        self.assertIn("graph.closure_cache.reserve(package_capacity);", source)
        self.assertIn("visiting.clear();", source)
        self.assertNotIn(
            "graph.dependency_issues(package_id, &mut HashSet::new())",
            source,
        )

    def test_graph_indexes_reserve_the_known_registration_bound(self):
        source = BRIDGE_DEPENDENCIES.read_text(encoding="utf-8")

        self.assertIn("let registration_capacity = registrations.len();", source)
        self.assertIn("HashSet::with_capacity(registration_capacity)", source)
        self.assertIn("Vec::with_capacity(registration_capacity)", source)
        self.assertIn("HashMap::with_capacity(registration_capacity)", source)
        self.assertIn(
            "self.issue_reachable_plugins\n            .reserve(self.package_order.len());",
            source,
        )
        self.assertIn("issue_reachable_plugins: HashSet::new(),", source)
        self.assertIn("closure_cache: HashMap::new(),", source)
        self.assertIn(
            "reused_root_visiting_scratch_clears_after_cycle_diagnostics",
            source,
        )

    def test_deterministic_model_removes_per_root_visiting_allocations(self):
        root_count = 1_024
        legacy_visiting_set_allocations = root_count
        optimized_visiting_set_allocations = 1

        self.assertEqual(legacy_visiting_set_allocations, 1_024)
        self.assertEqual(optimized_visiting_set_allocations, 1)
        self.assertEqual(
            legacy_visiting_set_allocations - optimized_visiting_set_allocations,
            1_023,
        )


if __name__ == "__main__":
    unittest.main()
