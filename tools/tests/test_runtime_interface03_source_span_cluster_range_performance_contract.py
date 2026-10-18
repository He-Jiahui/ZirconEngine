import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line.rs"
)
BENCHMARK_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line/tests/performance_tests.rs"
)


class RuntimeInterface03SourceSpanClusterRangePerformanceContract(unittest.TestCase):
    def test_ordered_source_spans_use_binary_cluster_slice(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("source_clusters_for_range", source)
        self.assertIn("partition_point", source)
        self.assertIn("visual_spans_for_source_range_linear", source)
        self.assertIn("source_ranges_are_disjoint_and_ordered", source)

    def test_release_benchmark_and_behavior_oracle_are_present(self) -> None:
        benchmark = BENCHMARK_SOURCE.read_text(encoding="utf-8")

        self.assertIn(
            "runtime_interface03_batch15_binary_source_span_range_preserves_projection",
            benchmark,
        )
        self.assertIn(
            "RUNTIME_INTERFACE03_BINARY_SOURCE_SPAN_RANGE_BENCH_V1", benchmark
        )


if __name__ == "__main__":
    unittest.main()
