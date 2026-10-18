import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/style.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/template/asset/style/tests/selector_token_performance_tests.rs"


class RuntimeInterface03SelectorTokenizationPerformanceContract(unittest.TestCase):
    def test_compound_tokenizer_uses_source_byte_slices(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn parse_compound_tokens(")
        end = source.index("\n}\n\nfn split_whitespace_prefix", start) + len("\n}")
        body = source[start:end]

        self.assertIn("char_indices()", body)
        self.assertIn("&input[start..end]", body)
        self.assertNotIn("collect::<Vec", body)
        self.assertNotIn("let chars: Vec<char>", body)

    def test_selector_tokenization_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("parse_compound_tokens_allocating_chars", bench)
        self.assertIn("RUNTIME_INTERFACE03_SLICED_SELECTOR_TOKEN_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)
        self.assertIn("mod selector_token_performance_tests", source)


if __name__ == "__main__":
    unittest.main()
