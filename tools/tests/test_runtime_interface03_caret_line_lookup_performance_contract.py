import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs"
BENCHMARK_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/text_geometry/tests/caret_performance_tests.rs"
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


class RuntimeInterface03CaretLineLookupPerformanceContract(unittest.TestCase):
    def test_caret_lookup_uses_affinity_specific_binary_partition(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "fn caret_line<'a>(")

        self.assertIn("partition_point", body)
        self.assertIn("UiTextCaretAffinity::Upstream", body)
        self.assertIn("UiTextCaretAffinity::Downstream", body)
        self.assertIn("linear_caret_line", body)
        self.assertNotIn("layout.lines.iter().rev().find", "".join(body.split()))

    def test_release_benchmark_and_behavior_oracle_are_present(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        benchmark = BENCHMARK_SOURCE.read_text(encoding="utf-8")

        self.assertIn("mod caret_performance_tests;", source)
        self.assertIn(
            "runtime_interface03_batch11_binary_caret_line_lookup_preserves_affinity",
            benchmark,
        )
        self.assertIn(
            "RUNTIME_INTERFACE03_BINARY_CARET_LINE_LOOKUP_BENCH_V1", benchmark
        )


if __name__ == "__main__":
    unittest.main()
