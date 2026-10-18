import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/design_tokens/cascade_registry.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/design_tokens/cascade_registry/tests/color_hex_performance_tests.rs"


class RuntimeInterface03ColorTokenHexPerformanceContract(unittest.TestCase):
    def test_color_token_hex_uses_exact_capacity_nibble_encoding(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn color_token_hex(")
        end = source.index("\n}\n", start) + len("\n}")
        body = source[start:end]

        self.assertIn("String::with_capacity(1 + channel_count * 2)", body)
        self.assertIn("COLOR_HEX_LOWER", body)
        self.assertNotIn("format!(", body)

    def test_color_token_hex_has_behavior_and_release_contract(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod color_hex_performance_tests", source)
        self.assertIn("color_token_hex_formatting", bench)
        self.assertIn("RUNTIME_INTERFACE03_STACK_COLOR_TOKEN_HEX_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)


if __name__ == "__main__":
    unittest.main()
