import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
VISUALIZER_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/visualizer.rs"
)
BENCH_SOURCE = (
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


class RuntimeInterface03VisualizerOverlayCapacityPerformanceContract(
    unittest.TestCase
):
    def test_overlay_vector_reserves_wireframe_lower_bound(self) -> None:
        source = VISUALIZER_SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "fn visualizer_overlays(")

        self.assertIn("Vec::with_capacity(elements.len())", body)
        self.assertNotIn("let mut overlays = Vec::new()", body)

    def test_overlay_capacity_has_release_benchmark(self) -> None:
        source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn("RUNTIME_INTERFACE03_VISUALIZER_OVERLAY_CAPACITY_BENCH_V1", source)
        self.assertIn("reserved.capacity()", source)
        self.assertIn("unreserved.capacity()", source)
        self.assertIn("saturating_mul(3)", source)


if __name__ == "__main__":
    unittest.main()
