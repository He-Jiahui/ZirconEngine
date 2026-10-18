import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/binding/expression.rs"
BENCH_SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/binding/expression/tests/parser_token_performance_tests.rs"


class RuntimeInterface03ParserTokenTakePerformanceContract(unittest.TestCase):
    def test_parser_next_moves_token_without_clone(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    fn next(&mut self)")
        end = source.index("\n    }", start) + len("\n    }")
        body = source[start:end]

        self.assertNotIn(".cloned()", body)
        self.assertIn("std::mem::replace", body)
        self.assertIn("Token::Consumed", body)

    def test_parser_token_take_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench_source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn("fn next_cloned", source)
        self.assertIn("RUNTIME_INTERFACE03_PARSER_TOKEN_TAKE_BENCH_V1", bench_source)
        self.assertIn("saturating_mul(2)", bench_source)


if __name__ == "__main__":
    unittest.main()
