import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/binding/expression.rs"


class RuntimeInterface03BindingFloatArrayPerformanceContract(unittest.TestCase):
    def test_fixed_float_arguments_use_stack_array(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    fn expect_float_arguments<const N: usize>(")
        end = source.index("\n    #[cfg(test)]", start)
        body = source[start:end]

        self.assertIn("let mut values = [0.0; N];", body)
        self.assertNotIn("Vec::with_capacity(N)", body)
        self.assertNotIn("try_into()", body)

    def test_fixed_float_arguments_have_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("RUNTIME_INTERFACE03_STACK_FLOAT_ARGUMENTS_BENCH_V1", source)
        self.assertIn("expect_float_arguments_allocating", source)
        self.assertIn("saturating_mul(5)", source)


if __name__ == "__main__":
    unittest.main()
