import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/ecs.rs"
BENCHMARK_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/ecs/tests/query_performance_tests.rs"
)


class RuntimeInterface03EcsChangeQueryFusionPerformanceContract(unittest.TestCase):
    def test_change_id_projection_filters_directly_without_reference_staging(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn node_ids_by_change_kind(")
        end = source.index("\n    }", start)
        body = source[start:end]

        self.assertIn("self.changes", body)
        self.assertIn(".iter()", body)
        self.assertIn("filter_map", body)
        self.assertNotIn("self.changes_by_kind", body)
        self.assertIn("node_ids_by_change_kind_staged", source)

    def test_release_benchmark_and_behavior_oracle_are_present(self) -> None:
        benchmark = BENCHMARK_SOURCE.read_text(encoding="utf-8")

        self.assertIn(
            "runtime_interface03_batch21_fused_change_id_query_preserves_order",
            benchmark,
        )
        self.assertIn("RUNTIME_INTERFACE03_FUSED_CHANGE_ID_QUERY_BENCH_V1", benchmark)


if __name__ == "__main__":
    unittest.main()
