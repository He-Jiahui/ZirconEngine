import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/text_shape.rs"
BENCHMARK_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/text_shape/tests/performance_tests.rs"
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


class RuntimeInterface03TextPaintRunCapacityPerformanceContract(unittest.TestCase):
    def test_shaped_projection_presizes_cluster_upper_bound(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "fn text_paint_runs_from_shaped(")
        compact = "".join(body.split())

        self.assertIn("line.clusters.len()", body)
        self.assertIn("Vec::with_capacity(run_capacity)", compact)
        self.assertNotIn("let mut runs = Vec::new();", body)
        self.assertIn("if cluster.text.is_empty()", body)

    def test_resolved_projection_presizes_run_upper_bound(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "fn text_paint_runs_from_resolved_layout(")
        compact = "".join(body.split())

        self.assertIn("line.runs.len()", body)
        self.assertIn("Vec::with_capacity(run_capacity)", compact)
        self.assertNotIn("let mut runs = Vec::new();", body)
        self.assertIn("if run.text.is_empty()", body)

    def test_release_benchmarks_cover_both_projection_paths(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        benchmark = BENCHMARK_SOURCE.read_text(encoding="utf-8")

        self.assertIn("mod performance_tests;", source)
        self.assertIn(
            "RUNTIME_INTERFACE03_PRESIZED_SHAPED_TEXT_RUNS_BENCH_V1", benchmark
        )
        self.assertIn(
            "RUNTIME_INTERFACE03_PRESIZED_RESOLVED_TEXT_RUNS_BENCH_V1",
            benchmark,
        )


if __name__ == "__main__":
    unittest.main()
