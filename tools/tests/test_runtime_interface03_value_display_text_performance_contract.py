import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/component/value.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/component/value/tests/display_text_performance_tests.rs"


class RuntimeInterface03ValueDisplayTextPerformanceContract(unittest.TestCase):
    def test_vector_display_text_uses_one_output_buffer(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn display_text(&self)")
        end = source.index("\n    /// Converts a TOML value", start)
        body = source[start:end]

        self.assertIn("Self::Vec2(value) => vector_display_text(value)", body)
        self.assertIn("Self::Vec3(value) => vector_display_text(value)", body)
        self.assertIn("Self::Vec4(value) => vector_display_text(value)", body)
        self.assertNotIn('format!("{}, {}"', body)
        self.assertIn("String::with_capacity", source)
        self.assertIn("write_trim_float", source)

    def test_vector_display_text_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("display_text_allocating", bench)
        self.assertIn("RUNTIME_INTERFACE03_SINGLE_BUFFER_VALUE_DISPLAY_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)
        self.assertIn("mod display_text_performance_tests", source)


if __name__ == "__main__":
    unittest.main()
