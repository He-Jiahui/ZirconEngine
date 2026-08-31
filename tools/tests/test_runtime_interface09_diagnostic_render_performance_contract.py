from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HOST_OUTPUT_ROOT = ROOT / "zircon_runtime_host/src/foreign_output"
STATE = HOST_OUTPUT_ROOT / "state.rs"
DIAGNOSTIC = HOST_OUTPUT_ROOT / "state/diagnostic.rs"


class RuntimeInterface09DiagnosticRenderPerformanceContractTests(unittest.TestCase):
    def test_diagnostic_renderer_uses_one_preallocated_string(self) -> None:
        state = STATE.read_text(encoding="utf-8")
        diagnostic = DIAGNOSTIC.read_text(encoding="utf-8")
        production = diagnostic[: diagnostic.index("#[cfg(test)]")]

        self.assertIn("mod diagnostic;", state)
        self.assertIn("render_diagnostic_line(self.metrics())", state)
        self.assertIn("String::with_capacity(DIAGNOSTIC_LINE_INITIAL_CAPACITY)", production)
        self.assertIn("fn push_u64(", production)
        self.assertIn("line.push_str(", production)
        self.assertNotIn("std::fmt::Write", production)
        self.assertNotIn("write!(", production)
        self.assertNotIn("Vec<", production)
        self.assertNotIn("vec![", production)
        self.assertNotIn("format!", production)
        self.assertNotIn(".join(", production)

    def test_diagnostic_contract_has_exact_output_and_release_evidence(self) -> None:
        source = DIAGNOSTIC.read_text(encoding="utf-8")

        self.assertIn("single_buffer_diagnostic_matches_legacy_output", source)
        self.assertIn("diagnostic_render_release_benchmark_evidence", source)
        self.assertIn("RUNTIME_INTERFACE09_DIAGNOSTIC_RENDER_BENCH_V1", source)
        self.assertIn("legacy_allocated_buffers=12", source)
        self.assertIn("optimized_allocated_buffers=1", source)
        self.assertIn("allocated_buffer_reduction_pct=91.667", source)


if __name__ == "__main__":
    unittest.main()
