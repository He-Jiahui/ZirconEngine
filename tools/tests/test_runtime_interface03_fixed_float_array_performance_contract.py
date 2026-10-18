import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/component/value.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/component/value/tests/fixed_float_array_performance_tests.rs"


class RuntimeInterface03FixedFloatArrayPerformanceContract(unittest.TestCase):
    def test_fixed_float_array_fills_stack_storage(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn fixed_float_array<const N: usize>")
        end = source.index("\nfn float_array_to_toml", start)
        body = source[start:end]

        self.assertIn("let mut floats = [0.0; N]", body)
        self.assertIn("floats[index] =", body)
        self.assertNotIn("collect::<Option<Vec", body)
        self.assertNotIn("try_into()", body)

    def test_fixed_float_array_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("fixed_float_array_allocating", bench)
        self.assertIn("RUNTIME_INTERFACE03_STACK_FIXED_FLOAT_ARRAY_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)
        self.assertIn("mod fixed_float_array_performance_tests", source)


if __name__ == "__main__":
    unittest.main()
