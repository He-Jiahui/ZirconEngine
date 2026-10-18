import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/command.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/surface/render/command/tests/rgba_hex_performance_tests.rs"


class RuntimeInterface03RgbaHexPerformanceContract(unittest.TestCase):
    def test_rgba_hex_uses_fixed_capacity_nibble_encoding(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn rgba_hex(")
        end = source.index("\n}\n", start) + len("\n}")
        body = source[start:end]

        self.assertIn("String::with_capacity(9)", body)
        self.assertIn("RGBA_HEX_UPPER", body)
        self.assertNotIn("format!(", body)

    def test_rgba_hex_has_behavior_and_release_contract(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod rgba_hex_performance_tests", source)
        self.assertIn("rgba_hex_formatting", bench)
        self.assertIn("RUNTIME_INTERFACE03_STACK_RGBA_HEX_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)


if __name__ == "__main__":
    unittest.main()
