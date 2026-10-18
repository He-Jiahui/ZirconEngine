import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line.rs"
)
BENCHMARK_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/line/tests/performance_tests.rs"
)


class RuntimeInterface03CaretSourceClusterLookupPerformanceContract(unittest.TestCase):
    def test_disjoint_source_clusters_use_binary_caret_lookup(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("source_ranges_are_disjoint_and_ordered", source)
        self.assertIn("visual_offset_for_ordered_caret", source)
        self.assertIn("partition_point", source)
        self.assertIn("visual_offset_for_caret_linear", source)

    def test_release_benchmark_and_behavior_oracle_are_present(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        benchmark = BENCHMARK_SOURCE.read_text(encoding="utf-8")

        self.assertIn("mod performance_tests;", source)
        self.assertIn(
            "runtime_interface03_batch14_binary_caret_source_lookup_preserves_affinity",
            benchmark,
        )
        self.assertIn(
            "RUNTIME_INTERFACE03_BINARY_CARET_SOURCE_LOOKUP_BENCH_V1", benchmark
        )


if __name__ == "__main__":
    unittest.main()
