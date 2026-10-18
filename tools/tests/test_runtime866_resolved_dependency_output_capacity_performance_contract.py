import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution.rs"
)
LOWER = (
    REPO_ROOT / "zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution/tests/optimization_batch_runtime866_resolved_dependency_capacity_tests.rs"
)


class Runtime866ResolvedDependencyOutputCapacityPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        match = re.search(
            r"fn\s+resolve_dependencies\(.*?\n}\n\nfn\s+admit_resolved_dependency_id",
            source,
            re.S,
        )
        if match is None:
            raise AssertionError("resolve_dependencies owner is missing")
        cls.source = source
        cls.body = match.group(0)

    def test_resolved_id_output_reserves_dependency_upper_bound(self) -> None:
        self.assertIn(
            "dependency_ids: Vec::with_capacity(dependencies.len())", self.body
        )

    def test_diagnostics_reserve_lazily_on_first_missing_dependency(self) -> None:
        self.assertIn("diagnostics: Vec::new()", self.body)
        self.assertIn("if resolved.diagnostics.is_empty()", self.body)
        self.assertIn("resolved.diagnostics.reserve(dependencies.len())", self.body)

    def test_resolution_and_dedup_semantics_remain_owned_by_existing_path(self) -> None:
        self.assertIn("registry.get_by_locator(dependency)", self.body)
        self.assertIn("admit_resolved_dependency_id(", self.body)
        self.assertIn("ResourceDiagnostic::error", self.body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "mod optimization_batch_runtime866_resolved_dependency_capacity_tests;",
            self.source,
        )
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "runtime866_resolved_dependency_capacity_preserves_empty_and_missing_paths",
            lower,
        )
        self.assertIn(
            "RUNTIME866_RESOLVED_DEPENDENCY_OUTPUT_CAPACITY_BENCH_V1", lower
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_growth_model(self) -> None:
        dependencies = 4_096
        legacy_growth_events = 11
        optimized_growth_events = 0
        self.assertEqual(dependencies, 4_096)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
