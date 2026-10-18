import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/binding/expression.rs"


class RuntimeInterface03BindingKeywordTokenPerformanceContract(unittest.TestCase):
    def test_literal_keywords_match_before_identifier_allocation(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn parse_ident(")
        end = source.index("\nfn parse_error_from_token", start)
        body = source[start:end]

        match_index = body.index("match &chars[start..*index]")
        allocation_index = body.index("collect::<String>()")
        self.assertLess(match_index, allocation_index)
        self.assertIn("['t', 'r', 'u', 'e']", body)
        self.assertIn("['f', 'a', 'l', 's', 'e']", body)
        self.assertIn("['n', 'u', 'l', 'l']", body)

    def test_literal_keyword_dispatch_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("RUNTIME_INTERFACE03_BORROWED_KEYWORD_TOKEN_BENCH_V1", source)
        self.assertIn("parse_ident_allocating", source)
        self.assertIn("saturating_mul(2)", source)


if __name__ == "__main__":
    unittest.main()
