import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/text_shape.rs"
BENCHMARK_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/text_shape/tests/performance_tests.rs"
)


class RuntimeInterface03TextLineMetricsIndexPerformanceContract(unittest.TestCase):
    def test_shaped_multi_run_lines_use_one_metric_index(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        self.assertIn("ShapedTextLineMetrics", source)
        self.assertIn("text_run_frame_with_metrics", source)
        self.assertIn("line.clusters.len() > 1", source)
        self.assertIn(".as_ref()", source)

    def test_resolved_multi_run_lines_reuse_validated_metrics(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        self.assertIn("ResolvedTextLineMetrics", source)
        self.assertIn("resolved_text_run_frame_with_metrics", source)
        self.assertIn("metrics.get_or_insert", source)
        self.assertIn("glyph_advances", source)

    def test_release_benchmarks_cover_indexed_geometry(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        benchmark = BENCHMARK_SOURCE.read_text(encoding="utf-8")
        self.assertIn("mod performance_tests;", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_INDEXED_SHAPED_TEXT_RUN_GEOMETRY_BENCH_V1", benchmark
        )
        self.assertIn(
            "RUNTIME_INTERFACE03_INDEXED_RESOLVED_TEXT_RUN_GEOMETRY_BENCH_V1",
            benchmark,
        )


if __name__ == "__main__":
    unittest.main()
