import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/frame_extract.rs"
BENCHMARK_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/frame_extract/tests/paint_element_capacity_tests.rs"
)


class RuntimeInterface03RenderFramePaintCapacityPerformanceContract(unittest.TestCase):
    def test_frame_projection_reserves_one_element_per_command(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("Vec::with_capacity(self.commands.len())", source)
        self.assertIn("to_paint_elements_with_metrics_unreserved", source)

    def test_release_benchmark_and_behavior_oracle_are_present(self) -> None:
        benchmark = BENCHMARK_SOURCE.read_text(encoding="utf-8")

        self.assertIn(
            "runtime_interface03_batch20_render_frame_capacity_preserves_projection",
            benchmark,
        )
        self.assertIn("RUNTIME_INTERFACE03_RENDER_FRAME_PAINT_CAPACITY_BENCH_V1", benchmark)


if __name__ == "__main__":
    unittest.main()
