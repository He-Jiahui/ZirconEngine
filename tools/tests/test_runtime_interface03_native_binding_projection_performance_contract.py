import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
PROJECTION = ROOT / "zircon_runtime_interface/src/ui/binding/model/binding_value/projection.rs"
BINDING_CALL = ROOT / "zircon_runtime_interface/src/ui/binding/model/binding_call.rs"
TYPES = ROOT / "zircon_runtime_interface/src/ui/binding/model/binding_value/types.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/binding/model/binding_value/projection/tests/native_repr_performance_tests.rs"


class RuntimeInterface03NativeBindingProjectionPerformanceContract(unittest.TestCase):
    def test_nested_projection_writes_into_one_output_buffer(self) -> None:
        projection = PROJECTION.read_text(encoding="utf-8")
        binding_call = BINDING_CALL.read_text(encoding="utf-8")
        types = TYPES.read_text(encoding="utf-8")

        self.assertIn("fn native_repr_into", projection)
        self.assertIn("value.native_repr_into(output)", projection)
        self.assertNotIn(".map(Self::native_repr)", projection)
        self.assertNotIn(".join(\",\")", projection)
        self.assertIn("fn native_repr_into", binding_call)
        self.assertIn("argument.native_repr_into(output)", binding_call)
        self.assertIn("fn quoted_into", types)

    def test_native_binding_projection_has_release_benchmark(self) -> None:
        source = PROJECTION.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("native_repr_allocating", bench)
        self.assertIn("RUNTIME_INTERFACE03_SINGLE_BUFFER_NATIVE_BINDING_BENCH_V1", bench)
        self.assertIn("saturating_mul(2)", bench)
        self.assertIn("native_repr_into", source)


if __name__ == "__main__":
    unittest.main()
