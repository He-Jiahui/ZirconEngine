import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/binding/model/event_binding.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/binding/model/event_binding/tests/capacity_performance_tests.rs"


class RuntimeInterface03EventBindingCapacityPerformanceContract(unittest.TestCase):
    def test_event_binding_preallocates_known_native_output(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn native_binding(&self)")
        end = source.index("\n    }", start) + len("\n    }")
        body = source[start:end]

        self.assertIn("String::with_capacity", body)
        self.assertIn("self.path.view_id.len()", body)
        self.assertIn("self.path.control_id.len()", body)
        self.assertIn("action.symbol.len()", body)
        self.assertNotIn("String::new()", body)

    def test_event_binding_capacity_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("native_binding_unpresized", bench)
        self.assertIn("RUNTIME_INTERFACE03_PRESIZED_EVENT_BINDING_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)
        self.assertIn("mod capacity_performance_tests", source)


if __name__ == "__main__":
    unittest.main()
