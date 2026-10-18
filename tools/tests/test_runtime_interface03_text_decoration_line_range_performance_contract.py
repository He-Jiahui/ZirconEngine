import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs"
BENCHMARK_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/text_geometry/tests/decoration_performance_tests.rs"
)


class RuntimeInterface03TextDecorationLineRangePerformanceContract(unittest.TestCase):
    def test_decorations_partition_to_intersecting_line_range(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("fn intersecting_line_range(", source)
        self.assertIn("partition_point", source)
        self.assertIn("source_range.end <= range.start", source)
        self.assertIn("source_range.start < range.end", source)
        self.assertIn("for line_index in intersecting_line_range", source)
        self.assertNotIn("for line_index in 0..source_maps.lines.len()", source)

    def test_release_benchmark_and_behavior_oracle_are_present(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        benchmark = BENCHMARK_SOURCE.read_text(encoding="utf-8")

        self.assertIn("mod decoration_performance_tests;", source)
        self.assertIn(
            "runtime_interface03_batch12_binary_decoration_line_range_preserves_intersections",
            benchmark,
        )
        self.assertIn(
            "RUNTIME_INTERFACE03_BINARY_DECORATION_LINE_RANGE_BENCH_V1", benchmark
        )


if __name__ == "__main__":
    unittest.main()
