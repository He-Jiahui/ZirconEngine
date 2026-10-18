import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/cluster.rs"
)
BENCHMARK_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/cluster/tests/performance_tests.rs"
)


class RuntimeInterface03VisualClusterRunCursorPerformanceContract(unittest.TestCase):
    def test_ordered_visual_runs_use_monotonic_cursor(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("runs_are_visually_ordered", source)
        self.assertIn("let mut run_start = 0;", source)
        self.assertIn("run_start += 1;", source)
        self.assertIn("cluster_for_visual_grapheme_in_runs", source)
        self.assertIn("visual_source_clusters_linear", source)

    def test_release_benchmark_and_behavior_oracle_are_present(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        benchmark = BENCHMARK_SOURCE.read_text(encoding="utf-8")

        self.assertIn("mod performance_tests;", source)
        self.assertIn(
            "runtime_interface03_batch13_visual_cluster_cursor_preserves_projection",
            benchmark,
        )
        self.assertIn(
            "RUNTIME_INTERFACE03_VISUAL_CLUSTER_RUN_CURSOR_BENCH_V1", benchmark
        )


if __name__ == "__main__":
    unittest.main()
