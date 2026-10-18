import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/template/asset/binding/expression.rs"


class RuntimeInterface03UnicodeEscapeStackPerformanceContract(unittest.TestCase):
    def test_unicode_escape_uses_fixed_stack_bytes(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn parse_unicode_escape(")
        end = source.index("\n#[cfg(test)]", start)
        body = source[start:end]

        self.assertIn("[0_u8; 4]", body)
        self.assertNotIn("collect::<String>()", body)
        self.assertIn("u32::from_str_radix", body)

    def test_unicode_escape_has_release_benchmark(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")

        self.assertIn("RUNTIME_INTERFACE03_STACK_UNICODE_ESCAPE_BENCH_V1", source)
        self.assertIn("parse_unicode_escape_allocating", source)
        self.assertIn("saturating_mul(2)", source)


if __name__ == "__main__":
    unittest.main()
