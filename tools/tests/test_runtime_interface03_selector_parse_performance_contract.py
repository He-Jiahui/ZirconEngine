import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/style.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/template/asset/style/tests/selector_parse_performance_tests.rs"


class RuntimeInterface03SelectorParsePerformanceContract(unittest.TestCase):
    def test_selector_parser_borrows_compounds_from_source(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn parse(input: &str)")
        end = source.index("\n    pub fn specificity", start)
        body = source[start:end]

        self.assertIn("let mut remaining = input", body)
        self.assertIn("parse_compound_tokens(compound)", body)
        self.assertIn("split_whitespace_prefix", body)
        self.assertNotIn("let mut compound = String::new()", body)
        self.assertNotIn("compound.push", body)

    def test_selector_parse_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("parse_selector_allocating_compounds", bench)
        self.assertIn("RUNTIME_INTERFACE03_SLICED_SELECTOR_COMPOUND_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)
        self.assertIn("mod selector_parse_performance_tests", source)


if __name__ == "__main__":
    unittest.main()
