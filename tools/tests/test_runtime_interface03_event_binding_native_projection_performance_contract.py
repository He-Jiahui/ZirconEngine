import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
EVENT_PATH = ROOT / "zircon_runtime_interface/src/ui/binding/model/event_path.rs"
EVENT_BINDING = ROOT / "zircon_runtime_interface/src/ui/binding/model/event_binding.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/binding/model/event_binding/tests/native_projection_performance_tests.rs"


class RuntimeInterface03EventBindingNativeProjectionPerformanceContract(unittest.TestCase):
    def test_event_binding_uses_one_native_output_buffer(self) -> None:
        event_path = EVENT_PATH.read_text(encoding="utf-8")
        event_binding = EVENT_BINDING.read_text(encoding="utf-8")
        start = event_binding.index("    pub fn native_binding(&self)")
        end = event_binding.index("\n    }", start) + len("\n    }")
        body = event_binding[start:end]

        self.assertIn("fn native_prefix_into", event_path)
        self.assertIn("self.path.native_prefix_into(&mut output)", body)
        self.assertIn("action.native_repr_into(&mut output)", body)
        self.assertNotIn("format!", body)

    def test_event_binding_projection_has_release_benchmark(self) -> None:
        source = EVENT_BINDING.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("native_binding_allocating", bench)
        self.assertIn("RUNTIME_INTERFACE03_SINGLE_BUFFER_EVENT_BINDING_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)
        self.assertIn("mod native_projection_performance_tests", source)


if __name__ == "__main__":
    unittest.main()
