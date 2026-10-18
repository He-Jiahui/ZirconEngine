import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/binding/expression.rs"


class RuntimeInterface03BindingTokenAdmissionPerformanceContract(unittest.TestCase):
    def test_tokenizer_stops_at_token_budget_during_scan(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn tokenize(")
        end = source.index("\nfn normalize_expression_input", start)
        body = source[start:end]

        self.assertIn("max_tokens: usize", body)
        self.assertIn("if tokens.len() > max_tokens", body)
        self.assertIn('binding_expression_budget_error("tokens", max_tokens)', body)

    def test_token_admission_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("RUNTIME_INTERFACE03_EARLY_TOKEN_ADMISSION_BENCH_V1", source)
        self.assertIn("tokenize_with_budget_unbounded_scan", source)
        self.assertIn("saturating_mul(2)", source)


if __name__ == "__main__":
    unittest.main()
