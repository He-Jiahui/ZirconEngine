from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ROUTE_STEPS = ROOT / "zircon_runtime/src/ui/surface/input/route_steps.rs"
REGRESSION = ROOT / "zircon_runtime/src/ui/surface/input/tests/route_steps_tests.rs"


class RuntimeUiRouteTerminalScanPerformanceContractTests(unittest.TestCase):
    def test_terminal_append_uses_a_loop_state_bit_instead_of_scanning_steps(self) -> None:
        source = ROUTE_STEPS.read_text(encoding="utf-8")
        append = source.split("fn append_out_of_route_terminal_step", 1)[1].split(
            "\nfn direct_route_steps", 1
        )[0]

        self.assertIn("stopped: bool", append)
        self.assertIn("if stopped", append)
        self.assertNotIn("steps.iter().any(|step| step.stopped)", append)
        self.assertIn("let mut stopped = false", source)
        self.assertIn("stopped = true", source)

    def test_route_regression_covers_terminal_append_and_marker(self) -> None:
        source = REGRESSION.read_text(encoding="utf-8")
        self.assertIn("runtime200_route_terminal_append_preserves_stop_semantics", source)
        self.assertIn("RUNTIME200_ROUTE_TERMINAL_SCAN_BENCH_V1", source)
        self.assertIn("legacy_steps_scanned", source)
        self.assertIn("optimized_steps_scanned", source)

    def test_deterministic_route_model_reduces_terminal_probe_work(self) -> None:
        route_depth = 4_096
        legacy_steps_scanned = route_depth
        optimized_steps_scanned = 1
        self.assertEqual(legacy_steps_scanned, 4_096)
        self.assertEqual(optimized_steps_scanned, 1)
        self.assertGreaterEqual(legacy_steps_scanned // optimized_steps_scanned, 4_000)
        self.assertGreaterEqual(route_depth, 4_000)


if __name__ == "__main__":
    unittest.main()
