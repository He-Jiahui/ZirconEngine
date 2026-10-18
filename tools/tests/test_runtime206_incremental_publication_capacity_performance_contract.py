import unittest
from pathlib import Path


def _function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    open_brace = source.index("{", start)
    depth = 0
    for index in range(open_brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[open_brace + 1 : index]
    raise AssertionError(f"unterminated function body: {signature}")


class Runtime206IncrementalPublicationCapacityPerformanceContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.source_path = (
            self.repo_root
            / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/resource_publication.rs"
        )
        self.source = self.source_path.read_text(encoding="utf-8")
        self.body = _function_body(
            self.source,
            "pub(super) fn prepare_incremental_project_resource_sync",
        )

    def test_incremental_publication_reserves_each_bounded_projection(self) -> None:
        for expression in (
            "HashMap::with_capacity(updated_records.len())",
            "Vec::with_capacity(previous_source_records.len())",
            "HashMap::with_capacity(records_by_id.len())",
            "HashSet::with_capacity(previous_source_records.len())",
        ):
            self.assertIn(expression, self.body, expression)
        self.assertNotIn("HashMap::new()", self.body)
        self.assertNotIn("HashSet::new()", self.body)

    def test_incremental_publication_keeps_dedup_and_projection_order_boundaries(self) -> None:
        self.assertIn("records_by_id.insert(record.id(), record)", self.body)
        self.assertIn("record_updates: records_by_id.into_values().collect()", self.body)
        self.assertIn("source_path_updates: source_path_updates.into_iter().collect()", self.body)
        self.assertIn("source_path_removals", self.body)

    def test_lower_capacity_regression_and_benchmark_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"resource_publication/tests/incremental_capacity_tests.rs\"]",
            self.source,
        )
        lower = (
            self.repo_root / "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/resource_publication/tests/incremental_capacity_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "optimization_batch_20260915_runtime786_incremental_publication_capacity_source_shape",
            lower,
        )
        self.assertIn(
            "optimization_batch_20260915_runtime786_incremental_publication_capacity_upper_bound",
            lower,
        )
        self.assertIn(
            "optimization_batch_20260915_runtime786_incremental_publication_capacity_p95",
            lower,
        )
        self.assertIn(
            "RUNTIME786_INCREMENTAL_PUBLICATION_CAPACITY_BENCH_V1",
            lower,
        )
        self.assertIn("geometric_growth_events", lower)


if __name__ == "__main__":
    unittest.main()
