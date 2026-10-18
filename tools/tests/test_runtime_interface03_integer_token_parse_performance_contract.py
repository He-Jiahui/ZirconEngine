import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/binding/expression.rs"
BENCH_SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/binding/expression/tests/integer_token_performance_tests.rs"


class RuntimeInterface03IntegerTokenParsePerformanceContract(unittest.TestCase):
    def test_valid_integer_token_parses_before_text_allocation(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn parse_number_or_ident(")
        end = source.index("\nfn parse_ident", start)
        body = source[start:end]

        integer_parse_index = body.index("parse_i64_chars")
        allocation_index = body.index("collect::<String>()")
        self.assertLess(integer_parse_index, allocation_index)
        self.assertIn("checked_mul", body)
        self.assertIn("checked_sub", body)

    def test_integer_token_parse_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench_source = BENCH_SOURCE.read_text(encoding="utf-8")

        self.assertIn("RUNTIME_INTERFACE03_STACK_INTEGER_TOKEN_BENCH_V1", bench_source)
        self.assertIn("parse_number_or_ident_allocating", source)
        self.assertIn("saturating_mul(5)", bench_source)
        self.assertIn("saturating_mul(4)", bench_source)


if __name__ == "__main__":
    unittest.main()
