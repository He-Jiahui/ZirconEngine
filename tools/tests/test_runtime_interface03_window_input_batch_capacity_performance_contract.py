import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
ADAPTER = ROOT / "zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs"
PUMP = ROOT / "zircon_runtime_interface/src/ui/window/pump.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/window/runtime_event_adapter/tests/batch_capacity_performance_tests.rs"


class RuntimeInterface03WindowInputBatchCapacityPerformanceContract(unittest.TestCase):
    def test_batch_adapter_reserves_iterator_lower_bound(self) -> None:
        source = ADAPTER.read_text(encoding="utf-8")
        start = source.index("pub fn runtime_events_to_window_input_pump_batch(")
        end = source.index("\n}\n", start) + len("\n}")
        body = source[start:end]

        self.assertIn("let events = events.into_iter();", body)
        self.assertIn("events.size_hint().0", body)
        self.assertIn("UiWindowInputPumpBatch::with_capacity", body)
        self.assertNotIn("UiWindowInputPumpBatch::default()", body)

    def test_batch_capacity_api_has_behavior_and_release_contract(self) -> None:
        pump = PUMP.read_text(encoding="utf-8")
        adapter = ADAPTER.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("pub fn with_capacity(capacity: usize) -> Self", pump)
        self.assertIn("Vec::with_capacity(capacity)", pump)
        self.assertIn("mod batch_capacity_performance_tests", adapter)
        self.assertIn("RUNTIME_INTERFACE03_BATCH_CAPACITY_BENCH_V1", bench)
        self.assertIn("saturating_mul(2)", bench)


if __name__ == "__main__":
    unittest.main()
