
from tools.tests.rust_test_files import read_rust_test_file
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
VISUALIZER_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/visualizer.rs"
)
OVERDRAW_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/visualizer/overdraw.rs"
)
OVERLAY_TEST_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/visualizer/tests/overlay_performance_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    brace = source.index("{", start)
    depth = 0
    for index in range(brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[brace : index + 1]
    raise AssertionError(f"unterminated function body for {signature}")


class RuntimeInterface03VisualizerOverlayProjectionPerformanceContract(
    unittest.TestCase
):
    def test_batch_bounds_are_projected_once_and_reused_for_resource_overlay(self) -> None:
        source = VISUALIZER_SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "fn visualizer_overlays(")

        self.assertIn(
            "let batch_frame = batch_bounds(elements, &batch.source_indices);",
            body,
        )
        self.assertEqual(
            body.count("batch_bounds(elements, &batch.source_indices)"),
            1,
        )
        self.assertIn("mod overlay_performance_tests;", source)
        benchmark = OVERLAY_TEST_SOURCE.read_text(encoding="utf-8")
        self.assertIn(
            "RUNTIME_INTERFACE03_VISUALIZER_SINGLE_BATCH_BOUNDS_BENCH_V1",
            benchmark,
        )

    def test_overdraw_deduplicates_frames_before_member_projection(self) -> None:
        visualizer = VISUALIZER_SOURCE.read_text(encoding="utf-8")
        source = OVERDRAW_SOURCE.read_text(encoding="utf-8")

        self.assertIn("mod overdraw;", visualizer)
        self.assertIn("use overdraw::overdraw_regions;", visualizer)
        self.assertNotIn("fn overdraw_regions(", visualizer)
        self.assertIn("HashMap<[u32; 4], Vec<usize>>", source)
        self.assertIn("regions[region_index].frame == frame", source)
        admission = source.index("frame_index.insert_if_absent(frame, &regions)")
        member_projection = source.index("let mut node_ids")
        self.assertLess(admission, member_projection)
        self.assertIn(
            "RUNTIME_INTERFACE03_VISUALIZER_OVERDRAW_FRAME_INDEX_BENCH_V1",
            read_rust_test_file("zircon_runtime_interface/src/ui/surface/render/visualizer/tests/overdraw.rs"),
        )


if __name__ == "__main__":
    unittest.main()
