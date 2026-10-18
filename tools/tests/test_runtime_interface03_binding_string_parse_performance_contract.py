import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/binding/model/parser.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/binding/model/parser/tests/string_performance_tests.rs"


class RuntimeInterface03BindingStringParsePerformanceContract(unittest.TestCase):
    def test_unescaped_binding_strings_use_a_slice_fast_path(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    fn parse_string(&mut self)")
        end = source.index("\n    fn parse_number", start)
        body = source[start:end]

        self.assertIn("find(['\"', '\\\\'])", body)
        self.assertIn("self.input[value_start..value_end].to_string()", body)
        self.assertIn("String::with_capacity", body)
        self.assertNotIn("let mut output = String::new()", body)

    def test_escaped_binding_strings_append_borrowed_chunks(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    fn parse_string(&mut self)")
        end = source.index("\n    fn parse_number", start)
        body = source[start:end]

        self.assertIn(
            "output.push_str(&self.input[chunk_start..special_index])", body
        )
        self.assertIn("chunk_start = self.index", body)
        self.assertIn("RUNTIME_INTERFACE03_CHUNKED_BINDING_STRING_BENCH_V1", BENCH.read_text(encoding="utf-8"))

    def test_binding_string_slice_fast_path_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("parse_string_growing", bench)
        self.assertIn("RUNTIME_INTERFACE03_SLICED_BINDING_STRING_BENCH_V1", bench)
        self.assertIn("RUNTIME_INTERFACE03_CHUNKED_BINDING_STRING_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)
        self.assertIn("mod string_performance_tests", source)


if __name__ == "__main__":
    unittest.main()
