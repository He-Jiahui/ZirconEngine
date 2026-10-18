import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line.rs"
)
BENCHMARK_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line/tests/performance_tests.rs"
)


class RuntimeInterface03SourceEdgeCachePerformanceContract(unittest.TestCase):
    def test_repeated_nonisomorphic_edges_use_a_lazy_range_index(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("UiTextSourceEdgeCache", source)
        self.assertIn("source_edge_bounds_by_range", source)
        self.assertIn("OnceLock<HashMap<(usize, usize), (usize, usize)>>", source)
        self.assertIn("SOURCE_EDGE_LINEAR_QUERY_LIMIT", source)
        self.assertIn("source_edge_visual_bounds_linear", source)

    def test_release_benchmark_and_behavior_oracle_are_present(self) -> None:
        benchmark = BENCHMARK_SOURCE.read_text(encoding="utf-8")

        self.assertIn(
            "runtime_interface03_batch16_cached_source_edges_preserve_projection",
            benchmark,
        )
        self.assertIn("RUNTIME_INTERFACE03_SOURCE_EDGE_CACHE_BENCH_V1", benchmark)


if __name__ == "__main__":
    unittest.main()
