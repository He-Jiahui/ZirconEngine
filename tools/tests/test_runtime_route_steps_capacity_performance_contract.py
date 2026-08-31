import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
ROUTE_STEPS = ROOT / "zircon_runtime/src/ui/surface/input/route_steps.rs"


class RuntimeRouteStepsCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = ROUTE_STEPS.read_text(encoding="utf-8")
        cls.routed = cls.source.split("fn routed_path_steps(", 1)[1].split(
            "fn append_out_of_route_terminal_step", 1
        )[0]

    def test_routed_steps_preallocate_the_known_upper_bound(self) -> None:
        compact = "".join(self.routed.split())
        self.assertIn("trace.preview_tunnel.len()", compact)
        self.assertIn("path.len()", compact)
        self.assertIn("terminal_step_capacity", compact)
        self.assertIn("Vec::with_capacity(step_capacity)", compact)

    def test_empty_unrouted_steps_preserve_zero_allocation(self) -> None:
        self.assertIn("if step_capacity == 0", self.routed)
        self.assertIn("Vec::new()", self.routed)


if __name__ == "__main__":
    unittest.main()
